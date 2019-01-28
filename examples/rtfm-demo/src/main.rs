#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_semihosting;

use cortex_m_semihosting::{debug, hprintln};
use rtfm::app;

#[cfg(feature = "52832")]
use nrf52832_pac;

#[cfg(feature = "52840")]
use nrf52840_pac;


#[cfg_attr(feature="52832", app(device = nrf52832_pac))]
#[cfg_attr(feature="52840", app(device = nrf52840_pac))]
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
