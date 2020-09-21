#![no_main]
#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};
use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
use nrf52832_hal as hal;
use nrf52832_hal::gpio::Level;

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
    let port0 = hal::gpio::p0::Parts::new(p.P0);
    let button = port0.p0_13.into_pullup_input();
    let mut led = port0.p0_17.into_push_pull_output(Level::Low);

    rprintln!("Blinky button demo starting");
    loop {
        if button.is_high().unwrap() {
            led.set_high().unwrap();
        } else {
            led.set_low().unwrap();
        }
    }
}
