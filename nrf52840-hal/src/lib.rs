#![no_std]

extern crate cast;
extern crate cortex_m;
extern crate embedded_hal as hal;
extern crate nb;
extern crate void;
pub extern crate nrf52840;


pub mod delay;
// pub mod spim;
pub mod gpio;
pub mod clocks;
pub mod time;
pub mod timer;

pub mod prelude {
    pub use hal::prelude::*;

    pub use clocks::ClocksExt;
    pub use gpio::GpioExt;
    // pub use spim::SpimExt;
    pub use time::U32Ext;
    pub use timer::TimerExt;
}


pub use clocks::Clocks;
pub use delay::Delay;
// pub use spim::Spim;
pub use timer::Timer;
