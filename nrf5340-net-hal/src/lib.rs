#![no_std]
#![doc(html_root_url = "https://docs.rs/nrf5340-net-hal/0.17.1")]

pub use nrf_hal_common::*;

pub mod prelude {
    pub use nrf_hal_common::prelude::*;
}
pub use crate::clocks::Clocks;
pub use crate::delay::Delay;
#[cfg(feature = "monotonic")]
pub use crate::monotonic;
pub use crate::rtc::Rtc;
pub use crate::spim::Spim;
pub use crate::timer::Timer;
pub use crate::twim::Twim;
pub use crate::uarte::Uarte;
