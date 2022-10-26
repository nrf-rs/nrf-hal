//! HAL interface to the PWM peripheral.
//!
//! The pulse with modulation (PWM) module enables the generation of pulse width modulated signals on GPIO.

#[cfg(not(any(feature = "9160", feature = "5340-app")))]
use crate::pac::pwm0::*;
#[cfg(any(feature = "9160", feature = "5340-app"))]
use crate::pac::pwm0_ns::*;

use crate::{
    gpio::{Output, Pin, PushPull},
    pac::Interrupt,
    target_constants::{SRAM_LOWER, SRAM_UPPER},
    time::*,
};
use core::{
    cell::Cell,
    ops::Deref,
    sync::atomic::{compiler_fence, Ordering},
};
use embedded_dma::*;

const MAX_SEQ_LEN: usize = 0x7FFF;

/// A safe wrapper around the raw peripheral.
#[derive(Debug)]
pub struct Pwm<T: Instance> {
    pwm: T,
}

impl<T> Pwm<T>
where
    T: Instance,
{
    /// Takes ownership of the peripheral and applies sane defaults.
    pub fn new(pwm: T) -> Pwm<T> {
        compiler_fence(Ordering::SeqCst);
        pwm.enable.write(|w| w.enable().enabled());
        pwm.mode.write(|w| w.updown().up());
        pwm.prescaler.write(|w| w.prescaler().div_1());
        pwm.countertop
            .write(|w| unsafe { w.countertop().bits(32767) });
        pwm.loop_.write(|w| w.cnt().disabled());
        pwm.decoder.write(|w| {
            w.load().individual();
            w.mode().refresh_count()
        });
        pwm.seq0.refresh.write(|w| unsafe { w.bits(0) });
        pwm.seq0.enddelay.write(|w| unsafe { w.bits(0) });
        pwm.seq1.refresh.write(|w| unsafe { w.bits(0) });
        pwm.seq1.enddelay.write(|w| unsafe { w.bits(0) });

        Self { pwm }
    }

    /// Sets the PWM clock prescaler.
    #[inline(always)]
    pub fn set_prescaler(&self, div: Prescaler) -> &Self {
        self.pwm.prescaler.write(|w| w.prescaler().bits(div.into()));
        self
    }

    /// Sets the PWM clock prescaler.
    #[inline(always)]
    pub fn prescaler(&self) -> Prescaler {
        match self.pwm.prescaler.read().prescaler().bits() {
            0 => Prescaler::Div1,
            1 => Prescaler::Div2,
            2 => Prescaler::Div4,
            3 => Prescaler::Div8,
            4 => Prescaler::Div16,
            5 => Prescaler::Div32,
            6 => Prescaler::Div64,
            7 => Prescaler::Div128,
            _ => unreachable!(),
        }
    }

    /// Sets the maximum duty cycle value.
    #[inline(always)]
    pub fn set_max_duty(&self, duty: u16) -> &Self {
        self.pwm
            .countertop
            .write(|w| unsafe { w.countertop().bits(duty.min(32767u16)) });
        self
    }
    /// Returns the maximum duty cycle value.
    #[inline(always)]
    pub fn max_duty(&self) -> u16 {
        self.pwm.countertop.read().countertop().bits()
    }

    /// Sets the PWM output frequency.
    #[inline(always)]
    pub fn set_period(&self, freq: Hertz) -> &Self {
        let duty = match self.prescaler() {
            Prescaler::Div1 => 16_000_000u32 / freq.0,
            Prescaler::Div2 => 8_000_000u32 / freq.0,
            Prescaler::Div4 => 4_000_000u32 / freq.0,
            Prescaler::Div8 => 2_000_000u32 / freq.0,
            Prescaler::Div16 => 1_000_000u32 / freq.0,
            Prescaler::Div32 => 500_000u32 / freq.0,
            Prescaler::Div64 => 250_000u32 / freq.0,
            Prescaler::Div128 => 125_000u32 / freq.0,
        };
        match self.counter_mode() {
            CounterMode::Up => self.set_max_duty(duty.min(32767) as u16),
            CounterMode::UpAndDown => self.set_max_duty((duty / 2).min(32767) as u16),
        };
        self
    }

    /// Returns the PWM output frequency.
    #[inline(always)]
    pub fn period(&self) -> Hertz {
        let max_duty = self.max_duty() as u32;
        let freq = match self.prescaler() {
            Prescaler::Div1 => 16_000_000u32 / max_duty,
            Prescaler::Div2 => 8_000_000u32 / max_duty,
            Prescaler::Div4 => 4_000_000u32 / max_duty,
            Prescaler::Div8 => 2_000_000u32 / max_duty,
            Prescaler::Div16 => 1_000_000u32 / max_duty,
            Prescaler::Div32 => 500_000u32 / max_duty,
            Prescaler::Div64 => 250_000u32 / max_duty,
            Prescaler::Div128 => 125_000u32 / max_duty,
        };
        match self.counter_mode() {
            CounterMode::Up => freq.hz(),
            CounterMode::UpAndDown => (freq / 2).hz(),
        }
    }

    /// Sets the associated output pin for the PWM channel.
    ///
    /// Modifying the pin configuration while the PWM instance is enabled is not recommended.
    #[inline(always)]
    pub fn set_output_pin(&self, channel: Channel, pin: Pin<Output<PushPull>>) -> &Self {
        self.pwm.psel.out[usize::from(channel)].write(|w| {
            unsafe { w.bits(pin.psel_bits()) };
            w.connect().connected()
        });
        self
    }

    /// Sets the output pin of `channel`, and returns the old pin (if any).
    ///
    /// Modifying the pin configuration while the PWM instance is enabled is not recommended.
    pub fn swap_output_pin(
        &mut self,
        channel: Channel,
        pin: Pin<Output<PushPull>>,
    ) -> Option<Pin<Output<PushPull>>> {
        // (needs `&mut self` because it reads, then writes, to the register)
        let psel = &self.pwm.psel.out[usize::from(channel)];
        let old = psel.read();
        let old = if old.connect().is_connected() {
            unsafe { Some(Pin::from_psel_bits(old.bits())) }
        } else {
            None
        };
        self.set_output_pin(channel, pin);
        old
    }

    /// Disables the output pin of `channel`.
    ///
    /// The output pin is returned, if one was previously configured.
    ///
    /// Modifying the pin configuration while the PWM instance is enabled is not recommended.
    pub fn clear_output_pin(&mut self, channel: Channel) -> Option<Pin<Output<PushPull>>> {
        // (needs `&mut self` because it reads, then writes, to the register)
        let psel = &self.pwm.psel.out[usize::from(channel)];
        let old = psel.read();
        let old = if old.connect().is_connected() {
            unsafe { Some(Pin::from_psel_bits(old.bits())) }
        } else {
            None
        };
        psel.reset();
        old
    }

    /// Enables the PWM generator.
    #[inline(always)]
    pub fn enable(&self) {
        self.pwm.enable.write(|w| w.enable().enabled());
    }

    /// Disables the PWM generator.
    #[inline(always)]
    pub fn disable(&self) {
        self.pwm.enable.write(|w| w.enable().disabled());
    }

    /// Enables a PWM channel.
    #[inline(always)]
    pub fn enable_channel(&self, channel: Channel) -> &Self {
        self.pwm.psel.out[usize::from(channel)].modify(|_r, w| w.connect().connected());
        self
    }

    /// Disables a PWM channel.
    #[inline(always)]
    pub fn disable_channel(&self, channel: Channel) -> &Self {
        self.pwm.psel.out[usize::from(channel)].modify(|_r, w| w.connect().disconnected());
        self
    }

    /// Enables a PWM group.
    #[inline(always)]
    pub fn enable_group(&self, group: Group) -> &Self {
        match group {
            Group::G0 => {
                self.pwm.psel.out[0].modify(|_r, w| w.connect().connected());
                self.pwm.psel.out[1].modify(|_r, w| w.connect().connected());
            }
            Group::G1 => {
                self.pwm.psel.out[2].modify(|_r, w| w.connect().connected());
                self.pwm.psel.out[3].modify(|_r, w| w.connect().connected());
            }
        }
        self
    }

    /// Disables a PWM group.
    #[inline(always)]
    pub fn disable_group(&self, group: Group) -> &Self {
        match group {
            Group::G0 => {
                self.pwm.psel.out[0].modify(|_r, w| w.connect().disconnected());
                self.pwm.psel.out[1].modify(|_r, w| w.connect().disconnected());
            }
            Group::G1 => {
                self.pwm.psel.out[2].modify(|_r, w| w.connect().disconnected());
                self.pwm.psel.out[3].modify(|_r, w| w.connect().disconnected());
            }
        }
        self
    }

    /// Cofigures how a sequence is read from RAM and is spread to the compare register.
    #[inline(always)]
    pub fn set_load_mode(&self, mode: LoadMode) -> &Self {
        self.pwm.decoder.modify(|_r, w| w.load().bits(mode.into()));
        if mode == LoadMode::Waveform {
            self.disable_channel(Channel::C3);
        } else {
            self.enable_channel(Channel::C3);
        }
        self
    }

    /// Returns how a sequence is read from RAM and is spread to the compare register.
    #[inline(always)]
    pub fn load_mode(&self) -> LoadMode {
        match self.pwm.decoder.read().load().bits() {
            0 => LoadMode::Common,
            1 => LoadMode::Grouped,
            2 => LoadMode::Individual,
            3 => LoadMode::Waveform,
            _ => unreachable!(),
        }
    }

    /// Selects operating mode of the wave counter.
    #[inline(always)]
    pub fn set_counter_mode(&self, mode: CounterMode) -> &Self {
        self.pwm.mode.write(|w| w.updown().bit(mode.into()));
        self
    }

    /// Returns selected operating mode of the wave counter.
    #[inline(always)]
    pub fn counter_mode(&self) -> CounterMode {
        match self.pwm.mode.read().updown().bit() {
            false => CounterMode::Up,
            true => CounterMode::UpAndDown,
        }
    }

    /// Selects source for advancing the active sequence.
    #[inline(always)]
    pub fn set_step_mode(&self, mode: StepMode) -> &Self {
        self.pwm.decoder.modify(|_r, w| w.mode().bit(mode.into()));
        self
    }

    /// Returns selected source for advancing the active sequence.
    #[inline(always)]
    pub fn step_mode(&self) -> StepMode {
        match self.pwm.decoder.read().mode().bit() {
            false => StepMode::Auto,
            true => StepMode::NextStep,
        }
    }

    // Internal helper function that returns 15 bit duty cycle value.
    #[inline(always)]
    fn duty_on_value(&self, index: usize) -> u16 {
        let val = T::buffer().get()[index];
        let is_inverted = (val >> 15) & 1 == 0;
        match is_inverted {
            false => val,
            true => self.max_duty() - (val & 0x7FFF),
        }
    }

    // Internal helper function that returns 15 bit inverted duty cycle value.
    #[inline(always)]
    fn duty_off_value(&self, index: usize) -> u16 {
        let val = T::buffer().get()[index];
        let is_inverted = (val >> 15) & 1 == 0;
        match is_inverted {
            false => self.max_duty() - val,
            true => val & 0x7FFF,
        }
    }

    /// Sets duty cycle (15 bit) for all PWM channels.
    /// Will replace any ongoing sequence playback.
    pub fn set_duty_on_common(&self, duty: u16) {
        let mut buffer = T::buffer().get();
        buffer.copy_from_slice(&[duty.min(self.max_duty()) & 0x7FFF; 4][..]);
        T::buffer().set(buffer);
        self.one_shot();
        self.set_load_mode(LoadMode::Common);
        self.pwm
            .seq0
            .ptr
            .write(|w| unsafe { w.bits(T::buffer().as_ptr() as u32) });
        self.pwm.seq0.cnt.write(|w| unsafe { w.bits(1) });
        self.start_seq(Seq::Seq0);
    }

    /// Sets inverted duty cycle (15 bit) for all PWM channels.
    /// Will replace any ongoing sequence playback.
    pub fn set_duty_off_common(&self, duty: u16) {
        let mut buffer = T::buffer().get();
        buffer.copy_from_slice(&[duty.min(self.max_duty()) | 0x8000; 4][..]);
        T::buffer().set(buffer);
        self.one_shot();
        self.set_load_mode(LoadMode::Common);
        self.pwm
            .seq0
            .ptr
            .write(|w| unsafe { w.bits(T::buffer().as_ptr() as u32) });
        self.pwm.seq0.cnt.write(|w| unsafe { w.bits(1) });
        self.start_seq(Seq::Seq0);
    }

    /// Returns the common duty cycle value for all PWM channels in `Common` load mode.
    #[inline(always)]
    pub fn duty_on_common(&self) -> u16 {
        self.duty_on_value(0)
    }

    /// Returns the inverted common duty cycle value for all PWM channels in `Common` load mode.
    #[inline(always)]
    pub fn duty_off_common(&self) -> u16 {
        self.duty_off_value(0)
    }

    /// Sets duty cycle (15 bit) for a PWM group.
    /// Will replace any ongoing sequence playback.
    pub fn set_duty_on_group(&self, group: Group, duty: u16) {
        let mut buffer = T::buffer().get();
        buffer[usize::from(group)] = duty.min(self.max_duty()) & 0x7FFF;
        T::buffer().set(buffer);
        self.one_shot();
        self.set_load_mode(LoadMode::Grouped);
        self.pwm
            .seq0
            .ptr
            .write(|w| unsafe { w.bits(T::buffer().as_ptr() as u32) });
        self.pwm.seq0.cnt.write(|w| unsafe { w.bits(2) });
        self.start_seq(Seq::Seq0);
    }

    /// Sets inverted duty cycle (15 bit) for a PWM group.
    /// Will replace any ongoing sequence playback.
    pub fn set_duty_off_group(&self, group: Group, duty: u16) {
        let mut buffer = T::buffer().get();
        buffer[usize::from(group)] = duty.min(self.max_duty()) | 0x8000;
        T::buffer().set(buffer);
        self.one_shot();
        self.set_load_mode(LoadMode::Grouped);
        self.pwm
            .seq0
            .ptr
            .write(|w| unsafe { w.bits(T::buffer().as_ptr() as u32) });
        self.pwm.seq0.cnt.write(|w| unsafe { w.bits(2) });
        self.start_seq(Seq::Seq0);
    }

    /// Returns duty cycle value for a PWM group.
    #[inline(always)]
    pub fn duty_on_group(&self, group: Group) -> u16 {
        self.duty_on_value(usize::from(group))
    }

    /// Returns inverted duty cycle value for a PWM group.
    #[inline(always)]
    pub fn duty_off_group(&self, group: Group) -> u16 {
        self.duty_off_value(usize::from(group))
    }

    /// Sets duty cycle (15 bit) for a PWM channel.
    /// Will replace any ongoing sequence playback and the other channels will return to their previously set value.
    pub fn set_duty_on(&self, channel: Channel, duty: u16) {
        let mut buffer = T::buffer().get();
        buffer[usize::from(channel)] = duty.min(self.max_duty()) & 0x7FFF;
        T::buffer().set(buffer);
        self.one_shot();
        self.set_load_mode(LoadMode::Individual);
        self.pwm
            .seq0
            .ptr
            .write(|w| unsafe { w.bits(T::buffer().as_ptr() as u32) });
        self.pwm.seq0.cnt.write(|w| unsafe { w.bits(4) });
        self.start_seq(Seq::Seq0);
    }

    /// Sets inverted duty cycle (15 bit) for a PWM channel.
    /// Will replace any ongoing sequence playback and the other channels will return to their previously set value.
    pub fn set_duty_off(&self, channel: Channel, duty: u16) {
        let mut buffer = T::buffer().get();
        buffer[usize::from(channel)] = duty.min(self.max_duty()) | 0x8000;
        T::buffer().set(buffer);
        self.one_shot();
        self.set_load_mode(LoadMode::Individual);
        self.pwm
            .seq0
            .ptr
            .write(|w| unsafe { w.bits(T::buffer().as_ptr() as u32) });
        self.pwm.seq0.cnt.write(|w| unsafe { w.bits(4) });
        self.start_seq(Seq::Seq0);
    }

    /// Returns the duty cycle value for a PWM channel.
    #[inline(always)]
    pub fn duty_on(&self, channel: Channel) -> u16 {
        self.duty_on_value(usize::from(channel))
    }

    /// Returns the inverted duty cycle value for a PWM group.
    #[inline(always)]
    pub fn duty_off(&self, channel: Channel) -> u16 {
        self.duty_off_value(usize::from(channel))
    }

    /// Sets number of playbacks of sequences.
    #[inline(always)]
    pub fn set_loop(&self, mode: Loop) {
        self.pwm.loop_.write(|w| match mode {
            Loop::Disabled => w.cnt().disabled(),
            Loop::Times(n) => unsafe { w.cnt().bits(n) },
            Loop::Inf => unsafe { w.cnt().bits(2) },
        });
        self.pwm.shorts.write(|w| match mode {
            Loop::Inf => w.loopsdone_seqstart0().enabled(),
            _ => w.loopsdone_seqstart0().disabled(),
        });
    }

    /// Looping disabled (stop at the end of the sequence).
    #[inline(always)]
    pub fn one_shot(&self) -> &Self {
        self.set_loop(Loop::Disabled);
        self
    }

    /// Loops playback of sequences indefinitely.
    #[inline(always)]
    pub fn loop_inf(&self) -> &Self {
        self.set_loop(Loop::Inf);
        self
    }

    /// Sets number of playbacks of sequences.
    #[inline(always)]
    pub fn repeat(&self, times: u16) -> &Self {
        self.set_loop(Loop::Times(times));
        self
    }

    /// Sets number of additional PWM periods between samples loaded into compare register.
    #[inline(always)]
    pub fn set_seq_refresh(&self, seq: Seq, periods: u32) -> &Self {
        match seq {
            Seq::Seq0 => self.pwm.seq0.refresh.write(|w| unsafe { w.bits(periods) }),
            Seq::Seq1 => self.pwm.seq1.refresh.write(|w| unsafe { w.bits(periods) }),
        }
        self
    }

    /// Sets number of additional PWM periods after the sequence ends.
    #[inline(always)]
    pub fn set_seq_end_delay(&self, seq: Seq, periods: u32) -> &Self {
        match seq {
            Seq::Seq0 => self.pwm.seq0.enddelay.write(|w| unsafe { w.bits(periods) }),
            Seq::Seq1 => self.pwm.seq1.enddelay.write(|w| unsafe { w.bits(periods) }),
        }
        self
    }

    /// Loads the first PWM value on all enabled channels from a sequence and starts playing that sequence.
    /// Causes PWM generation to start if not running.
    #[inline(always)]
    pub fn start_seq(&self, seq: Seq) {
        compiler_fence(Ordering::SeqCst);
        self.pwm.enable.write(|w| w.enable().enabled());
        self.pwm.tasks_seqstart[usize::from(seq)].write(|w| unsafe { w.bits(1) });
        while self.pwm.events_seqstarted[usize::from(seq)].read().bits() == 0 {}
        self.pwm.events_seqend[0].write(|w| w);
        self.pwm.events_seqend[1].write(|w| w);
    }

    /// Steps by one value in the current sequence on all enabled channels, if the `NextStep` step mode is selected.
    /// Does not cause PWM generation to start if not running.
    #[inline(always)]
    pub fn next_step(&self) {
        self.pwm.tasks_nextstep.write(|w| unsafe { w.bits(1) });
    }

    /// Stops PWM pulse generation on all channels at the end of current PWM period, and stops sequence playback.
    #[inline(always)]
    pub fn stop(&self) {
        compiler_fence(Ordering::SeqCst);
        self.pwm.tasks_stop.write(|w| unsafe { w.bits(1) });
        while self.pwm.events_stopped.read().bits() == 0 {}
    }

    /// Loads the given sequence buffers and optionally (re-)starts sequence playback.
    /// Returns a `PemSeq`, containing `Pwm<T>` and the buffers.
    #[allow(unused_mut)]
    pub fn load<B0, B1>(
        mut self,
        seq0_buffer: Option<B0>,
        seq1_buffer: Option<B1>,
        start: bool,
    ) -> Result<PwmSeq<T, B0, B1>, (Error, Pwm<T>, Option<B0>, Option<B1>)>
    where
        B0: ReadBuffer<Word = u16> + 'static,
        B1: ReadBuffer<Word = u16> + 'static,
    {
        if let Some(buf) = &seq0_buffer {
            let (ptr, len) = unsafe { buf.read_buffer() };
            if (ptr as usize) < SRAM_LOWER || (ptr as usize) > SRAM_UPPER {
                return Err((
                    Error::DMABufferNotInDataMemory,
                    self,
                    seq0_buffer,
                    seq1_buffer,
                ));
            }
            if len > MAX_SEQ_LEN {
                return Err((Error::BufferTooLong, self, seq0_buffer, seq1_buffer));
            }
            compiler_fence(Ordering::SeqCst);
            self.pwm.seq0.ptr.write(|w| unsafe { w.bits(ptr as u32) });
            self.pwm.seq0.cnt.write(|w| unsafe { w.bits(len as u32) });
            if start {
                self.start_seq(Seq::Seq0);
            }
        } else {
            self.pwm.seq0.cnt.write(|w| unsafe { w.bits(0) });
        }

        if let Some(buf) = &seq1_buffer {
            let (ptr, len) = unsafe { buf.read_buffer() };
            if (ptr as usize) < SRAM_LOWER || (ptr as usize) > SRAM_UPPER {
                return Err((
                    Error::DMABufferNotInDataMemory,
                    self,
                    seq0_buffer,
                    seq1_buffer,
                ));
            }
            if len > MAX_SEQ_LEN {
                return Err((Error::BufferTooLong, self, seq0_buffer, seq1_buffer));
            }
            compiler_fence(Ordering::SeqCst);
            self.pwm.seq1.ptr.write(|w| unsafe { w.bits(ptr as u32) });
            self.pwm.seq1.cnt.write(|w| unsafe { w.bits(len as u32) });
            if start {
                self.start_seq(Seq::Seq1);
            }
        } else {
            self.pwm.seq1.cnt.write(|w| unsafe { w.bits(0) });
        }

        Ok(PwmSeq {
            inner: Some(Inner {
                seq0_buffer,
                seq1_buffer,
                pwm: self,
            }),
        })
    }

    /// Enables interrupt triggering on the specified event.
    #[inline(always)]
    pub fn enable_interrupt(&self, event: PwmEvent) -> &Self {
        match event {
            PwmEvent::Stopped => self.pwm.intenset.modify(|_r, w| w.stopped().set()),
            PwmEvent::LoopsDone => self.pwm.intenset.modify(|_r, w| w.loopsdone().set()),
            PwmEvent::PwmPeriodEnd => self.pwm.intenset.modify(|_r, w| w.pwmperiodend().set()),
            PwmEvent::SeqStarted(seq) => match seq {
                Seq::Seq0 => self.pwm.intenset.modify(|_r, w| w.seqstarted0().set()),
                Seq::Seq1 => self.pwm.intenset.modify(|_r, w| w.seqstarted1().set()),
            },
            PwmEvent::SeqEnd(seq) => match seq {
                Seq::Seq0 => self.pwm.intenset.modify(|_r, w| w.seqend0().set()),
                Seq::Seq1 => self.pwm.intenset.modify(|_r, w| w.seqend1().set()),
            },
        };
        self
    }

    /// Disables interrupt triggering on the specified event.
    #[inline(always)]
    pub fn disable_interrupt(&self, event: PwmEvent) -> &Self {
        match event {
            PwmEvent::Stopped => self.pwm.intenclr.modify(|_r, w| w.stopped().clear()),
            PwmEvent::LoopsDone => self.pwm.intenclr.modify(|_r, w| w.loopsdone().clear()),
            PwmEvent::PwmPeriodEnd => self.pwm.intenclr.modify(|_r, w| w.pwmperiodend().clear()),
            PwmEvent::SeqStarted(seq) => match seq {
                Seq::Seq0 => self.pwm.intenclr.modify(|_r, w| w.seqstarted0().clear()),
                Seq::Seq1 => self.pwm.intenclr.modify(|_r, w| w.seqstarted1().clear()),
            },
            PwmEvent::SeqEnd(seq) => match seq {
                Seq::Seq0 => self.pwm.intenclr.modify(|_r, w| w.seqend0().clear()),
                Seq::Seq1 => self.pwm.intenclr.modify(|_r, w| w.seqend1().clear()),
            },
        };
        self
    }

    /// Checks if an event has been triggered.
    #[inline(always)]
    pub fn is_event_triggered(&self, event: PwmEvent) -> bool {
        match event {
            PwmEvent::Stopped => self.pwm.events_stopped.read().bits() != 0,
            PwmEvent::LoopsDone => self.pwm.events_loopsdone.read().bits() != 0,
            PwmEvent::PwmPeriodEnd => self.pwm.events_pwmperiodend.read().bits() != 0,
            PwmEvent::SeqStarted(seq) => {
                self.pwm.events_seqstarted[usize::from(seq)].read().bits() != 0
            }
            PwmEvent::SeqEnd(seq) => self.pwm.events_seqend[usize::from(seq)].read().bits() != 0,
        }
    }

    /// Marks event as handled.
    #[inline(always)]
    pub fn reset_event(&self, event: PwmEvent) {
        match event {
            PwmEvent::Stopped => self.pwm.events_stopped.write(|w| w),
            PwmEvent::LoopsDone => self.pwm.events_loopsdone.write(|w| w),
            PwmEvent::PwmPeriodEnd => self.pwm.events_pwmperiodend.write(|w| w),
            PwmEvent::SeqStarted(seq) => self.pwm.events_seqstarted[usize::from(seq)].write(|w| w),
            PwmEvent::SeqEnd(seq) => self.pwm.events_seqend[usize::from(seq)].write(|w| w),
        }
    }

    /// Returns reference to `Stopped` event endpoint for PPI.
    #[inline(always)]
    pub fn event_stopped(&self) -> &EVENTS_STOPPED {
        &self.pwm.events_stopped
    }

    /// Returns reference to `LoopsDone` event endpoint for PPI.
    #[inline(always)]
    pub fn event_loops_done(&self) -> &EVENTS_LOOPSDONE {
        &self.pwm.events_loopsdone
    }

    /// Returns reference to `PwmPeriodEnd` event endpoint for PPI.
    #[inline(always)]
    pub fn event_pwm_period_end(&self) -> &EVENTS_PWMPERIODEND {
        &self.pwm.events_pwmperiodend
    }

    /// Returns reference to `Seq0 End` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq0_end(&self) -> &EVENTS_SEQEND {
        &self.pwm.events_seqend[0]
    }

    /// Returns reference to `Seq1 End` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq1_end(&self) -> &EVENTS_SEQEND {
        &self.pwm.events_seqend[1]
    }

    /// Returns reference to `Seq0 Started` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq0_started(&self) -> &EVENTS_SEQSTARTED {
        &self.pwm.events_seqstarted[0]
    }

    /// Returns reference to `Seq1 Started` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq1_started(&self) -> &EVENTS_SEQSTARTED {
        &self.pwm.events_seqstarted[1]
    }

    /// Returns reference to `Seq0 Start` task endpoint for PPI.
    #[inline(always)]
    pub fn task_start_seq0(&self) -> &TASKS_SEQSTART {
        &self.pwm.tasks_seqstart[0]
    }

    /// Returns reference to `Seq1 Started` task endpoint for PPI.
    #[inline(always)]
    pub fn task_start_seq1(&self) -> &TASKS_SEQSTART {
        &self.pwm.tasks_seqstart[1]
    }

    /// Returns reference to `NextStep` task endpoint for PPI.
    #[inline(always)]
    pub fn task_next_step(&self) -> &TASKS_NEXTSTEP {
        &self.pwm.tasks_nextstep
    }

    /// Returns reference to `Stop` task endpoint for PPI.
    #[inline(always)]
    pub fn task_stop(&self) -> &TASKS_STOP {
        &self.pwm.tasks_stop
    }

    /// Returns individual handles to the four PWM channels.
    #[inline(always)]
    pub fn split_channels(&self) -> (PwmChannel<T>, PwmChannel<T>, PwmChannel<T>, PwmChannel<T>) {
        (
            PwmChannel::new(self, Channel::C0),
            PwmChannel::new(self, Channel::C1),
            PwmChannel::new(self, Channel::C2),
            PwmChannel::new(self, Channel::C3),
        )
    }

    /// Returns individual handles to the two PWM groups.
    pub fn split_groups(&self) -> (PwmGroup<T>, PwmGroup<T>) {
        (
            PwmGroup::new(self, Group::G0),
            PwmGroup::new(self, Group::G1),
        )
    }

    /// Consumes `self` and returns back the raw peripheral.
    pub fn free(self) -> (T, Pins) {
        let ch0 = self.pwm.psel.out[0].read();
        let ch1 = self.pwm.psel.out[1].read();
        let ch2 = self.pwm.psel.out[2].read();
        let ch3 = self.pwm.psel.out[3].read();
        self.pwm.psel.out[0].reset();
        self.pwm.psel.out[1].reset();
        self.pwm.psel.out[2].reset();
        self.pwm.psel.out[3].reset();
        (
            self.pwm,
            Pins {
                ch0: if ch0.connect().is_connected() {
                    Some(unsafe { Pin::from_psel_bits(ch0.bits()) })
                } else {
                    None
                },
                ch1: if ch1.connect().is_connected() {
                    Some(unsafe { Pin::from_psel_bits(ch1.bits()) })
                } else {
                    None
                },
                ch2: if ch2.connect().is_connected() {
                    Some(unsafe { Pin::from_psel_bits(ch2.bits()) })
                } else {
                    None
                },
                ch3: if ch3.connect().is_connected() {
                    Some(unsafe { Pin::from_psel_bits(ch3.bits()) })
                } else {
                    None
                },
            },
        )
    }
}

