//! Time units.

/// Bits per second.
#[derive(Clone, Copy)]
pub struct Bps(pub u32);

/// Hertz.
#[derive(Clone, Copy)]
pub struct Hertz(pub u32);

/// KiloHertz.
#[derive(Clone, Copy)]
pub struct KiloHertz(pub u32);

/// MegaHertz.
#[derive(Clone, Copy)]
pub struct MegaHertz(pub u32);

/// Extension trait that adds convenience methods to the `u32` type.
pub trait U32Ext {
    /// Wrap in `Bps`.
    fn bps(self) -> Bps;

    /// Wrap in `Hertz`.
    fn hz(self) -> Hertz;

    /// Wrap in `KiloHertz`.
    fn khz(self) -> KiloHertz;

    /// Wrap in `MegaHertz`.
    fn mhz(self) -> MegaHertz;
}

impl U32Ext for u32 {
    fn bps(self) -> Bps {
        Bps(self)
    }

    fn hz(self) -> Hertz {
        Hertz(self)
    }

    fn khz(self) -> KiloHertz {
        KiloHertz(self)
    }

    fn mhz(self) -> MegaHertz {
        MegaHertz(self)
    }
}

impl From<KiloHertz> for Hertz {
    fn from(val: KiloHertz) -> Self {
        Hertz(val.0 * 1_000)
    }
}

impl From<MegaHertz> for Hertz {
    fn from(val: MegaHertz) -> Self {
        Hertz(val.0 * 1_000_000)
    }
}

impl From<MegaHertz> for KiloHertz {
    fn from(val: MegaHertz) -> Self {
        KiloHertz(val.0 * 1_000)
    }
}
