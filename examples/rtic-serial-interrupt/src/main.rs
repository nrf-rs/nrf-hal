#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_semihosting;

use cortex_m_semihosting::hprintln;
use rtic::app;

#[cfg(feature = "51")]
use nrf51_hal as hal;

#[cfg(feature = "52810")]
use nrf52810_hal as hal;

#[cfg(feature = "52811")]
use nrf52811_hal as hal;

#[cfg(feature = "52832")]
use nrf52832_hal as hal;

#[cfg(feature = "52833")]
use nrf52833_hal as hal;

#[cfg(feature = "52840")]
use nrf52840_hal as hal;

use crate::hal::{
    gpio::{p0, Level},
    pac::{Interrupt::UARTE0_UART0, UARTE0},
    prelude::_embedded_hal_serial_Read,
    uarte::{self, UarteRx},
};

#[app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        serial0: UarteRx<UARTE0>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let p = cx.device;
        let p0parts = p0::Parts::new(p.P0);

        // enable UARTE0 endrx interrupt
        p.UARTE0.intenset.modify(|_, w| w.endrx().set_bit());

        static mut SERIAL0_TX_BUF: [u8; 1] = [0; 1];
        static mut SERIAL0_RX_BUF: [u8; 1] = [0; 1];
        let (_, serial0) = uarte::Uarte::new(
            p.UARTE0,
            uarte::Pins {
                txd: p0parts.p0_00.into_push_pull_output(Level::High).degrade(),
                rxd: p0parts.p0_01.into_floating_input().degrade(),
                cts: None,
                rts: None,
            },
            uarte::Parity::EXCLUDED,
            uarte::Baudrate::BAUD115200,
        )
        .split(unsafe { &mut SERIAL0_TX_BUF }, unsafe {
            &mut SERIAL0_RX_BUF
        })
        .expect("Could not split serial0");

        // on NRF* serial interrupts are only called after the first read
        rtic::pend(UARTE0_UART0);

        init::LateResources { serial0 }
    }

    #[task(binds = UARTE0_UART0, resources = [serial0])]
    fn uarte0_interrupt(cx: uarte0_interrupt::Context) {
        hprintln!("uarte0 interrupt");
        while let Ok(b) = cx.resources.serial0.read() {
            hprintln!("Byte on serial0: {}", b);
        }
    }
};
