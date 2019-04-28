#![no_std]

use embedded_hal as hal;
pub use nrf52840_pac;
pub use nrf52_hal_common::*;

pub mod prelude {
    pub use crate::hal::prelude::*;
    pub use nrf52_hal_common::prelude::*;

    pub use crate::clocks::ClocksExt;
    pub use crate::gpio::GpioExt;
    pub use crate::time::U32Ext;
    pub use crate::uarte::UarteExt;
}

pub use crate::clocks::Clocks;
pub use crate::delay::Delay;
pub use crate::saadc::Saadc;
pub use crate::spim::Spim;
pub use crate::timer::Timer;
pub use crate::uarte::Uarte;
pub use crate::temp::Temp;
