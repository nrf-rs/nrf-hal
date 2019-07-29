#![no_std]

use embedded_hal as hal;
pub use nrf52832_pac;
pub use nrf52_hal_common::*;

pub mod prelude {
    pub use crate::hal::prelude::*;
    pub use nrf52_hal_common::prelude::*;
}

pub use crate::clocks::Clocks;
pub use crate::delay::Delay;
pub use crate::rtc::Rtc;
pub use crate::saadc::Saadc;
pub use crate::spim::Spim;
pub use crate::temp::Temp;
pub use crate::timer::Timer;
pub use crate::uarte::Uarte;
