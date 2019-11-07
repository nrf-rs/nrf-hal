#![no_std]
#![no_main]

extern crate cortex_m_rt as rt;
extern crate nb;
extern crate nrf52_dk_bsp as dk;
extern crate panic_semihosting;

use dk::{nrf52832_hal::Timer, prelude::*, Board};
use nb::block;
use rt::entry;

#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();

    let mut timer = Timer::new(board.TIMER0);

    let mut led_is_on = false;
    loop {
        if led_is_on {
            board.leds.led_1.disable();
        } else {
            board.leds.led_1.enable();
        }
        timer.start(1_000_000_u32);
        block!(timer.wait()).unwrap();
        led_is_on = !led_is_on;
    }
}