/// Pins for the Pwm
pub struct Pins {
    /// Channel 0 pin, `None` if it was unused
    pub ch0: Option<Pin<Output<PushPull>>>,
    /// Channel 1 pin, `None` if it was unused
    pub ch1: Option<Pin<Output<PushPull>>>,
    /// Channel 2 pin, `None` if it was unused
    pub ch2: Option<Pin<Output<PushPull>>>,
    /// Channel 3 pin, `None` if it was unused
    pub ch3: Option<Pin<Output<PushPull>>>,
}

/// A Pwm sequence wrapper
#[derive(Debug)]
pub struct PwmSeq<T: Instance, B0, B1> {
    inner: Option<Inner<T, B0, B1>>,
}

#[derive(Debug)]
struct Inner<T: Instance, B0, B1> {
    seq0_buffer: Option<B0>,
    seq1_buffer: Option<B1>,
    pwm: Pwm<T>,
}

impl<T: Instance, B0, B1> PwmSeq<T, B0, B1>
where
    B0: ReadBuffer<Word = u16> + 'static,
    B1: ReadBuffer<Word = u16> + 'static,
{
    /// Returns the wrapped contents.
    #[inline(always)]
    pub fn split(mut self) -> (Option<B0>, Option<B1>, Pwm<T>) {
        compiler_fence(Ordering::SeqCst);
        let inner = self
            .inner
            .take()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        (inner.seq0_buffer, inner.seq1_buffer, inner.pwm)
    }

    /// Stops PWM generation.
    #[inline(always)]
    pub fn stop(&self) {
        let inner = self
            .inner
            .as_ref()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        inner.pwm.stop();
    }

    /// Starts playing the given sequence.
    #[inline(always)]
    pub fn start_seq(&self, seq: Seq) {
        let inner = self
            .inner
            .as_ref()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        inner.pwm.start_seq(seq);
    }

    /// Checks if the given event has been triggered.
    #[inline(always)]
    pub fn is_event_triggered(&self, event: PwmEvent) -> bool {
        let inner = self
            .inner
            .as_ref()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        inner.pwm.is_event_triggered(event)
    }

    /// Marks the given event as handled.
    #[inline(always)]
    pub fn reset_event(&self, event: PwmEvent) {
        let inner = self
            .inner
            .as_ref()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        inner.pwm.reset_event(event)
    }
}

