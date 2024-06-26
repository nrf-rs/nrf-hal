#![no_std]
#![doc(html_root_url = "https://docs.rs/nrf9160-hal/0.18.0")]

pub use nrf_hal_common::*;

pub mod prelude {
    pub use nrf_hal_common::prelude::*;
}

pub use crate::clocks::Clocks;
pub use crate::delay::Delay;
pub use crate::rtc::Rtc;
pub use crate::saadc::Saadc;
pub use crate::spim::Spim;
pub use crate::timer::Timer;
pub use crate::twim::Twim;
pub use crate::uarte::Uarte;
