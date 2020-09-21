//! HAL interface to the PWM peripheral.
//!
//! The pulse with modulation (PWM) module enables the generation of pulse width modulated signals on GPIO.

use core::cell::RefCell;
use core::sync::atomic::{compiler_fence, Ordering};

#[cfg(any(feature = "52833", feature = "52840"))]
use crate::{
    gpio::{Output, Pin, Port, PushPull},
    pac::{
        generic::Reg,
        pwm0::{
            _EVENTS_LOOPSDONE, _EVENTS_PWMPERIODEND, _EVENTS_SEQEND, _EVENTS_SEQSTARTED,
            _EVENTS_STOPPED, _TASKS_NEXTSTEP, _TASKS_SEQSTART, _TASKS_STOP,
        },
        PWM0, PWM1, PWM2, PWM3,
    },
    target_constants::{SRAM_LOWER, SRAM_UPPER},
    time::*,
};

/// A safe wrapper around the raw peripheral.
#[derive(Debug)]
pub struct Pwm<T: Instance> {
    pwm: T,
    duty: RefCell<[u16; 4]>,
}

impl<T> Pwm<T>
where
    T: Instance,
{
    /// Takes ownership of the peripheral and applies sane defaults.
    pub fn new(pwm: T) -> Pwm<T> {
        compiler_fence(Ordering::SeqCst);
        let duty = RefCell::new([0u16; 4]);
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

        pwm.seq0
            .ptr
            .write(|w| unsafe { w.bits(duty.as_ptr() as u32) });
        pwm.seq0.cnt.write(|w| unsafe { w.bits(4) });
        pwm.seq1
            .ptr
            .write(|w| unsafe { w.bits(duty.as_ptr() as u32) });
        pwm.seq1.cnt.write(|w| unsafe { w.bits(4) });

        Self { pwm, duty }
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

    /// Sets the associated output pin for the PWM channel and enables it.
    #[inline(always)]
    pub fn set_output_pin(&self, channel: Channel, pin: &Pin<Output<PushPull>>) -> &Self {
        self.pwm.psel.out[usize::from(channel)].write(|w| {
            #[cfg(any(feature = "52833", feature = "52840"))]
            match pin.port() {
                Port::Port0 => w.port().clear_bit(),
                Port::Port1 => w.port().set_bit(),
            };
            unsafe {
                w.pin().bits(pin.pin());
            }
            w.connect().connected()
        });
        self
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
        let val = self.duty.borrow()[index];
        let is_inverted = (val >> 15) & 1 == 0;
        match is_inverted {
            false => val,
            true => self.max_duty() - (val & 0x7FFF),
        }
    }

    // Internal helper function that returns 15 bit inverted duty cycle value.
    #[inline(always)]
    fn duty_off_value(&self, index: usize) -> u16 {
        let val = self.duty.borrow()[index];
        let is_inverted = (val >> 15) & 1 == 0;
        match is_inverted {
            false => self.max_duty() - val,
            true => val & 0x7FFF,
        }
    }

    /// Sets duty cycle (15 bit) for all PWM channels.
    /// Will replace any ongoing sequence playback.
    pub fn set_duty_on_common(&self, duty: u16) {
        compiler_fence(Ordering::SeqCst);
        self.duty
            .borrow_mut()
            .copy_from_slice(&[duty.min(self.max_duty()) & 0x7FFF; 4][..]);
        self.one_shot();
        self.set_load_mode(LoadMode::Common);
        self.pwm
            .seq0
            .ptr
            .write(|w| unsafe { w.bits(self.duty.as_ptr() as u32) });
        self.pwm.seq0.cnt.write(|w| unsafe { w.bits(1) });
        self.start_seq(Seq::Seq0);
    }

    /// Sets inverted duty cycle (15 bit) for all PWM channels.
    /// Will replace any ongoing sequence playback.
    pub fn set_duty_off_common(&self, duty: u16) {
        compiler_fence(Ordering::SeqCst);
        self.duty
            .borrow_mut()
            .copy_from_slice(&[duty.min(self.max_duty()) | 0x8000; 4][..]);
        self.one_shot();
        self.set_load_mode(LoadMode::Common);
        self.pwm
            .seq0
            .ptr
            .write(|w| unsafe { w.bits(self.duty.as_ptr() as u32) });
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
        compiler_fence(Ordering::SeqCst);
        self.duty.borrow_mut()[usize::from(group)] = duty.min(self.max_duty()) & 0x7FFF;
        self.one_shot();
        self.set_load_mode(LoadMode::Grouped);
        self.pwm
            .seq0
            .ptr
            .write(|w| unsafe { w.bits(self.duty.as_ptr() as u32) });
        self.pwm.seq0.cnt.write(|w| unsafe { w.bits(2) });
        self.start_seq(Seq::Seq0);
    }

    /// Sets inverted duty cycle (15 bit) for a PWM group.
    /// Will replace any ongoing sequence playback.
    pub fn set_duty_off_group(&self, group: Group, duty: u16) {
        compiler_fence(Ordering::SeqCst);
        self.duty.borrow_mut()[usize::from(group)] = duty.min(self.max_duty()) | 0x8000;
        self.one_shot();
        self.set_load_mode(LoadMode::Grouped);
        self.pwm
            .seq0
            .ptr
            .write(|w| unsafe { w.bits(self.duty.as_ptr() as u32) });
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
        compiler_fence(Ordering::SeqCst);
        self.duty.borrow_mut()[usize::from(channel)] = duty.min(self.max_duty()) & 0x7FFF;
        self.one_shot();
        self.set_load_mode(LoadMode::Individual);
        if self.load_seq(Seq::Seq0, &*self.duty.borrow()).is_ok() {
            self.start_seq(Seq::Seq0);
        }
    }

    /// Sets inverted duty cycle (15 bit) for a PWM channel.
    /// Will replace any ongoing sequence playback and the other channels will return to their previously set value.
    pub fn set_duty_off(&self, channel: Channel, duty: u16) {
        compiler_fence(Ordering::SeqCst);
        self.duty.borrow_mut()[usize::from(channel)] = duty.min(self.max_duty()) | 0x8000;
        self.one_shot();
        self.set_load_mode(LoadMode::Individual);
        if self.load_seq(Seq::Seq0, &*self.duty.borrow()).is_ok() {
            self.start_seq(Seq::Seq0);
        }
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

    /// Loops playback of sequences indefinately.
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

    /// Loads a sequence buffer.
    /// NOTE: `buf` must live until the sequence is done playing, or it might play a corrupted sequence.
    pub fn load_seq(&self, seq: Seq, buf: &[u16]) -> Result<(), Error> {
        if (buf.as_ptr() as usize) < SRAM_LOWER || (buf.as_ptr() as usize) > SRAM_UPPER {
            return Err(Error::DMABufferNotInDataMemory);
        }

        if buf.len() > 32_768 {
            return Err(Error::BufferTooLong);
        }

        compiler_fence(Ordering::SeqCst);

        match seq {
            Seq::Seq0 => {
                self.pwm
                    .seq0
                    .ptr
                    .write(|w| unsafe { w.bits(buf.as_ptr() as u32) });
                self.pwm
                    .seq0
                    .cnt
                    .write(|w| unsafe { w.bits(buf.len() as u32) });
            }
            Seq::Seq1 => {
                self.pwm
                    .seq1
                    .ptr
                    .write(|w| unsafe { w.bits(buf.as_ptr() as u32) });
                self.pwm
                    .seq1
                    .cnt
                    .write(|w| unsafe { w.bits(buf.len() as u32) });
            }
        }
        Ok(())
    }

    /// Loads the first PWM value on all enabled channels from a sequence and starts playing that sequence.
    /// Causes PWM generation to start if not running.
    #[inline(always)]
    pub fn start_seq(&self, seq: Seq) {
        compiler_fence(Ordering::SeqCst);
        self.pwm.tasks_seqstart[usize::from(seq)].write(|w| w.tasks_seqstart().set_bit());
        while self.pwm.events_seqstarted[usize::from(seq)].read().bits() == 0 {}
        self.pwm.events_seqend[0].write(|w| w);
        self.pwm.events_seqend[1].write(|w| w);
    }

    /// Steps by one value in the current sequence on all enabled channels, if the `NextStep` step mode is selected.
    /// Does not cause PWM generation to start if not running.
    #[inline(always)]
    pub fn next_step(&self) {
        self.pwm
            .tasks_nextstep
            .write(|w| w.tasks_nextstep().set_bit());
    }

    /// Stops PWM pulse generation on all channels at the end of current PWM period, and stops sequence playback.
    #[inline(always)]
    pub fn stop(&self) {
        compiler_fence(Ordering::SeqCst);
        self.pwm.tasks_stop.write(|w| w.tasks_stop().set_bit());
        while self.pwm.events_stopped.read().bits() == 0 {}
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
    pub fn event_stopped(&self) -> &Reg<u32, _EVENTS_STOPPED> {
        &self.pwm.events_stopped
    }

    /// Returns reference to `LoopsDone` event endpoint for PPI.
    #[inline(always)]
    pub fn event_loops_done(&self) -> &Reg<u32, _EVENTS_LOOPSDONE> {
        &self.pwm.events_loopsdone
    }

    /// Returns reference to `PwmPeriodEnd` event endpoint for PPI.
    #[inline(always)]
    pub fn event_pwm_period_end(&self) -> &Reg<u32, _EVENTS_PWMPERIODEND> {
        &self.pwm.events_pwmperiodend
    }

    /// Returns reference to `Seq0 End` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq0_end(&self) -> &Reg<u32, _EVENTS_SEQEND> {
        &self.pwm.events_seqend[0]
    }

    /// Returns reference to `Seq1 End` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq1_end(&self) -> &Reg<u32, _EVENTS_SEQEND> {
        &self.pwm.events_seqend[1]
    }

    /// Returns reference to `Seq0 Started` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq0_started(&self) -> &Reg<u32, _EVENTS_SEQSTARTED> {
        &self.pwm.events_seqstarted[0]
    }

    /// Returns reference to `Seq1 Started` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq1_started(&self) -> &Reg<u32, _EVENTS_SEQSTARTED> {
        &self.pwm.events_seqstarted[1]
    }

    /// Returns reference to `Seq0 Start` task endpoint for PPI.
    #[cfg(any(feature = "52833", feature = "52840"))]
    #[inline(always)]
    pub fn task_start_seq0(&self) -> &Reg<u32, _TASKS_SEQSTART> {
        &self.pwm.tasks_seqstart[0]
    }

    /// Returns reference to `Seq1 Started` task endpoint for PPI.
    #[cfg(any(feature = "52833", feature = "52840"))]
    #[inline(always)]
    pub fn task_start_seq1(&self) -> &Reg<u32, _TASKS_SEQSTART> {
        &self.pwm.tasks_seqstart[1]
    }

    /// Returns reference to `NextStep` task endpoint for PPI.
    #[inline(always)]
    pub fn task_next_step(&self) -> &Reg<u32, _TASKS_NEXTSTEP> {
        &self.pwm.tasks_nextstep
    }

    /// Returns reference to `Stop` task endpoint for PPI.
    #[inline(always)]
    pub fn task_stop(&self) -> &Reg<u32, _TASKS_STOP> {
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
    pub fn free(self) -> T {
        self.pwm
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

pub trait Instance: private::Sealed {}

impl Instance for PWM0 {}

#[cfg(not(any(feature = "52810", feature = "52811")))]
impl Instance for PWM1 {}

#[cfg(not(any(feature = "52810", feature = "52811")))]
impl Instance for PWM2 {}

#[cfg(not(any(feature = "52810", feature = "52811", feature = "52832")))]
impl Instance for PWM3 {}

mod private {
    pub trait Sealed: core::ops::Deref<Target = crate::pac::pwm0::RegisterBlock> {}

    impl Sealed for crate::pwm::PWM0 {}

    #[cfg(not(any(feature = "52810", feature = "52811")))]
    impl Sealed for crate::pwm::PWM1 {}

    #[cfg(not(any(feature = "52810", feature = "52811")))]
    impl Sealed for crate::pwm::PWM2 {}

    #[cfg(not(any(feature = "52810", feature = "52811", feature = "52832")))]
    impl Sealed for crate::pwm::PWM3 {}
}
