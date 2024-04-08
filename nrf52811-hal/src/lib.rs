#![no_std]
#![doc(html_root_url = "https://docs.rs/nrf52811-hal/0.17.0")]

pub use nrf_hal_common::*;

pub mod prelude {
    pub use nrf_hal_common::prelude::*;
}

pub use crate::ccm::Ccm;
pub use crate::clocks::Clocks;
pub use crate::delay::Delay;
pub use crate::ecb::Ecb;
pub use crate::saadc::Saadc;
pub use crate::spim::Spim;
pub use crate::temp::Temp;
pub use crate::timer::Timer;
pub use crate::uarte::Uarte;
