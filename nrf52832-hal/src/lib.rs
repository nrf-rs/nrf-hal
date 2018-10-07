#![no_std]

extern crate embedded_hal as hal;
pub extern crate nrf52;
extern crate nrf52_hal_common;

pub use nrf52_hal_common::*;

pub mod prelude {
    pub use hal::prelude::*;
    pub use nrf52_hal_common::prelude::*;

    pub use clocks::ClocksExt;
    pub use gpio::GpioExt;
    pub use spim::SpimExt;
    pub use time::U32Ext;
    pub use timer::TimerExt;
    pub use uarte::UarteExt;
}

pub use clocks::Clocks;
pub use delay::Delay;
pub use spim::Spim;
pub use timer::Timer;
pub use uarte::Uarte;