impl<T: Instance> embedded_hal::Pwm for Pwm<T> {
    type Channel = Channel;
    type Duty = u16;
    type Time = Hertz;

    fn enable(&mut self, channel: Self::Channel) {
        self.enable_channel(channel);
    }

    fn disable(&mut self, channel: Self::Channel) {
        self.disable_channel(channel);
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        self.duty_on(channel)
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        self.set_duty_on(channel, duty);
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.max_duty()
    }

    fn get_period(&self) -> Self::Time {
        self.period()
    }

    fn set_period<P>(&mut self, period: P)
    where
        P: Into<Self::Time>,
    {
        Self::set_period(self, period.into());
    }
}

/// PWM channel
#[derive(Debug)]
pub struct PwmChannel<'a, T: Instance> {
    pwm: &'a Pwm<T>,
    channel: Channel,
}

impl<'a, T: Instance> PwmChannel<'a, T> {
    pub fn new(pwm: &'a Pwm<T>, channel: Channel) -> Self {
        Self { pwm, channel }
    }

    pub fn enable(&self) {
        self.pwm.enable_channel(self.channel);
    }

    pub fn disable(&self) {
        self.pwm.disable_channel(self.channel);
    }

    pub fn max_duty(&self) -> u16 {
        self.pwm.max_duty()
    }
    pub fn set_duty(&self, duty: u16) {
        self.pwm.set_duty_on(self.channel, duty);
    }
    pub fn set_duty_on(&self, duty: u16) {
        self.pwm.set_duty_on(self.channel, duty);
    }
    pub fn set_duty_off(&self, duty: u16) {
        self.pwm.set_duty_off(self.channel, duty);
    }
    pub fn duty_on(&self) -> u16 {
        self.pwm.duty_on(self.channel)
    }
    pub fn duty_off(&self) -> u16 {
        self.pwm.duty_off(self.channel)
    }
}

