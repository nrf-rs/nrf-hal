#![no_std]
#![no_main]

extern crate cortex_m_rt as rt;
extern crate nb;
extern crate actinius_icarus_bsp as bsp;
extern crate panic_semihosting;

use core::fmt::Write;
use bsp::{hal::Timer, prelude::*, Board};
use nb::block;
use rt::entry;

#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0_NS);

    writeln!(board.cdc_uart, "Hello, world!").unwrap();

    let mut led_is_on = false;
    loop {
        if led_is_on {
            board.leds.led_red.disable();
        } else {
            board.leds.led_red.enable();
        }
        timer.start(1_000_000_u32);
        block!(timer.wait()).unwrap();
        led_is_on = !led_is_on;
    }
}
