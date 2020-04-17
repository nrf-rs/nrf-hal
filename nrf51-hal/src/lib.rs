#![no_std]

use embedded_hal as hal;
pub use nrf51 as pac;
pub use nrf_hal_common::*;

pub mod prelude {
    pub use crate::hal::prelude::*;
    pub use nrf_hal_common::prelude::*;
}

pub use crate::clocks::Clocks;
//pub use crate::delay::Delay;
pub use crate::adc::Adc;
pub use crate::rtc::Rtc;
//pub use crate::spim::Spim;
pub use crate::temp::Temp;
pub use crate::timer::Timer;
pub use crate::twi::Twi;
pub use crate::uart::Uart;
