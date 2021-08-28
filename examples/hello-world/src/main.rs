#![no_std]
#![no_main]

// Simple UART example

#[cfg(feature = "52840")]
use nrf52840_hal as hal;
#[cfg(feature = "9160")]
use nrf9160_hal as hal;

use core::fmt::Write;
use hal::{gpio, uarte, uarte::Uarte};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::pac::Peripherals::take().unwrap();

    #[cfg(feature = "52840")]
    let (uart0, cdc_pins) = {
        let p0 = gpio::p0::Parts::new(p.P0);
        (
            p.UARTE0,
            uarte::Pins {
                txd: p0.p0_06.into_push_pull_output(gpio::Level::High).degrade(),
                rxd: p0.p0_08.into_floating_input().degrade(),
                cts: Some(p0.p0_07.into_floating_input().degrade()),
                rts: Some(p0.p0_05.into_push_pull_output(gpio::Level::High).degrade()),
            },
        )
    };
    #[cfg(feature = "9160")]
    let (uart0, cdc_pins) = {
        let p0 = gpio::p0::Parts::new(p.P0_NS);
        (
            p.UARTE0_NS,
            uarte::Pins {
                txd: p0.p0_29.into_push_pull_output(gpio::Level::High).degrade(),
                rxd: p0.p0_28.into_floating_input().degrade(),
                cts: Some(p0.p0_26.into_floating_input().degrade()),
                rts: Some(p0.p0_27.into_push_pull_output(gpio::Level::High).degrade()),
            },
        )
    };

    let mut uarte = Uarte::new(
        uart0,
        cdc_pins,
        uarte::Parity::EXCLUDED,
        uarte::Baudrate::BAUD115200,
        uarte::Stopbits::ONE
    );

    write!(uarte, "Hello, World!\r\n").unwrap();

    loop {
        cortex_m::asm::wfi();
    }
}

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
