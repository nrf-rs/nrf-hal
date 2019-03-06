#![no_std]
#![no_main]

extern crate cortex_m_rt as rt; // v0.5.x

extern crate embedded_hal_spy;
extern crate nrf52832_hal;
extern crate panic_halt;
use embedded_hal::blocking::spi::*;

use cortex_m_rt::entry;
use embedded_hal::digital::OutputPin;
use nrf52832_hal::gpio;
use nrf52832_hal::gpio::p0::*;
use nrf52832_hal::gpio::Level;
use nrf52832_hal::gpio::*;
use nrf52832_hal::prelude::GpioExt;

/// SPIM demonstation code.
/// connect a resistor between pin 22 and 23 on to feed MOSI direct back to MISO
///
/// If all tests Led1 to 4 will light up, in case of error only the failing test
/// one or more Led will remain off.
#[entry]
fn main() -> ! {
    let p = nrf52832_hal::nrf52832_pac::Peripherals::take().unwrap();
    let port0 = p.P0.split();

    let cs: P0_21<gpio::Output<PushPull>> = port0.p0_21.into_push_pull_output(Level::Low);

    let mut led1: P0_17<gpio::Output<PushPull>> = port0.p0_17.into_push_pull_output(Level::High);
    let mut led2: P0_18<gpio::Output<PushPull>> = port0.p0_18.into_push_pull_output(Level::High);
    let mut led3: P0_19<gpio::Output<PushPull>> = port0.p0_19.into_push_pull_output(Level::High);
    let mut led4: P0_20<gpio::Output<PushPull>> = port0.p0_20.into_push_pull_output(Level::High);

    let _btn1 = port0.p0_13.into_pullup_input();
    let _btn2 = port0.p0_14.into_pullup_input();
    let _btn3 = port0.p0_15.into_pullup_input();
    let _btn4 = port0.p0_16.into_pullup_input();

    let spiclk = port0.p0_24.into_push_pull_output(Level::Low).degrade();
    let spimosi = port0.p0_23.into_push_pull_output(Level::Low).degrade();
    let spimiso = port0.p0_22.into_floating_input().degrade();

    let mut tests_ok = true;
    let pins = nrf52832_hal::spim::Pins {
        sck: spiclk,
        miso: Some(spimiso),
        mosi: Some(spimosi),
    };
    let mut spi = p.SPIM2.constrain(
        pins,
        nrf52832_hal::spim::Frequency::K500,
        nrf52832_hal::spim::MODE_0,
        0,
    );

    let reference_data = b"Hello,echo Loopback";
    // Read only test vector
    let test_vec1 = *reference_data;
    let mut readbuf = [0; 255];
    use nrf52832_hal::spim::SpimExt;

    // This will write 8 bytes, then shift out ORC

    // Note :     spi.read( &mut cs.degrade(), reference_data, &mut readbuf )
    //            will fail because reference data is in flash, the copy to
    //            an array will move it to RAM.

    match spi.read(&mut cs.degrade(), &test_vec1, &mut readbuf) {
        Ok(_) => {
            for i in 0..test_vec1.len() {
                tests_ok &= test_vec1[i] == readbuf[i];
            }
            if !tests_ok {
                led1.set_low();
            } else {
                const ORC: u8 = 0;
                for i in test_vec1.len()..readbuf.len() {
                    if ORC != readbuf[i] {
                        tests_ok = false;
                        led1.set_low();
                    }
                }
            }
        }
        Err(_) => {
            tests_ok = false;
            led1.set_low();
        }
    }

    // Wrap interface in embedded-hal-spy to access embedded_hal traits
    let mut eh_spi = embedded_hal_spy::new(spi, |_| {});
    use embedded_hal::blocking::spi::Write;
    match eh_spi.write(reference_data) {
        Ok(_) => {}
        Err(_) => {
            tests_ok = false;
            led2.set_low()
        }
    }

    let mut test_vec2 = *reference_data;
    match eh_spi.transfer(&mut test_vec2) {
        Ok(_) => {
            for i in 0..test_vec2.len() {
                if test_vec2[i] != reference_data[i] {
                    tests_ok = false;
                    led3.set_low();
                }
            }
        }
        Err(_) => {
            tests_ok = false;
            led4.set_low();
        }
    }

    if tests_ok {
        led1.set_low();
        led2.set_low();
        led3.set_low();
        led4.set_low();
    }

    loop {}
}
