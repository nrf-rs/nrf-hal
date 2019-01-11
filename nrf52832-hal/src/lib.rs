#![no_std]

pub use nrf52832_pac;
pub use nrf52_hal_common::*;

pub mod prelude {
    pub use nrf52_hal_common::prelude::*;
}

// blocking
pub use crate::clocks::Clocks;
pub use crate::delay::Delay;
pub use crate::spim::Spim;
pub use crate::timer::Timer;
pub use crate::uarte::Uarte;

// nonblocking
pub use crate::uarte::nonblocking::UarteAsync;
