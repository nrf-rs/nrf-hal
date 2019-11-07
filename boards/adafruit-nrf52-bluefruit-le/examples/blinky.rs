#![no_main]
#![no_std]

extern crate panic_halt;

use adafruit_nrf52_bluefruit_le::{prelude::*, Board};
use core::fmt::Write;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use nb::block;
use nrf52832_hal::timer::{self, Timer};

#[entry]
fn main() -> ! {
    let mut b = Board::take().unwrap();

    let mut timer = Timer::new(b.TIMER4);

    b.leds.red.disable();
    b.leds.blue.disable();
    let boot: [u8; 6] = [
        0x62, // b
        0x6f, // o
        0x6f, // o
        0x74, // t
        0x0d, // CR
        0x0a, // LF
    ];
    b.leds.red.enable();
    b.cdc.write(&boot).unwrap();
    delay(&mut timer, 1_000_000);

    b.leds.red.disable();
    b.leds.blue.disable();
    let mut count: u32 = 0;
    loop {
        count += 1;
        b.leds.red.enable();
        let r = write!(b.cdc, "what a big data chonk: {}\r\n", count);
        b.leds.red.disable();
        match r {
            Ok(()) => {
                for _ in 0..3 {
                    b.leds.blue.enable();
                    delay(&mut timer, 100_000);
                    b.leds.blue.disable();
                    delay(&mut timer, 100_000);
                }
            }
            Err(_) => {
                for _ in 0..3 {
                    b.leds.red.enable();
                    delay(&mut timer, 100_000);
                    b.leds.red.disable();
                    delay(&mut timer, 100_000);
                }
            }
        }

        delay(&mut timer, 1_000_000);
    }
}

fn delay<T>(timer: &mut Timer<T>, cycles: u32)
where
    T: timer::Instance,
{
    timer.start(cycles);
    block!(timer.wait()).unwrap();
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    let mut b = Board::steal();
    b.leds.red.enable();
    write!(b.cdc, "!!! Hard fault: {:?}", ef).unwrap();
    loop {}
}
