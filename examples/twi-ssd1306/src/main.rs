#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m_rt::entry;
use embedded_graphics::{mono_font::ascii::FONT_5X8, text::Text};
use embedded_graphics::{mono_font::MonoTextStyle, pixelcolor::BinaryColor, prelude::*};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

#[cfg(feature = "52832")]
use nrf52832_hal::{
    gpio::*,
    pac,
    twim::{self, Twim},
};

#[cfg(feature = "52840")]
use nrf52840_hal::{
    gpio::*,
    pac,
    twim::{self, Twim},
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
    let port0 = p0::Parts::new(p.P0);

    let scl = port0.p0_26.into_floating_input().degrade();
    let sda = port0.p0_27.into_floating_input().degrade();

    let pins = twim::Pins { scl, sda };

    let i2c = Twim::new(p.TWIM0, pins, twim::Frequency::K100);

    let interface = I2CDisplayInterface::new(i2c);
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    disp.init().expect("Display initialization");
    disp.flush().expect("Cleans the display");

    let style = MonoTextStyle::new(&FONT_5X8, BinaryColor::On);
    Text::new("Hello Rust!", Point::new(10, 24), style)
        .draw(&mut disp)
        .expect("Drawing text");

    disp.flush().expect("Render display");

    loop {}
}
