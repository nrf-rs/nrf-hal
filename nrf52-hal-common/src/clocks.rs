//! Configuration and control of the High and Low Frequency Clock
//! sources

use crate::target::CLOCK;

// ZST Type States

/// Internal/RC Oscillator
pub struct Internal;

/// External Crystal Oscillator
pub struct ExternalOscillator;

/// Low Frequency Clock synthesize from High Frequency Clock
pub struct LfOscSynthesized;

/// Low Frequency Clock Started
pub struct LfOscStarted;

/// Low Frequency Clock Stopped
pub struct LfOscStopped;

/// Extension trait for the CLOCK peripheral
pub trait ClocksExt {
    fn constrain(self) -> Clocks<Internal, Internal, LfOscStopped>;
}

/// High Frequency Clock Frequency (in Hz)
pub const HFCLK_FREQ: u32 = 64_000_000;
/// Low Frequency Clock Frequency (in Hz)
pub const LFCLK_FREQ: u32 = 32_768;

/// A high level abstraction for the CLOCK peripheral
pub struct Clocks<H, L, LSTAT> {
    hfclk: H,
    lfclk: L,
    lfstat: LSTAT,
    periph: CLOCK,
}

impl<H, L, LSTAT> Clocks<H, L, LSTAT> {
    /// Use an external oscillator as the high frequency clock source
    pub fn enable_ext_hfosc(self) -> Clocks<ExternalOscillator, L, LSTAT> {
        self.periph.tasks_hfclkstart.write(|w| unsafe { w.bits(1) });

        // Datasheet says this is likely to take 0.36ms
        while self.periph.events_hfclkstarted.read().bits() != 1 {}
        self.periph
            .events_hfclkstarted
            .write(|w| unsafe { w.bits(0) });

        Clocks {
            hfclk: ExternalOscillator,
            lfclk: self.lfclk,
            lfstat: self.lfstat,
            periph: self.periph,
        }
    }

    /// Use the internal oscillator as the high frequency clock source
    pub fn disable_ext_hfosc(self) -> Clocks<Internal, L, LSTAT> {
        self.periph.tasks_hfclkstop.write(|w| unsafe { w.bits(1) });
        Clocks {
            hfclk: Internal,
            lfclk: self.lfclk,
            lfstat: self.lfstat,
            periph: self.periph,
        }
    }

    /// Start the Low Frequency clock
    pub fn start_lfclk(self) -> Clocks<H, L, LfOscStarted> {
        self.periph.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });

        // Datasheet says this could take 100us from synth source
        // 600us from rc source, 0.25s from an external source
        while self.periph.events_lfclkstarted.read().bits() != 1 {}
        self.periph
            .events_lfclkstarted
            .write(|w| unsafe { w.bits(0) });

        Clocks {
            hfclk: self.hfclk,
            lfclk: self.lfclk,
            lfstat: LfOscStarted,
            periph: self.periph,
        }
    }
}

/// Allowable configuration options for the low frequency oscillator when
/// driven fron an external crystal
pub enum LfOscConfiguration {
    NoExternalNoBypass,
    ExternalNoBypass,
    ExternalAndBypass,
}

impl<H, L> Clocks<H, L, LfOscStarted> {
    /// Stop the Low Frequency clock
    pub fn stop_lfclk(self) -> Clocks<H, L, LfOscStopped> {
        self.periph.tasks_lfclkstop.write(|w| unsafe { w.bits(1) });
        Clocks {
            hfclk: self.hfclk,
            lfclk: self.lfclk,
            lfstat: LfOscStopped,
            periph: self.periph,
        }
    }
}

impl<H, L> Clocks<H, L, LfOscStopped> {
    /// Use the internal RC Oscillator for the low frequency clock source
    pub fn set_lfclk_src_rc(self) -> Clocks<H, Internal, LfOscStopped> {
        self.periph
            .lfclksrc
            .write(|w| w.src().rc().bypass().disabled().external().disabled());
        Clocks {
            hfclk: self.hfclk,
            lfclk: Internal,
            lfstat: self.lfstat,
            periph: self.periph,
        }
    }

    /// Generate the Low Frequency clock from the high frequency clock source
    pub fn set_lfclk_src_synth(self) -> Clocks<H, LfOscSynthesized, LfOscStopped> {
        self.periph
            .lfclksrc
            .write(|w| w.src().synth().bypass().disabled().external().disabled());
        Clocks {
            hfclk: self.hfclk,
            lfclk: LfOscSynthesized,
            lfstat: self.lfstat,
            periph: self.periph,
        }
    }

    /// Use an external crystal to drive the low frequency clock
    pub fn set_lfclk_src_external(
        self,
        cfg: LfOscConfiguration,
    ) -> Clocks<H, ExternalOscillator, LfOscStopped> {
        let (ext, byp) = match cfg {
            LfOscConfiguration::NoExternalNoBypass => (false, false),
            LfOscConfiguration::ExternalNoBypass => (true, false),
            LfOscConfiguration::ExternalAndBypass => (true, true),
        };
        self.periph
            .lfclksrc
            .write(move |w| w.src().xtal().bypass().bit(byp).external().bit(ext));
        Clocks {
            hfclk: self.hfclk,
            lfclk: ExternalOscillator,
            lfstat: self.lfstat,
            periph: self.periph,
        }
    }
}

impl ClocksExt for CLOCK {
    fn constrain(self) -> Clocks<Internal, Internal, LfOscStopped> {
        Clocks {
            hfclk: Internal,
            lfclk: Internal,
            lfstat: LfOscStopped,
            periph: self,
        }
    }
}
