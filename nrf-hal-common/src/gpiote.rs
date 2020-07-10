#[cfg(feature = "51")]
use crate::target::{gpio, GPIO as P0};

#[cfg(not(feature = "51"))]
use crate::target::{p0 as gpio, P0};

#[cfg(any(feature = "52833", feature = "52840"))]
use crate::target::P1;

use {
    crate::gpio::{
        Floating, Input, Level, OpenDrain, Output, Pin, Port, PullDown, PullUp, PushPull,
    },
    crate::target::gpiote::{_EVENTS_IN, _EVENTS_PORT, _TASKS_OUT},
    crate::target::{generic::Reg, GPIOTE},
};

#[cfg(not(feature = "51"))]
use crate::target::gpiote::{_TASKS_CLR, _TASKS_SET};

#[cfg(not(feature = "51"))]
const NUM_CHANNELS: usize = 8;
#[cfg(feature = "51")]
const NUM_CHANNELS: usize = 4;

pub struct Gpiote {
    gpiote: GPIOTE,
}

impl Gpiote {
    pub fn new(gpiote: GPIOTE) -> Self {
        Self { gpiote }
    }

    fn channel(&self, channel: usize) -> GpioteChannel {
        GpioteChannel {
            gpiote: &self.gpiote,
            channel,
        }
    }
    pub fn channel0(&self) -> GpioteChannel {
        self.channel(0)
    }
    pub fn channel1(&self) -> GpioteChannel {
        self.channel(1)
    }
    pub fn channel2(&self) -> GpioteChannel {
        self.channel(2)
    }
    pub fn channel3(&self) -> GpioteChannel {
        self.channel(3)
    }
    #[cfg(not(feature = "51"))]
    pub fn channel4(&self) -> GpioteChannel {
        self.channel(4)
    }
    #[cfg(not(feature = "51"))]
    pub fn channel5(&self) -> GpioteChannel {
        self.channel(5)
    }
    #[cfg(not(feature = "51"))]
    pub fn channel6(&self) -> GpioteChannel {
        self.channel(6)
    }
    #[cfg(not(feature = "51"))]
    pub fn channel7(&self) -> GpioteChannel {
        self.channel(7)
    }

    pub fn port(&self) -> GpiotePort {
        GpiotePort {
            gpiote: &self.gpiote,
        }
    }

    pub fn reset_events(&self) {
        // Mark all events as handled
        (0..NUM_CHANNELS).for_each(|ch| self.gpiote.events_in[ch].write(|w| w));
        self.gpiote.events_port.write(|w| w);
    }

    pub fn free(self) -> GPIOTE {
        self.gpiote
    }
}

pub struct GpioteChannel<'a> {
    gpiote: &'a GPIOTE,
    channel: usize,
}

impl<'a> GpioteChannel<'_> {
    pub fn input_pin<P: GpioteInputPin>(&'a self, pin: &'a P) -> GpioteChannelEvent<'a, P> {
        GpioteChannelEvent {
            gpiote: &self.gpiote,
            pin: pin,
            channel: self.channel,
        }
    }

    pub fn output_pin<P: GpioteOutputPin>(&'a self, pin: P) -> GpioteTask<'a, P> {
        GpioteTask {
            gpiote: &self.gpiote,
            pin: pin,
            channel: self.channel,
            task_out_polarity: TaskOutPolarity::Toggle,
        }
    }

    pub fn is_event_triggered(&self) -> bool {
        self.gpiote.events_in[self.channel].read().bits() != 0
    }

    pub fn reset_events(&self) {
        self.gpiote.events_in[self.channel].write(|w| w);
    }

    pub fn out(&self) {
        self.gpiote.tasks_out[self.channel].write(|w| unsafe { w.bits(1) });
    }

    #[cfg(not(feature = "51"))]
    pub fn set(&self) {
        self.gpiote.tasks_set[self.channel].write(|w| unsafe { w.bits(1) });
    }

    #[cfg(not(feature = "51"))]
    pub fn clear(&self) {
        self.gpiote.tasks_clr[self.channel].write(|w| unsafe { w.bits(1) });
    }

    pub fn event(&self) -> &Reg<u32, _EVENTS_IN> {
        // Return reference to event for PPI
        &self.gpiote.events_in[self.channel]
    }

    pub fn task_out(&self) -> &Reg<u32, _TASKS_OUT> {
        // Return reference to task_out for PPI
        &self.gpiote.tasks_out[self.channel]
    }

    #[cfg(not(feature = "51"))]
    pub fn task_clr(&self) -> &Reg<u32, _TASKS_CLR> {
        // Return reference to task_clr for PPI
        &self.gpiote.tasks_clr[self.channel]
    }

    #[cfg(not(feature = "51"))]
    pub fn task_set(&self) -> &Reg<u32, _TASKS_SET> {
        // Return reference to task_set for PPI
        &self.gpiote.tasks_set[self.channel]
    }
}

