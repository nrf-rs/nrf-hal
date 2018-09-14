#![no_main]
#![no_std]

#[macro_use]
extern crate cortex_m_rt;
#[macro_use]
extern crate nb;

extern crate adafruit_nrf52pro;
extern crate panic_semihosting;

use adafruit_nrf52pro::hal::{
    prelude::*,
    gpio::Level,
    timer::Timer,
};
use adafruit_nrf52pro::nrf52::{Peripherals};
use adafruit_nrf52pro::Pins;

entry!(main);

fn main() -> ! {
    let p = Peripherals::take().unwrap();
    let pins = Pins::new(p.P0.split());

    let mut led1 = pins.led1.into_push_pull_output(Level::Low);
    let mut led2 = pins.led2.into_push_pull_output(Level::Low);

    let mut timer = p.TIMER0.constrain();

    // Alternately flash the red and blue leds
    loop {
        led1.set_low();
        led2.set_high();
        delay(&mut timer, 250_000); // 250ms
        led1.set_high();
        led2.set_low();
        delay(&mut timer, 1_000_000); // 1s
    }
}

fn delay<T>(timer: &mut Timer<T>, cycles: u32)
where
    T: TimerExt,
{
    timer.start(cycles);
    block!(timer.wait());
}
