//! HAL interface to the SPI peripheral.
use core::ops::Deref;

use crate::{
    gpio::{Floating, Input, Output, Pin, PushPull},
    pac::{spi0, SPI0},
};

pub use embedded_hal::{
    blocking::spi::{transfer, write, write_iter},
    spi::{FullDuplex, Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3},
};
pub use spi0::frequency::FREQUENCY_A as Frequency;

/// Interface to a SPI instance.
pub struct Spi<T>(T);

impl<T> write::Default<u8> for Spi<T>
where
    Spi<T>: FullDuplex<u8>,
    T: Instance,
{
}
impl<T> write_iter::Default<u8> for Spi<T>
where
    Spi<T>: FullDuplex<u8>,
    T: Instance,
{
}
impl<T> transfer::Default<u8> for Spi<T>
where
    Spi<T>: FullDuplex<u8>,
    T: Instance,
{
}

impl<T> FullDuplex<u8> for Spi<T>
where
    T: Instance,
{
    type Error = Error;

    /// Must only be called after `send` as the interface will read and write at the same time.
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        match self.0.events_ready.read().bits() {
            0 => Err(nb::Error::WouldBlock),
            _ => {
                // Read one 8-bit value.
                let byte = self.0.rxd.read().bits() as u8;

                // Reset ready for receive event.
                self.0.events_ready.reset();

                Ok(byte)
            }
        }
    }

    /// Must only be called the same number of times as `read`.
    ///
    /// nRF51 is double buffered; two bytes can be written before data must be read.
    fn send(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        self.0.txd.write(|w| unsafe { w.bits(u32::from(byte)) });
        Ok(())
    }
}

impl<T> Spi<T>
where
    T: Instance,
{
    pub fn new(spi: T, pins: Pins, frequency: Frequency, mode: Mode) -> Self {
        // Select pins.
        let mut spi = spi;
        Self::set_pins(&mut spi, pins);

        // Enable SPI instance.
        spi.enable.write(|w| w.enable().enabled());

        // Configure mode.
        spi.config.write(|w| match mode {
            MODE_0 => w.order().msb_first().cpha().leading().cpol().active_high(),
            MODE_1 => w.order().msb_first().cpha().trailing().cpol().active_high(),
            MODE_2 => w.order().msb_first().cpha().leading().cpol().active_low(),
            MODE_3 => w.order().msb_first().cpha().trailing().cpol().active_low(),
        });

        // Configure frequency.
        spi.frequency.write(|w| w.frequency().variant(frequency));

        Self(spi)
    }

    #[cfg(feature = "51")]
    fn set_pins(spi: &mut T, pins: Pins) {
        spi.pselsck.write(|w| unsafe {
            if let Some(ref pin) = pins.sck {
                w.bits(pin.pin().into())
            } else {
                // Disconnect
                w.bits(0xFFFFFFFF)
            }
        });
        spi.pselmosi.write(|w| unsafe {
            if let Some(ref pin) = pins.mosi {
                w.bits(pin.pin().into())
            } else {
                // Disconnect
                w.bits(0xFFFFFFFF)
            }
        });

        spi.pselmiso.write(|w| unsafe {
            if let Some(ref pin) = pins.miso {
                w.bits(pin.pin().into())
            } else {
                // Disconnect
                w.bits(0xFFFFFFFF)
            }
        });
    }

    #[cfg(not(feature = "51"))]
    fn set_pins(spi: &mut T, pins: Pins) {
        if let Some(ref pin) = pins.sck {
            spi.psel.sck.write(|w| unsafe { w.bits(pin.pin().into()) });
        }
        if let Some(ref pin) = pins.mosi {
            spi.psel.mosi.write(|w| unsafe { w.bits(pin.pin().into()) });
        }
        if let Some(ref pin) = pins.miso {
            spi.psel.miso.write(|w| unsafe { w.bits(pin.pin().into()) });
        }
    }

    /// Return the raw interface to the underlying SPI peripheral.
    pub fn free(self) -> T {
        self.0
    }
}

/// GPIO pins for SPI interface.
pub struct Pins {
    /// SPI clock.
    ///
    /// None if unused.
    pub sck: Option<Pin<Output<PushPull>>>,

    /// MOSI Master out, slave in.
    ///
    /// None if unused.
    pub mosi: Option<Pin<Output<PushPull>>>,

    /// MISO Master in, slave out.
    ///
    /// None if unused.
    pub miso: Option<Pin<Input<Floating>>>,
}

#[derive(Debug)]
pub enum Error {
    Transmit,
    Receive,
}

/// Trait implemented by all SPI peripheral instances.
pub trait Instance: Deref<Target = spi0::RegisterBlock> + sealed::Sealed {}

mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for SPI0 {}
impl Instance for SPI0 {}

#[cfg(not(feature = "52810"))]
impl sealed::Sealed for crate::pac::SPI1 {}
#[cfg(not(feature = "52810"))]
impl Instance for crate::pac::SPI1 {}

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
impl sealed::Sealed for crate::pac::SPI2 {}
#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
impl Instance for crate::pac::SPI2 {}
