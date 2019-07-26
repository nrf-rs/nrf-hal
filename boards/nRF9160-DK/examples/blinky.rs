#![no_std]
#![no_main]

extern crate cortex_m_rt as rt;
extern crate nb;
extern crate nrf9160_dk_bsp as bsp;
extern crate panic_semihosting;

use core::fmt::Write;
use bsp::{hal::Timer, prelude::*, Board};
use nb::block;
use rt::entry;

#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0_NS);

    writeln!(board.cdc, "Hello, world!").unwrap();

    let mut led_is_on = false;
    loop {
        if led_is_on {
            board.leds.led_1.off();
        } else {
            board.leds.led_1.on();
        }
        timer.start(1_000_000_u32);
        block!(timer.wait()).unwrap();
        led_is_on = !led_is_on;
    }
}
