//! HAL interface to the TWIM peripheral.
//!
//! See product specification:
//!
//! - nRF52832: Section 33
//! - nRF52840: Section 6.31
use core::ops::Deref;
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};

#[cfg(any(feature = "9160", feature = "5340-app", feature = "5340-net"))]
// use crate::pac::{
//     twim0_ns as twim0, TWIM0_NS as TWIM0, TWIM1_NS as TWIM1, TWIM2_NS as TWIM2, TWIM3_NS as TWIM3 };
use crate::pac::{twim0_ns as twim0, TWIM0_NS as TWIM0};

#[cfg(any(feature = "9160", feature = "5340-app"))]
use crate::pac::{TWIM1_NS as TWIM1, TWIM2_NS as TWIM2, TWIM3_NS as TWIM3};

#[cfg(not(any(feature = "9160", feature = "5340-app", feature = "5340-net")))]
use crate::pac::{twim0, TWIM0};

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
use crate::pac::TWIM1;

use crate::{
    gpio::{Floating, Input, Pin},
    slice_in_ram, slice_in_ram_or,
    target_constants::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE},
};

pub use twim0::frequency::FREQUENCY_A as Frequency;

/// Interface to a TWIM instance.
///
/// This is a very basic interface that comes with the following limitation:
/// The TWIM instances share the same address space with instances of SPIM,
/// SPIS, SPI, TWIS, and TWI. For example, TWIM0 conflicts with SPIM0, SPIS0,
/// etc.; TWIM1 conflicts with SPIM1, SPIS1, etc. You need to make sure that
/// conflicting instances are disabled before using `Twim`. Please refer to the
/// product specification for more information (section 15.2 for nRF52832,
/// section 6.1.2 for nRF52840).
pub struct Twim<T>(T);

