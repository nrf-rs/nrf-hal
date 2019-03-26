use crate::target::CLOCK;

pub use crate::target::clock::lfclksrc::SRCW as LfOscSource;

pub trait ClocksExt {
    fn constrain(self) -> Clocks<Internal, Internal, LfOscStopped>;
}

pub const HFCLK_FREQ: u32 = 64_000_000;
pub const LFCLK_FREQ: u32 =     32_768;

pub struct Clocks<H, L, LSTAT> {
    hfclk: H,
    lfclk: L,
    lfstat: LSTAT,
    periph: CLOCK,
}

pub struct Internal;
pub struct ExternalOscillator;
pub struct LfOscSynthesized;

pub struct LfOscStarted;
pub struct LfOscStopped;

impl<H, L, LSTAT> Clocks<H, L, LSTAT> {
    pub fn enable_ext_hfosc(self) -> Clocks<ExternalOscillator, L, LSTAT> {
        self.periph.tasks_hfclkstart.write(|w| unsafe { w.bits(1) });
        Clocks {
            hfclk: ExternalOscillator,
            lfclk: self.lfclk,
            lfstat: self.lfstat,
            periph: self.periph,
        }
    }

    pub fn disable_ext_hfosc(self) -> Clocks<Internal, L, LSTAT> {
        self.periph.tasks_hfclkstop.write(|w| unsafe { w.bits(1) });
        Clocks {
            hfclk: Internal,
            lfclk: self.lfclk,
            lfstat: self.lfstat,
            periph: self.periph,
        }
    }

    pub fn stop_lfclk(self) -> Clocks<H, L, LfOscStopped> {
        self.periph.tasks_lfclkstop.write(|w| unsafe { w.bits(1) });
        Clocks {
            hfclk: self.hfclk,
            lfclk: self.lfclk,
            lfstat: LfOscStopped,
            periph: self.periph,
        }
    }

    pub fn start_lfclk(self) -> Clocks<H, L, LfOscStarted> {
        self.periph.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });
        Clocks {
            hfclk: self.hfclk,
            lfclk: self.lfclk,
            lfstat: LfOscStarted,
            periph: self.periph,
        }
    }
}

impl<H, L> Clocks<H, L, LfOscStopped> {
    pub fn set_lfclk_src(self, src: LfOscSource, external: bool, bypass: bool) -> Clocks<H, Internal, LfOscStopped> {
        // Verify datasheet requirements, nRF52832 PS 1.4 Table 26
        debug_assert!(match (&src, external, bypass) {
            // RC and synth MUST NOT set bypass or external
            (LfOscSource::RC, false, false) => true,
            (LfOscSource::SYNTH, false, false) => true,
            // RC may cannot set bypass with external
            (LfOscSource::RC, false, true) => false,
            // All other RC settings are valid
            (LfOscSource::RC, _, _) => true,
            // All other settings are invalid
            (_,               _, _) => false,
        });
        self.periph.lfclksrc.write(|w| {
            w.src().variant(src);
            w.bypass().bit(bypass);
            w.external().bit(external)
        });
        Clocks {
            hfclk: self.hfclk,
            lfclk: Internal,
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
