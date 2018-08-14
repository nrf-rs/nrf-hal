use nrf52::CLOCK;
use time::{Hertz, U32Ext};

pub trait ClocksExt {
    fn constrain(self) -> ClocksCfg;
}

pub struct ClocksCfg {
    pub hfclk: HFCLK,
    pub lfclk: LFCLK,
}

pub struct HFCLK {
    _0: ()
}

pub struct LFCLK {
    _0: ()
}

pub struct Clocks {
    hfclk: Hertz,
    lfclk: Hertz,
}

impl HFCLK {
    // TODO: allow external clock selection?
}

impl LFCLK {
    // TODO: allow external clock selection? Calibration?
}

impl ClocksExt for CLOCK {
    fn constrain(self) -> ClocksCfg {
        ClocksCfg {
            hfclk: HFCLK { _0: () },
            lfclk: LFCLK { _0: () },
        }
    }
}

impl ClocksCfg {
    pub fn freeze(self) -> Clocks {
        // TODO - this isn't very useful, can you actually change internal clock speeds?
        Clocks {
            hfclk: 64_000_000.hz(),
            lfclk: 32_768.hz(),
        }
    }
}

impl Clocks {
    pub fn hfclk(&self) -> Hertz {
        self.hfclk
    }

    pub fn lfclk(&self) -> Hertz {
        self.lfclk
    }
}
