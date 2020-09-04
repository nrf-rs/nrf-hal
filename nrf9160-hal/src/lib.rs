#![no_std]
#![doc(html_root_url = "https://docs.rs/nrf9160-hal/0.11.1")]

use embedded_hal as hal;
pub use nrf_hal_common::*;

pub mod prelude {
    pub use crate::hal::prelude::*;
    pub use nrf_hal_common::prelude::*;

    pub use crate::time::U32Ext;
}

pub use crate::clocks::Clocks;
pub use crate::delay::Delay;
pub use crate::rtc::Rtc;
pub use crate::saadc::Saadc;
pub use crate::spim::Spim;
pub use crate::timer::Timer;
pub use crate::twim::Twim;
pub use crate::uarte::Uarte;