impl<'a, T: Instance> embedded_hal::PwmPin for PwmChannel<'a, T> {
    type Duty = u16;

    fn disable(&mut self) {
        Self::disable(self);
    }

    fn enable(&mut self) {
        Self::enable(self);
    }

    fn get_duty(&self) -> Self::Duty {
        self.duty_on()
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.max_duty()
    }

    fn set_duty(&mut self, duty: u16) {
        self.set_duty_on(duty)
    }
}

/// PWM group
#[derive(Debug)]
pub struct PwmGroup<'a, T: Instance> {
    pwm: &'a Pwm<T>,
    group: Group,
}

impl<'a, T: Instance> PwmGroup<'a, T> {
    pub fn new(pwm: &'a Pwm<T>, group: Group) -> Self {
        Self { pwm, group }
    }

    pub fn enable(&self) {
        self.pwm.enable_group(self.group);
    }

    pub fn disable(&self) {
        self.pwm.disable_group(self.group);
    }
    pub fn max_duty(&self) -> u16 {
        self.pwm.max_duty()
    }
    pub fn set_duty(&self, duty: u16) {
        self.pwm.set_duty_on_group(self.group, duty);
    }
    pub fn set_duty_on(&self, duty: u16) {
        self.pwm.set_duty_on_group(self.group, duty);
    }
    pub fn set_duty_off(&self, duty: u16) {
        self.pwm.set_duty_off_group(self.group, duty);
    }
    pub fn duty_on(&self) -> u16 {
        self.pwm.duty_on_group(self.group)
    }
    pub fn duty_off(&self) -> u16 {
        self.pwm.duty_off_group(self.group)
    }
}