pub struct GpiotePort<'a> {
    gpiote: &'a GPIOTE,
}

impl<'a> GpiotePort<'_> {
    pub fn input_pin<P: GpioteInputPin>(&'a self, pin: &'a P) -> GpiotePortEvent<'a, P> {
        GpiotePortEvent { pin }
    }
    pub fn enable_interrupt(&self) {
        // Enable port interrupt
        self.gpiote.intenset.write(|w| w.port().set());
    }
    pub fn disable_interrupt(&self) {
        // Disable port interrupt
        self.gpiote.intenclr.write(|w| w.port().set_bit());
    }
    pub fn is_event_triggered(&self) -> bool {
        self.gpiote.events_port.read().bits() != 0
    }
    pub fn reset_events(&self) {
        // Mark port events as handled
        self.gpiote.events_port.write(|w| w);
    }
    pub fn event(&self) -> &Reg<u32, _EVENTS_PORT> {
        // Return reference to event for PPI
        &self.gpiote.events_port
    }
}

pub struct GpioteChannelEvent<'a, P: GpioteInputPin> {
    gpiote: &'a GPIOTE,
    pin: &'a P,
    channel: usize,
}

impl<'a, P: GpioteInputPin> GpioteChannelEvent<'_, P> {
    pub fn hi_to_lo(&self) -> &Self {
        config_channel_event_pin(self.gpiote, self.channel, self.pin, EventPolarity::HiToLo);
        self
    }

    pub fn lo_to_hi(&self) -> &Self {
        config_channel_event_pin(self.gpiote, self.channel, self.pin, EventPolarity::LoToHi);
        self
    }

    pub fn toggle(&self) -> &Self {
        config_channel_event_pin(self.gpiote, self.channel, self.pin, EventPolarity::Toggle);
        self
    }

    pub fn none(&self) -> &Self {
        config_channel_event_pin(self.gpiote, self.channel, self.pin, EventPolarity::None);
        self
    }

    pub fn enable_interrupt(&self) -> &Self {
        // Enable interrupt for pin
        unsafe {
            self.gpiote
                .intenset
                .modify(|r, w| w.bits(r.bits() | self.pin.pin() as u32))
        }
        self
    }

    pub fn disable_interrupt(&self) -> &Self {
        // Disable interrupt for pin
        unsafe {
            self.gpiote
                .intenclr
                .write(|w| w.bits(self.pin.pin() as u32))
        }
        self
    }
}

fn config_channel_event_pin<P: GpioteInputPin>(
    gpiote: &GPIOTE,
    channel: usize,
    pin: &P,
    trigger_mode: EventPolarity,
) {
    // Config pin as event-triggering input for specified edge transition trigger mode
    gpiote.config[channel].write(|w| {
        match trigger_mode {
            EventPolarity::HiToLo => w.mode().event().polarity().hi_to_lo(),
            EventPolarity::LoToHi => w.mode().event().polarity().lo_to_hi(),
            EventPolarity::None => w.mode().event().polarity().none(),
            EventPolarity::Toggle => w.mode().event().polarity().toggle(),
        };
        unsafe { w.psel().bits(pin.pin()) }
    });
}

