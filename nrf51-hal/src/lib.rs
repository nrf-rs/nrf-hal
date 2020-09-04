#![no_std]
#![doc(html_root_url = "https://docs.rs/nrf51-hal/0.11.1")]

use embedded_hal as hal;
pub use nrf_hal_common::*;

pub mod prelude {
    pub use crate::hal::prelude::*;
    pub use nrf_hal_common::prelude::*;
}

pub use crate::adc::Adc;
pub use crate::ccm::Ccm;
pub use crate::clocks::Clocks;
pub use crate::ecb::Ecb;
pub use crate::rtc::Rtc;
pub use crate::spi::Spi;
pub use crate::temp::Temp;
pub use crate::timer::Timer;
pub use crate::twi::Twi;
pub use crate::uart::Uart;