impl<T> Twim<T>
where
    T: Instance,
{
    pub fn new(twim: T, pins: Pins, frequency: Frequency) -> Self {
        // The TWIM peripheral requires the pins to be in a mode that is not
        // exposed through the GPIO API, and might it might not make sense to
        // expose it there.
        //
        // Until we've figured out what to do about this, let's just configure
        // the pins through the raw peripheral API. All of the following is
        // safe, as we own the pins now and have exclusive access to their
        // registers.
        for &pin in &[&pins.scl, &pins.sda] {
            pin.conf().write(|w| {
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

        // Select pins.
        twim.psel.scl.write(|w| {
            unsafe { w.bits(pins.scl.psel_bits()) };
            w.connect().connected()
        });
        twim.psel.sda.write(|w| {
            unsafe { w.bits(pins.sda.psel_bits()) };
            w.connect().connected()
        });

        // Enable TWIM instance.
        twim.enable.write(|w| w.enable().enabled());

        // Configure frequency.
        twim.frequency.write(|w| w.frequency().variant(frequency));

        Twim(twim)
    }

    /// Disable the instance.
    ///
    /// Disabling the instance will switch off the peripheral leading to a
    /// considerably lower energy use. However, while the instance is disabled
    /// it is not possible to use it for communication. The configuration of
    /// the instance will be retained.
    pub fn disable(&mut self) {
        self.0.enable.write(|w| w.enable().disabled());
    }

    /// Re-enable the instance after it was previously disabled.
    pub fn enable(&mut self) {
        self.0.enable.write(|w| w.enable().enabled());
    }

    /// Set TX buffer, checking that it is in RAM and has suitable length.
    unsafe fn set_tx_buffer(&mut self, buffer: &[u8]) -> Result<(), Error> {
        slice_in_ram_or(buffer, Error::DMABufferNotInDataMemory)?;

        if buffer.len() == 0 {
            return Err(Error::TxBufferZeroLength);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::TxBufferTooLong);
        }

        self.0.txd.ptr.write(|w|
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the I2C transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            w.ptr().bits(buffer.as_ptr() as u32));
        self.0.txd.maxcnt.write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to `u8` is also fine.
            //
            // The MAXCNT field is 8 bits wide and accepts the full range of
            // values.
            w.maxcnt().bits(buffer.len() as _));

        Ok(())
    }

    /// Set RX buffer, checking that it has suitable length.
    unsafe fn set_rx_buffer(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        // NOTE: RAM slice check is not necessary, as a mutable
        // slice can only be built from data located in RAM.

        if buffer.len() == 0 {
            return Err(Error::RxBufferZeroLength);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::RxBufferTooLong);
        }

        self.0.rxd.ptr.write(|w|
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the I2C transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            w.ptr().bits(buffer.as_mut_ptr() as u32));
        self.0.rxd.maxcnt.write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to the type of maxcnt
            // is also fine.
            //
            // Note that that nrf52840 maxcnt is a wider
            // type than a u8, so we use a `_` cast rather than a `u8` cast.
            // The MAXCNT field is thus at least 8 bits wide and accepts the
            // full range of values that fit in a `u8`.
            w.maxcnt().bits(buffer.len() as _));

        Ok(())
    }

    fn clear_errorsrc(&mut self) {
        self.0
            .errorsrc
            .write(|w| w.anack().bit(true).dnack().bit(true).overrun().bit(true));
    }

    /// Get Error instance, if any occurred.
    fn read_errorsrc(&self) -> Result<(), Error> {
        let err = self.0.errorsrc.read();
        if err.anack().is_received() {
            return Err(Error::AddressNack);
        }
        if err.dnack().is_received() {
            return Err(Error::DataNack);
        }
        if err.overrun().is_received() {
            return Err(Error::DataNack);
        }
        Ok(())
    }

    /// Wait for stop or error
    fn wait(&mut self) {
        loop {
            if self.0.events_stopped.read().bits() != 0 {
                self.0.events_stopped.reset();
                break;
            }
            if self.0.events_error.read().bits() != 0 {
                self.0.events_error.reset();
                self.0.tasks_stop.write(|w| unsafe { w.bits(1) });
            }
        }
    }

    /// Write to an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started.
        compiler_fence(SeqCst);

        self.0
            .address
            .write(|w| unsafe { w.address().bits(address) });

        // Set up the DMA write.
        unsafe { self.set_tx_buffer(buffer)? };

        // Clear events
        self.0.events_stopped.reset();
        self.0.events_error.reset();
        self.0.events_lasttx.reset();
        self.clear_errorsrc();

        // Start write operation.
        self.0.shorts.write(|w| w.lasttx_stop().enabled());
        self.0.tasks_starttx.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) });

        self.wait();

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed.
        compiler_fence(SeqCst);

        self.read_errorsrc()?;

        if self.0.txd.amount.read().bits() != buffer.len() as u32 {
            return Err(Error::Transmit);
        }

        Ok(())
    }

    /// Read from an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started.
        compiler_fence(SeqCst);

        self.0
            .address
            .write(|w| unsafe { w.address().bits(address) });

        // Set up the DMA read.
        unsafe { self.set_rx_buffer(buffer)? };

        // Clear events
        self.0.events_stopped.reset();
        self.0.events_error.reset();
        self.clear_errorsrc();

        // Start read operation.
        self.0.shorts.write(|w| w.lastrx_stop().enabled());
        self.0.tasks_startrx.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) });

        self.wait();

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed.
        compiler_fence(SeqCst);

        self.read_errorsrc()?;

        if self.0.rxd.amount.read().bits() != buffer.len() as u32 {
            return Err(Error::Receive);
        }

        Ok(())
    }

    /// Write data to an I2C slave, then read data from the slave without
    /// triggering a stop condition between the two.
    ///
    /// The buffers must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn write_then_read(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
    ) -> Result<(), Error> {
        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started.
        compiler_fence(SeqCst);

        self.0
            .address
            .write(|w| unsafe { w.address().bits(address) });

        // Set up DMA buffers.
        unsafe {
            self.set_tx_buffer(wr_buffer)?;
            self.set_rx_buffer(rd_buffer)?;
        }

        // Clear events
        self.0.events_stopped.reset();
        self.0.events_error.reset();
        self.clear_errorsrc();

        // Start write+read operation.
        self.0.shorts.write(|w| {
            w.lasttx_startrx().enabled();
            w.lastrx_stop().enabled();
            w
        });
        // `1` is a valid value to write to task registers.
        self.0.tasks_starttx.write(|w| unsafe { w.bits(1) });

        self.wait();

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed.
        compiler_fence(SeqCst);

        self.read_errorsrc()?;

        let bad_write = self.0.txd.amount.read().bits() != wr_buffer.len() as u32;
        let bad_read = self.0.rxd.amount.read().bits() != rd_buffer.len() as u32;

        if bad_write {
            return Err(Error::Transmit);
        }

        if bad_read {
            return Err(Error::Receive);
        }

        Ok(())
    }

    /// Copy data into RAM and write to an I2C slave, then read data from the slave without
    /// triggering a stop condition between the two.
    ///
    /// The write buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 1024 bytes on the nRF52840.
    ///
    /// The read buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn copy_write_then_read(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
    ) -> Result<(), Error> {
        if wr_buffer.len() > FORCE_COPY_BUFFER_SIZE {
            return Err(Error::TxBufferTooLong);
        }

        // Copy to RAM
        let wr_ram_buffer = &mut [0; FORCE_COPY_BUFFER_SIZE][..wr_buffer.len()];
        wr_ram_buffer.copy_from_slice(wr_buffer);

        self.write_then_read(address, wr_ram_buffer, rd_buffer)
    }

    /// Return the raw interface to the underlying TWIM peripheral.
    pub fn free(self) -> (T, Pins) {
        let scl = self.0.psel.scl.read();
        let sda = self.0.psel.sda.read();
        self.0.psel.scl.reset();
        self.0.psel.sda.reset();
        (
            self.0,
            Pins {
                scl: unsafe { Pin::from_psel_bits(scl.bits()) },
                sda: unsafe { Pin::from_psel_bits(sda.bits()) },
            },
        )
    }
}