pub struct GpiotePortEvent<'a, P: GpioteInputPin> {
    pin: &'a P,
}

impl<'a, P: GpioteInputPin> GpiotePortEvent<'_, P> {
    pub fn low(&self) {
        config_port_event_pin(self.pin, PortEventSense::Low);
    }
    pub fn high(&self) {
        config_port_event_pin(self.pin, PortEventSense::High);
    }
    pub fn disabled(&self) {
        config_port_event_pin(self.pin, PortEventSense::Disabled);
    }
}

fn config_port_event_pin<P: GpioteInputPin>(pin: &P, sense: PortEventSense) {
    // Set pin sense to specified mode to trigger port events
    unsafe {
        &(*{
            match pin.port() {
                Port::Port0 => P0::ptr(),
                #[cfg(any(feature = "52833", feature = "52840"))]
                Port::Port1 => P1::ptr(),
            }
        })
        .pin_cnf[pin.pin() as usize]
    }
    .modify(|_r, w| match sense {
        PortEventSense::Disabled => w.sense().disabled(),
        PortEventSense::High => w.sense().high(),
        PortEventSense::Low => w.sense().low(),
    });
}

pub struct GpioteTask<'a, P: GpioteOutputPin> {
    gpiote: &'a GPIOTE,
    pin: P,
    channel: usize,
    task_out_polarity: TaskOutPolarity,
}

impl<'a, P: GpioteOutputPin> GpioteTask<'_, P> {
    pub fn init_high(&self) {
        config_channel_task_pin(
            self.gpiote,
            self.channel,
            &self.pin,
            &self.task_out_polarity,
            Level::High,
        );
    }

    pub fn init_low(&self) {
        config_channel_task_pin(
            self.gpiote,
            self.channel,
            &self.pin,
            &self.task_out_polarity,
            Level::Low,
        );
    }

    pub fn task_out_polarity(&mut self, polarity: TaskOutPolarity) -> &mut Self {
        self.task_out_polarity = polarity;
        self
    }
}

fn config_channel_task_pin<P: GpioteOutputPin>(
    gpiote: &GPIOTE,
    channel: usize,
    pin: &P,
    task_out_polarity: &TaskOutPolarity,
    init_out: Level,
) {
    // Config pin as task output with specified initial state and task out polarity
    gpiote.config[channel].write(|w| {
        match init_out {
            Level::High => w.mode().task().outinit().high(),
            Level::Low => w.mode().task().outinit().low(),
        };
        match task_out_polarity {
            TaskOutPolarity::Set => w.polarity().lo_to_hi(),
            TaskOutPolarity::Clear => w.polarity().hi_to_lo(),
            TaskOutPolarity::Toggle => w.polarity().toggle(),
        };
        unsafe { w.psel().bits(pin.pin()) }
    });
}

pub enum TaskOutPolarity {
    Set,
    Clear,
    Toggle,
}

pub enum EventPolarity {
    None,
    HiToLo,
    LoToHi,
    Toggle,
}

pub enum PortEventSense {
    Disabled,
    High,
    Low,
}

pub trait GpioteInputPin {
    fn pin(&self) -> u8;
    fn port(&self) -> Port;
}

impl GpioteInputPin for Pin<Input<PullUp>> {
    fn pin(&self) -> u8 {
        self.pin()
    }
    fn port(&self) -> Port {
        self.port()
    }
}

impl GpioteInputPin for Pin<Input<PullDown>> {
    fn pin(&self) -> u8 {
        self.pin()
    }
    fn port(&self) -> Port {
        self.port()
    }
}

impl GpioteInputPin for Pin<Input<Floating>> {
    fn pin(&self) -> u8 {
        self.pin()
    }
    fn port(&self) -> Port {
        self.port()
    }
}

pub trait GpioteOutputPin {
    fn pin(&self) -> u8;
}

impl GpioteOutputPin for Pin<Output<OpenDrain>> {
    fn pin(&self) -> u8 {
        self.pin()
    }
}

impl GpioteOutputPin for Pin<Output<PushPull>> {
    fn pin(&self) -> u8 {
        self.pin()
    }
}
