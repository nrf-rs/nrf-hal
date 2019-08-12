#![no_main]
#![no_std]

use core::fmt::Write;
use cortex_m_rt::entry;
use nb::block;

#[allow(unused_imports)]
extern crate panic_semihosting;

use nrf52840_mdk_bsp::{
    hal::{
        prelude::*,
        timer::{self, Timer},
    },
    Board,
};

#[entry]
fn main() -> ! {
    let mut nrf52 = Board::take().unwrap();

    let mut timer = Timer::new(nrf52.TIMER0);

    // Write to the serial port every second.
    loop {
        write!(nrf52.cdc, ".").ok();
        delay(&mut timer, 1_000_000); // 1s
    }
}

fn delay<T>(timer: &mut Timer<T>, cycles: u32)
where
    T: timer::Instance,
{
    timer.start(cycles);
    let _ = block!(timer.wait());
}
