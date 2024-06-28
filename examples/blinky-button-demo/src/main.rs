#![no_main]
#![no_std]

use embedded_hal::digital::InputPin;
use embedded_hal::digital::OutputPin;
use rtt_target::{rprintln, rtt_init_print};

#[cfg(feature = "52832")]
use nrf52832_hal as hal;
#[cfg(feature = "52832")]
use nrf52832_hal::gpio::Level;

#[cfg(feature = "9120")]
use nrf9120_hal as hal;
#[cfg(feature = "9120")]
use nrf9120_hal::gpio::Level;

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();
    let p = hal::pac::Peripherals::take().unwrap();

    #[cfg(feature = "52832")]
    let port0 = hal::gpio::p0::Parts::new(p.P0);
    #[cfg(feature = "52832")]
    let mut button = port0.p0_13.into_pullup_input();
    #[cfg(feature = "52832")]
    let mut led = port0.p0_17.into_push_pull_output(Level::Low);

    #[cfg(feature = "9120")]
    let port0 = hal::gpio::p0::Parts::new(p.P0_NS);
    #[cfg(feature = "9120")]
    let mut button = port0.p0_08.into_pullup_input();
    #[cfg(feature = "9120")]
    let mut led = port0.p0_00.into_push_pull_output(Level::Low);

    rprintln!("Blinky button demo starting");
    loop {
        if button.is_high().unwrap() {
            led.set_high().unwrap();
        } else {
            led.set_low().unwrap();
        }
    }
}
