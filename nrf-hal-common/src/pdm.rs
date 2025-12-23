//! HAL interface to the PDM peripheral
//!
//! The PDM (Pulse Density Modulation) peripheral enables the sampling of pulse
//! density signals.

use crate::{
    hal::digital::v2::OutputPin,
    gpio::{Floating, Input, Output, Pin, PushPull},
    pac::PDM,
};
// Publicly re-export configuration enums for convenience
pub use crate::pac::pdm::{
    gainl::GAINL_A as GainL, 
    gainr::GAINR_A as GainR, 
    mode::EDGE_A as Sampling,
    mode::OPERATION_A as Channel, 
    pdmclkctrl::FREQ_A as Frequency, 
    ratio::RATIO_A as Ratio,
};

pub struct Pdm {
    pdm: PDM,
    clk: Pin<Output<PushPull>>,
    din: Pin<Input<Floating>>,
}

impl Pdm {
    /// Create the `Pdm` instance, initialize the raw peripheral and enable it.
    pub fn new(pdm: PDM, mut clk: Pin<Output<PushPull>>, din: Pin<Input<Floating>>) -> Self {
        // Set the CLK pin low as requested by the docs
        clk.set_low().unwrap();

        // Configure the pins
        pdm.psel.clk.write(|w| {
            unsafe { w.bits(clk.psel_bits()) };
            w.connect().connected()
        });
        pdm.psel.din.write(|w| {
            unsafe { w.bits(din.psel_bits()) };
            w.connect().connected()
        });

        Self { pdm, clk, din }
    }

    /// Set clock frequency
    pub fn frequency(&self, frequency: Frequency) -> &Self {
        self.pdm.pdmclkctrl.write(|w| w.freq().variant(frequency));

        self
    }

    /// Set the hardware decimation filter gain for the left channel (this is
    /// also the gain used in mono mode)
    pub fn left_gain(&self, gain: GainL) -> &Self {
        self.pdm.gainl.write(|w| w.gainl().variant(gain));

        self
    }

    /// Set the hardware decimation filter gain for the left channel (this is
    /// also the gain used in mono mode)
    pub fn right_gain(&self, gain: GainR) -> &Self {
        self.pdm.gainr.write(|w| w.gainr().variant(gain));

        self
    }

    /// Set the ratio clock frequency/sample rate (sample rate = clock frequency / ratio)
    pub fn ratio(&self, ratio: Ratio) -> &Self {
        self.pdm.ratio.write(|w| w.ratio().variant(ratio));
        
        self
    }

    /// Set whether the left (or mono) samples are taken on a clock rise or fall.
    pub fn sampling(&self, sampling: Sampling) -> &Self {
        self.pdm.mode.write(|w| w.edge().variant(sampling));

        self
    }

    /// Set the channel mode : mono or stereo
    pub fn channel(&self, channel: Channel) -> &Self {
        self.pdm.mode.write(|w| w.operation().variant(channel));

        self
    }

    /// Enable the peripheral
    pub fn enable(&self) {
        self.pdm.enable.write(|w| w.enable().enabled());
    } 

    /// Return ownership of underlying pins and peripheral
    pub fn free(self) -> (PDM, Pin<Output<PushPull>>, Pin<Input<Floating>>) {
        (self.pdm, self.clk, self.din)
    }

    /// Perform one blocking acquisition, filling the given buffer with samples.
    ///
    /// The buffer length must not exceed 2^16 - 1
    pub fn read(&self, buffer: &mut [i16]) {
        // Setup the buffer address and the number of samples to acquire
        self.pdm.sample
            .ptr
            .write(|w| unsafe { w.sampleptr().bits(buffer.as_ptr() as u32) });
        self.pdm.sample
            .maxcnt
            .write(|w| unsafe { w.buffsize().bits(buffer.len() as u16) });
        
        // Start the acquisition
        self.pdm.tasks_start.write(|w| w.tasks_start().set_bit());

        // Wait for the acquisition to start then prevent it from restarting
        // after
        while !self.pdm.events_started.read().events_started().bit_is_set() {}
        self.pdm.sample
            .maxcnt
            .write(|w| unsafe { w.buffsize().bits(0) });
        
        // Wait for the acquisition to finish
        while !self.pdm.events_end.read().events_end().bit_is_set() {}

        self.clear_events();
    }

    /// Clear all events
    fn clear_events(&self) {
        self.pdm.events_started.write(|w| w.events_started().clear_bit());
        self.pdm.events_stopped.write(|w| w.events_stopped().clear_bit());
        self.pdm.events_end.write(|w| w.events_end().clear_bit());
    }
}
