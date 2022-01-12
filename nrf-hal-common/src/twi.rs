//! HAL interface to the TWI peripheral.

use core::ops::Deref;

use crate::{
    gpio::{Floating, Input, Pin},
    pac::{twi0, GPIO, TWI0, TWI1},
};

pub use twi0::frequency::FREQUENCY_A as Frequency;

pub struct Twi<T>(T);

impl<T> Twi<T>
where
    T: Instance,
{
    pub fn new(twi: T, pins: Pins, frequency: Frequency) -> Self {
        // The TWIM peripheral requires the pins to be in a mode that is not
        // exposed through the GPIO API, and might it might not make sense to
        // expose it there.
        //
        // Until we've figured out what to do about this, let's just configure
        // the pins through the raw peripheral API. All of the following is
        // safe, as we own the pins now and have exclusive access to their
        // registers.
        for &pin in &[pins.scl.pin(), pins.sda.pin()] {
            unsafe { &*GPIO::ptr() }.pin_cnf[pin as usize].write(|w| {
                w.dir()
                    .input()
                    .input()
                    .connect()
                    .pull()
                    .pullup()
                    .drive()
                    .s0d1()
                    .sense()
                    .disabled()
            });
        }

        // Set pins.
        twi.pselscl
            .write(|w| unsafe { w.bits(pins.scl.pin().into()) });
        twi.pselsda
            .write(|w| unsafe { w.bits(pins.sda.pin().into()) });

        // Set frequency.
        twi.frequency.write(|w| w.frequency().variant(frequency));

        twi.enable.write(|w| w.enable().enabled());

        Self(twi)
    }

    fn send_byte(&self, byte: u8) -> Result<(), Error> {
        // Clear sent event.
        self.0.events_txdsent.write(|w| unsafe { w.bits(0) });

        // Copy data into the send buffer.
        self.0.txd.write(|w| unsafe { w.bits(u32::from(byte)) });

        // Wait until transmission was confirmed.
        while self.0.events_txdsent.read().bits() == 0 {
            // Bail out if we get an error instead.
            if self.0.events_error.read().bits() != 0 {
                self.0.events_error.write(|w| unsafe { w.bits(0) });
                return Err(Error::Transmit);
            }
        }

        // Clear sent event.
        self.0.events_txdsent.write(|w| unsafe { w.bits(0) });

        Ok(())
    }

    fn recv_byte(&self) -> Result<u8, Error> {
        // Wait until something ended up in the buffer.
        while self.0.events_rxdready.read().bits() == 0 {
            // Bail out if it's an error instead of data.
            if self.0.events_error.read().bits() != 0 {
                self.0.events_error.write(|w| unsafe { w.bits(0) });
                return Err(Error::Receive);
            }
        }

        // Read out data.
        let out = self.0.rxd.read().bits() as u8;

        // Clear reception event.
        self.0.events_rxdready.write(|w| unsafe { w.bits(0) });

        Ok(out)
    }

    fn send_stop(&self) -> Result<(), Error> {
        // Clear stopped event.
        self.0.events_stopped.write(|w| unsafe { w.bits(0) });

        // Start stop condition.
        self.0.tasks_stop.write(|w| unsafe { w.bits(1) });

        // Wait until stop was sent.
        while self.0.events_stopped.read().bits() == 0 {
            // Bail out if we get an error instead.
            if self.0.events_error.read().bits() != 0 {
                self.0.events_error.write(|w| unsafe { w.bits(0) });
                return Err(Error::Transmit);
            }
        }

        Ok(())
    }

    /// Write to an I2C slave.
    pub fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        // Make sure all previously used shortcuts are disabled.
        self.0
            .shorts
            .write(|w| w.bb_stop().disabled().bb_suspend().disabled());

        // Set Slave I2C address.
        self.0
            .address
            .write(|w| unsafe { w.address().bits(address.into()) });

        // Start data transmission.
        self.0.tasks_starttx.write(|w| unsafe { w.bits(1) });

        // Clock out all bytes.
        for byte in buffer {
            self.send_byte(*byte)?;
        }

        // Send stop.
        self.send_stop()?;
        Ok(())
    }

    /// Read from an I2C slave.
    pub fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        // Make sure all previously used shortcuts are disabled.
        self.0
            .shorts
            .write(|w| w.bb_stop().disabled().bb_suspend().disabled());

        // Set Slave I2C address.
        self.0
            .address
            .write(|w| unsafe { w.address().bits(address.into()) });

        // Read into buffer.
        if let Some((last, before)) = buffer.split_last_mut() {
            // If we want to read multiple bytes we need to use the suspend mode.
            if !before.is_empty() {
                self.0.shorts.write(|w| w.bb_suspend().enabled());
            } else {
                self.0.shorts.write(|w| w.bb_stop().enabled());
            }

            // Clear reception event.
            self.0.events_rxdready.write(|w| unsafe { w.bits(0) });

            // Start data reception.
            self.0.tasks_startrx.write(|w| unsafe { w.bits(1) });

            for byte in &mut before.into_iter() {
                self.0.tasks_resume.write(|w| unsafe { w.bits(1) });
                *byte = self.recv_byte()?;
            }

            self.0.shorts.write(|w| w.bb_stop().enabled());
            self.0.tasks_resume.write(|w| unsafe { w.bits(1) });
            *last = self.recv_byte()?;
        } else {
            self.send_stop()?;
        }
        Ok(())
    }

    /// Write data to an I2C slave, then read data from the slave without
    /// triggering a stop condition between the two.
    pub fn write_then_read(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
    ) -> Result<(), Error> {
        // Make sure all previously used shortcuts are disabled.
        self.0
            .shorts
            .write(|w| w.bb_stop().disabled().bb_suspend().disabled());

        // Set Slave I2C address.
        self.0
            .address
            .write(|w| unsafe { w.address().bits(address.into()) });

        // Start data transmission.
        self.0.tasks_starttx.write(|w| unsafe { w.bits(1) });

        // Send out all bytes in the outgoing buffer.
        for byte in wr_buffer {
            self.send_byte(*byte)?;
        }

        // Turn around to read data.
        if let Some((last, before)) = rd_buffer.split_last_mut() {
            // If we want to read multiple bytes we need to use the suspend mode.
            if !before.is_empty() {
                self.0.shorts.write(|w| w.bb_suspend().enabled());
            } else {
                self.0.shorts.write(|w| w.bb_stop().enabled());
            }

            // Clear reception event.
            self.0.events_rxdready.write(|w| unsafe { w.bits(0) });

            // Start data reception.
            self.0.tasks_startrx.write(|w| unsafe { w.bits(1) });

            for byte in &mut before.into_iter() {
                self.0.tasks_resume.write(|w| unsafe { w.bits(1) });
                *byte = self.recv_byte()?;
            }

            self.0.shorts.write(|w| w.bb_stop().enabled());
            self.0.tasks_resume.write(|w| unsafe { w.bits(1) });
            *last = self.recv_byte()?;
        } else {
            self.send_stop()?;
        }
        Ok(())
    }

    /// Return the raw interface to the underlying TWI peripheral.
    pub fn free(self) -> (T, Pins) {
        let scl = self.0.pselscl.read();
        let sda = self.0.pselsda.read();
        self.0.pselscl.reset();
        self.0.pselsda.reset();
        (
            self.0,
            Pins {
                scl: unsafe { Pin::from_psel_bits(scl.bits()) },
                sda: unsafe { Pin::from_psel_bits(sda.bits()) },
            },
        )
    }
}

