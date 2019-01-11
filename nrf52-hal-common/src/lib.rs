#![no_std]

use embedded_hal as hal;

#[cfg(feature = "52832")]
pub use nrf52832_pac as target;

#[cfg(feature = "52840")]
pub use nrf52840_pac as target;

pub mod delay;
pub mod spim;
pub mod gpio;
pub mod clocks;
pub mod rng;
pub mod time;
pub mod timer;
pub mod twim;
pub mod uarte;

pub mod prelude {
    pub use crate::hal::prelude::*;

    // Blocking
    pub use crate::clocks::ClocksExt;
    pub use crate::gpio::GpioExt;
    pub use crate::rng::RngExt;
    pub use crate::spim::SpimExt;
    pub use crate::time::U32Ext;
    pub use crate::timer::TimerExt;
    pub use crate::twim::TwimExt;
    pub use crate::uarte::UarteExt;

    // Async
    pub use crate::uarte::nonblocking::UarteAsyncExt;
}

// Blocking
pub use crate::clocks::Clocks;
pub use crate::delay::Delay;
pub use crate::spim::Spim;
pub use crate::timer::Timer;
pub use crate::twim::Twim;
pub use crate::uarte::Uarte;

// Async
pub use crate::uarte::nonblocking::UarteAsync;
