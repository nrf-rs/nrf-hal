#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m_rt::entry;

use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use ssd1306::prelude::*;
use ssd1306::Builder;

#[cfg(feature = "52832")]
use nrf52832_hal::{
    nrf52832_pac as pac,
    gpio::*,
    twim,
    prelude::TwimExt,
};

#[cfg(feature = "52840")]
use nrf52840_hal::{
    nrf52840_pac as pac,
    gpio::*,
    twim,
    prelude::TwimExt,
};


/// TWI write example code using an SSD1306 OLED display:
/// https://cdn-shop.adafruit.com/datasheets/SSD1306.pdf
///
/// Connect SDA to P0.27 and SCL to pin P0.26
///
/// You should see the words "Hello Rust!" on the display.
#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    let port0 = p.P0.split();

    let scl = port0.p0_26.into_floating_input().degrade();
    let sda = port0.p0_27.into_floating_input().degrade();

    let pins = twim::Pins { scl, sda };

    let i2c = p.TWIM0.constrain(pins, twim::Frequency::K100);

    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();

    disp.init().expect("Display initialization");
    disp.flush().expect("Cleans the display");

    disp.draw(
        Font6x8::render_str("Hello Rust!")
            .with_stroke(Some(1u8.into()))
            .translate(Coord::new(10, 24))
            .into_iter(),
    );

    disp.flush().expect("Render display");

    loop {}
}