impl<T> embedded_hal::blocking::i2c::Write for Twi<T>
where
    T: Instance,
{
    type Error = Error;

    fn write<'w>(&mut self, addr: u8, bytes: &'w [u8]) -> Result<(), Error> {
        self.write(addr, bytes)
    }
}

impl<T> embedded_hal::blocking::i2c::Read for Twi<T>
where
    T: Instance,
{
    type Error = Error;

    fn read<'w>(&mut self, addr: u8, bytes: &'w mut [u8]) -> Result<(), Error> {
        self.read(addr, bytes)
    }
}

impl<T> embedded_hal::blocking::i2c::WriteRead for Twi<T>
where
    T: Instance,
{
    type Error = Error;

    fn write_read<'w>(
        &mut self,
        addr: u8,
        bytes: &'w [u8],
        buffer: &'w mut [u8],
    ) -> Result<(), Error> {
        self.write_then_read(addr, bytes, buffer)
    }
}

/// The pins used by the TWI peripheral.
///
/// Currently, only P0 pins are supported.
pub struct Pins {
    // Serial Clock Line.
    pub scl: Pin<Input<Floating>>,

    // Serial Data Line.
    pub sda: Pin<Input<Floating>>,
}

#[derive(Debug)]
pub enum Error {
    Transmit,
    Receive,
}

/// Implemented by all TWIM instances.
pub trait Instance: Deref<Target = twi0::RegisterBlock> + sealed::Sealed {}

mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for TWI0 {}
impl Instance for TWI0 {}

impl sealed::Sealed for TWI1 {}
impl Instance for TWI1 {}
