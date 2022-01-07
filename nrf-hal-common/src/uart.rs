//! HAL interface to the UART peripheral.

use core::fmt::{self, Write};
use core::ops::Deref;

use nb::block;
use void::Void;

use crate::gpio::{Floating, Input, Output, Pin, PushPull};
use crate::pac::{uart0, UART0};

// Re-export SVD variants to allow user to directly set values.
pub use uart0::{baudrate::BAUDRATE_A as Baudrate, config::PARITY_A as Parity};

/// Interface to a UART instance.
pub struct Uart<T>(T);

#[derive(Debug)]
pub enum Error {}

impl<T> Uart<T>
where
    T: Instance,
{
    pub fn new(uart: T, pins: Pins, parity: Parity, baudrate: Baudrate) -> Self {
        // Fill register with dummy data to trigger txd event.
        uart.txd.write(|w| unsafe { w.bits(0) });

        // Required pins
        uart.pseltxd
            .write(|w| unsafe { w.bits(pins.txd.pin().into()) });
        uart.pselrxd
            .write(|w| unsafe { w.bits(pins.rxd.pin().into()) });

        // Optional pins
        uart.pselcts.write(|w| unsafe {
            if let Some(ref pin) = pins.cts {
                w.bits(pin.pin().into())
            } else {
                // Disconnect
                w.bits(0xFFFFFFFF)
            }
        });

        uart.pselrts.write(|w| unsafe {
            if let Some(ref pin) = pins.rts {
                w.bits(pin.pin().into())
            } else {
                // Disconnect
                w.bits(0xFFFFFFFF)
            }
        });

        // Set baud rate.
        uart.baudrate.write(|w| w.baudrate().variant(baudrate));

        // Set parity.
        let hardware_flow_control = pins.rts.is_some() && pins.cts.is_some();
        uart.config
            .write(|w| w.hwfc().bit(hardware_flow_control).parity().variant(parity));

        // Enable UART function.
        uart.enable.write(|w| w.enable().enabled());

        // Fire up transmitting and receiving task.
        uart.tasks_starttx.write(|w| unsafe { w.bits(1) });
        uart.tasks_startrx.write(|w| unsafe { w.bits(1) });

        Uart(uart)
    }

    /// Return the raw interface to the underlying UARTE peripheral.
    pub fn free(self) -> (T, Pins) {
        let rxd = self.0.pselrxd.read();
        let txd = self.0.pseltxd.read();
        let cts = self.0.pselcts.read();
        let rts = self.0.pselrts.read();
        self.0.pselrxd.reset(); // Reset pins
        self.0.pseltxd.reset();
        self.0.pselcts.reset();
        self.0.pselrts.reset();
        (
            self.0,
            Pins {
                rxd: unsafe { Pin::from_psel_bits(rxd.bits()) },
                txd: unsafe { Pin::from_psel_bits(txd.bits()) },
                cts: if cts.bits() != 0xFFFFFFFF {
                    Some(unsafe { Pin::from_psel_bits(cts.bits()) })
                } else {
                    None
                },
                rts: if rts.bits() != 0xFFFFFFFF {
                    Some(unsafe { Pin::from_psel_bits(rts.bits()) })
                } else {
                    None
                },
            },
        )
    }
}

impl<T> embedded_hal::serial::Read<u8> for Uart<T>
where
    T: Instance,
{
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        match self.0.events_rxdrdy.read().bits() {
            0 => Err(nb::Error::WouldBlock),
            _ => {
                // Reset ready for receive event.
                self.0.events_rxdrdy.reset();

                // Read one 8bit value.
                let byte = self.0.rxd.read().bits() as u8;

                Ok(byte)
            }
        }
    }
}

impl<T> embedded_hal::serial::Write<u8> for Uart<T>
where
    T: Instance,
{
    type Error = Void;

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }

    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        // Are we ready for sending out next byte?
        if self.0.events_txdrdy.read().bits() == 1 {
            // Reset ready for transmit event.
            self.0.events_txdrdy.reset();

            // Send byte.
            self.0.txd.write(|w| unsafe { w.bits(u32::from(byte)) });

            Ok(())
        } else {
            // We're not ready, tell application to try again
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<T> Write for Uart<T>
where
    Uart<T>: embedded_hal::serial::Write<u8>,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        use embedded_hal::serial::Write;
        let _ = s.as_bytes().iter().map(|c| block!(self.write(*c))).last();
        Ok(())
    }
}

pub struct Pins {
    pub rxd: Pin<Input<Floating>>,
    pub txd: Pin<Output<PushPull>>,
    pub cts: Option<Pin<Input<Floating>>>,
    pub rts: Option<Pin<Output<PushPull>>>,
}

pub trait Instance: Deref<Target = uart0::RegisterBlock> + sealed::Sealed {}

mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for UART0 {}
impl Instance for UART0 {}