impl<'a, T: Instance> embedded_hal::PwmPin for PwmGroup<'a, T> {
    type Duty = u16;

    fn disable(&mut self) {
        Self::disable(self);
    }

    fn enable(&mut self) {
        Self::enable(self);
    }

    fn get_duty(&self) -> Self::Duty {
        self.duty_on()
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.max_duty()
    }

    fn set_duty(&mut self, duty: u16) {
        self.set_duty_on(duty)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Channel {
    C0,
    C1,
    C2,
    C3,
}
impl From<Channel> for usize {
    fn from(variant: Channel) -> Self {
        variant as _
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Group {
    G0,
    G1,
}
impl From<Group> for usize {
    fn from(variant: Group) -> Self {
        variant as _
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum LoadMode {
    Common,
    Grouped,
    Individual,
    Waveform,
}
impl From<LoadMode> for u8 {
    fn from(variant: LoadMode) -> Self {
        variant as _
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Seq {
    Seq0,
    Seq1,
}
impl From<Seq> for usize {
    fn from(variant: Seq) -> Self {
        variant as _
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Prescaler {
    Div1,
    Div2,
    Div4,
    Div8,
    Div16,
    Div32,
    Div64,
    Div128,
}
impl From<Prescaler> for u8 {
    fn from(variant: Prescaler) -> Self {
        variant as _
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CounterMode {
    Up,
    UpAndDown,
}
impl From<CounterMode> for bool {
    fn from(variant: CounterMode) -> Self {
        match variant {
            CounterMode::Up => false,
            CounterMode::UpAndDown => true,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Loop {
    Disabled,
    Times(u16),
    Inf,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PwmEvent {
    Stopped,
    LoopsDone,
    PwmPeriodEnd,
    SeqStarted(Seq),
    SeqEnd(Seq),
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum StepMode {
    Auto,
    NextStep,
}
impl From<StepMode> for bool {
    fn from(variant: StepMode) -> Self {
        match variant {
            StepMode::Auto => false,
            StepMode::NextStep => true,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    DMABufferNotInDataMemory,
    BufferTooLong,
}

pub trait Instance: sealed::Sealed + Deref<Target = RegisterBlock> {
    const INTERRUPT: Interrupt;

    /// Provides access to the associated internal duty buffer for the instance.
    fn buffer() -> &'static Cell<[u16; 4]>;
}

// Internal static duty buffers. One per instance.
static mut BUF0: Cell<[u16; 4]> = Cell::new([0; 4]);
#[cfg(not(any(feature = "52810", feature = "52811")))]
static mut BUF1: Cell<[u16; 4]> = Cell::new([0; 4]);
#[cfg(not(any(feature = "52810", feature = "52811")))]
static mut BUF2: Cell<[u16; 4]> = Cell::new([0; 4]);
#[cfg(not(any(feature = "52810", feature = "52811", feature = "52832")))]
static mut BUF3: Cell<[u16; 4]> = Cell::new([0; 4]);

#[cfg(not(any(feature = "9160", feature = "5340-app")))]
impl Instance for crate::pac::PWM0 {
    const INTERRUPT: Interrupt = Interrupt::PWM0;
    #[inline(always)]
    fn buffer() -> &'static Cell<[u16; 4]> {
        unsafe { &BUF0 }
    }
}

#[cfg(not(any(
    feature = "52810",
    feature = "52811",
    feature = "9160",
    feature = "5340-app"
)))]
impl Instance for crate::pac::PWM1 {
    const INTERRUPT: Interrupt = Interrupt::PWM1;
    fn buffer() -> &'static Cell<[u16; 4]> {
        unsafe { &BUF1 }
    }
}

#[cfg(not(any(
    feature = "52810",
    feature = "52811",
    feature = "9160",
    feature = "5340-app"
)))]
impl Instance for crate::pac::PWM2 {
    const INTERRUPT: Interrupt = Interrupt::PWM2;
    fn buffer() -> &'static Cell<[u16; 4]> {
        unsafe { &BUF2 }
    }
}

#[cfg(not(any(
    feature = "52810",
    feature = "52811",
    feature = "52832",
    feature = "5340-app",
    feature = "9160"
)))]
impl Instance for crate::pac::PWM3 {
    const INTERRUPT: Interrupt = Interrupt::PWM3;
    fn buffer() -> &'static Cell<[u16; 4]> {
        unsafe { &BUF3 }
    }
}

#[cfg(any(feature = "9160", feature = "5340-app"))]
impl Instance for crate::pac::PWM0_NS {
    const INTERRUPT: Interrupt = Interrupt::PWM0;
    #[inline(always)]
    fn buffer() -> &'static Cell<[u16; 4]> {
        unsafe { &BUF0 }
    }
}

#[cfg(any(feature = "9160", feature = "5340-app"))]
impl Instance for crate::pac::PWM1_NS {
    const INTERRUPT: Interrupt = Interrupt::PWM1;
    fn buffer() -> &'static Cell<[u16; 4]> {
        unsafe { &BUF1 }
    }
}

#[cfg(any(feature = "9160", feature = "5340-app"))]
impl Instance for crate::pac::PWM2_NS {
    const INTERRUPT: Interrupt = Interrupt::PWM2;
    fn buffer() -> &'static Cell<[u16; 4]> {
        unsafe { &BUF2 }
    }
}