impl<T> embedded_hal::blocking::i2c::Write for Twim<T>
where
    T: Instance,
{
    type Error = Error;

    fn write<'w>(&mut self, addr: u8, bytes: &'w [u8]) -> Result<(), Error> {
        if slice_in_ram(bytes) {
            self.write(addr, bytes)
        } else {
            let buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..];
            for chunk in bytes.chunks(FORCE_COPY_BUFFER_SIZE) {
                buf[..chunk.len()].copy_from_slice(chunk);
                self.write(addr, &buf[..chunk.len()])?;
            }
            Ok(())
        }
    }
}

impl<T> embedded_hal::blocking::i2c::Read for Twim<T>
where
    T: Instance,
{
    type Error = Error;

    fn read<'w>(&mut self, addr: u8, bytes: &'w mut [u8]) -> Result<(), Error> {
        self.read(addr, bytes)
    }
}

impl<T> embedded_hal::blocking::i2c::WriteRead for Twim<T>
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
        if slice_in_ram(bytes) {
            self.write_then_read(addr, bytes, buffer)
        } else {
            self.copy_write_then_read(addr, bytes, buffer)
        }
    }
}

/// The pins used by the TWIM peripheral.
///
/// Currently, only P0 pins are supported.
pub struct Pins {
    // Serial Clock Line.
    pub scl: Pin<Input<Floating>>,

    // Serial Data Line.
    pub sda: Pin<Input<Floating>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    TxBufferZeroLength,
    RxBufferZeroLength,
    Transmit,
    Receive,
    DMABufferNotInDataMemory,
    AddressNack,
    DataNack,
    Overrun,
}

/// Implemented by all TWIM instances
pub trait Instance: Deref<Target = twim0::RegisterBlock> + sealed::Sealed {}

mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for TWIM0 {}
impl Instance for TWIM0 {}

#[cfg(any(
    feature = "52832",
    feature = "52833",
    feature = "52840",
    feature = "9160",
    feature = "5340-app",
))]
mod _twim1 {
    use super::*;
    impl sealed::Sealed for TWIM1 {}
    impl Instance for TWIM1 {}
}

#[cfg(any(feature = "9160", feature = "5340-app"))]
mod _twim2 {
    use super::*;
    impl sealed::Sealed for TWIM2 {}
    impl Instance for TWIM2 {}
}

#[cfg(any(feature = "9160", feature = "5340-app"))]
mod _twim3 {
    use super::*;
    impl sealed::Sealed for TWIM3 {}
    impl Instance for TWIM3 {}
}
