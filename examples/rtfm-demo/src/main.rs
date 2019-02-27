#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_semihosting;

use cortex_m_semihosting::{debug, hprintln};
use rtfm::app;

#[cfg(feature = "52810")]
use nrf52810_hal as hal;

#[cfg(feature = "52832")]
use nrf52832_hal as hal;

#[cfg(feature = "52840")]
use nrf52840_hal as hal;


#[app(device = crate::hal::target)]
const APP: () = {
    #[init]
    fn init() {
        hprintln!("init").unwrap();
    }

    #[idle]
    fn idle() -> ! {
        hprintln!("idle").unwrap();

        debug::exit(debug::EXIT_SUCCESS);

        loop {}
    }
};