#[cfg(any(feature = "9160", feature = "5340-app"))]
impl Instance for crate::pac::PWM3_NS {
    const INTERRUPT: Interrupt = Interrupt::PWM3;
    fn buffer() -> &'static Cell<[u16; 4]> {
        unsafe { &BUF3 }
    }
}
mod sealed {
    pub trait Sealed {}

    #[cfg(not(any(feature = "5340-app", feature = "9160")))]
    impl Sealed for crate::pac::PWM0 {}

    #[cfg(not(any(
        feature = "52810",
        feature = "52811",
        feature = "5340-app",
        feature = "9160"
    )))]
    impl Sealed for crate::pac::PWM1 {}

    #[cfg(not(any(
        feature = "52810",
        feature = "52811",
        feature = "5340-app",
        feature = "9160"
    )))]
    impl Sealed for crate::pac::PWM2 {}

    #[cfg(not(any(
        feature = "52810",
        feature = "52811",
        feature = "52832",
        feature = "5340-app",
        feature = "9160"
    )))]
    impl Sealed for crate::pac::PWM3 {}

    #[cfg(any(feature = "9160", feature = "5340-app"))]
    impl Sealed for crate::pac::PWM0_NS {}

    #[cfg(any(feature = "9160", feature = "5340-app"))]
    impl Sealed for crate::pac::PWM1_NS {}

    #[cfg(any(feature = "9160", feature = "5340-app"))]
    impl Sealed for crate::pac::PWM2_NS {}

    #[cfg(any(feature = "9160", feature = "5340-app"))]
    impl Sealed for crate::pac::PWM3_NS {}
}
