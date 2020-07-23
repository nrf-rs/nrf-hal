//! HAL interface to the UARTE peripheral.
//!
//! See product specification:
//!
//! - nrf52832: Section 35
//! - nrf52840: Section 6.34
use core::fmt;
use core::ops::Deref;
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};

use embedded_hal::digital::v2::OutputPin;

#[cfg(any(feature = "52833", feature = "52840"))]
use crate::pac::UARTE1;

#[cfg(feature = "9160")]
use crate::pac::{uarte0_ns as uarte0, UARTE0_NS as UARTE0, UARTE1_NS as UARTE1};

#[cfg(not(feature = "9160"))]
use crate::pac::{uarte0, UARTE0};

use crate::pac::Interrupt;

use crate::gpio::{Floating, Input, Output, Pin, PushPull};
use crate::prelude::*;
use crate::slice_in_ram_or;
use crate::target_constants::EASY_DMA_SIZE;
use crate::timer::{self, Timer};

// Re-export SVD variants to allow user to directly set values.
pub use uarte0::{baudrate::BAUDRATE_A as Baudrate, config::PARITY_A as Parity};

/// Interface to a UARTE instance.
///
/// This is a very basic interface that comes with the following limitations:
/// - The UARTE instances share the same address space with instances of UART.
///   You need to make sure that conflicting instances
///   are disabled before using `Uarte`. See product specification:
///     - nrf52832: Section 15.2
///     - nrf52840: Section 6.1.2
pub struct Uarte<T: Instance> {
    inner: LowLevelUarte<T>,
}

impl<T> Uarte<T>
where
    T: Instance,
{
    pub fn new(uarte: T, pins: Pins, parity: Parity, baudrate: Baudrate) -> Self {
        let ll_uarte = LowLevelUarte { uarte };
        ll_uarte.setup(pins, parity, baudrate);
        Uarte { inner: ll_uarte }
    }

    /// Write via UARTE.
    ///
    /// This method uses transmits all bytes in `tx_buffer`.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn write(&mut self, tx_buffer: &[u8]) -> Result<(), Error> {
        unsafe {
            self.inner.start_write(tx_buffer)?;
        }

        // Wait for transmission to end.
        let mut endtx;
        let mut txstopped;
        loop {
            endtx = self.inner.uarte.events_endtx.read().bits() != 0;
            txstopped = self.inner.uarte.events_txstopped.read().bits() != 0;
            if endtx || txstopped {
                break;
            }
        }

        if txstopped {
            return Err(Error::Transmit);
        }

        self.inner.stop_write();

        Ok(())
    }

    /// Read via UARTE.
    ///
    /// This method fills all bytes in `rx_buffer`, and blocks
    /// until the buffer is full.
    ///
    /// The buffer must have a length of at most 255 bytes.
    pub fn read(&mut self, rx_buffer: &mut [u8]) -> Result<(), Error> {
        unsafe {
            self.inner.start_read(rx_buffer)?;
        }

        // Wait for transmission to end.
        while self.inner.uarte.events_endrx.read().bits() == 0 {}

        self.inner.finalize_read();

        if self.inner.uarte.rxd.amount.read().bits() != rx_buffer.len() as u32 {
            return Err(Error::Receive);
        }

        Ok(())
    }

    /// Read via UARTE.
    ///
    /// This method fills all bytes in `rx_buffer`, and blocks
    /// until the buffer is full or the timeout expires, whichever
    /// comes first.
    ///
    /// If the timeout occurs, an `Error::Timeout(n)` will be returned,
    /// where `n` is the number of bytes read successfully.
    ///
    /// This method assumes the interrupt for the given timer is NOT enabled,
    /// and in cases where a timeout does NOT occur, the timer will be left running
    /// until completion.
    ///
    /// The buffer must have a length of at most 255 bytes.
    pub fn read_timeout<I>(
        &mut self,
        rx_buffer: &mut [u8],
        timer: &mut Timer<I>,
        cycles: u32,
    ) -> Result<(), Error>
    where
        I: timer::Instance,
    {
        // Start the read.
        unsafe {
            self.inner.start_read(rx_buffer)?;
        }

        // Start the timeout timer.
        timer.start(cycles);

        // Wait for transmission to end.
        let mut event_complete = false;
        let mut timeout_occured = false;

        loop {
            event_complete |= self.inner.uarte.events_endrx.read().bits() != 0;
            timeout_occured |= timer.wait().is_ok();
            if event_complete || timeout_occured {
                break;
            }
        }

        if !event_complete {
            // Cancel the reception if it did not complete until now.
            self.inner.cancel_read();
        }

        // Cleanup, even in the error case.
        self.inner.finalize_read();

        let bytes_read = self.inner.uarte.rxd.amount.read().bits() as usize;

        if timeout_occured && !event_complete {
            return Err(Error::Timeout(bytes_read));
        }

        if bytes_read != rx_buffer.len() as usize {
            return Err(Error::Receive);
        }

        Ok(())
    }

    /// Return the raw interface to the underlying UARTE peripheral.
    pub fn free(self) -> T {
        self.inner.uarte
    }
}

impl<T> fmt::Write for Uarte<T>
where
    T: Instance,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Copy all data into an on-stack buffer so we never try to EasyDMA from
        // flash.
        let buf = &mut [0; 16][..];
        for block in s.as_bytes().chunks(16) {
            buf[..block.len()].copy_from_slice(block);
            self.write(&buf[..block.len()]).map_err(|_| fmt::Error)?;
        }

        Ok(())
    }
}

pub struct Pins {
    pub rxd: Pin<Input<Floating>>,
    pub txd: Pin<Output<PushPull>>,
    pub cts: Option<Pin<Input<Floating>>>,
    pub rts: Option<Pin<Output<PushPull>>>,
}

#[derive(Debug)]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    Transmit,
    Receive,
    Timeout(usize),
    BufferNotInRAM,
}

/// This is a low level wrapper around the PAC uarte.
///
/// It is provided to allow for low level helper functions that are
/// use by the HAL, or could be used to implement alternative HAL functionality.
///
/// In general, you should prefer to use `Uarte` when interfacing with the HAL,
/// as it takes care of safety guarantees not handled by these low level interfaces.
/// Failure to operate these interfaces correctly may leave the hardware in an
/// unexpected or inconsistent state.
pub struct LowLevelUarte<T: Instance> {
    pub uarte: T,
}

impl<T> LowLevelUarte<T>
where
    T: Instance,
{
    /// Start a UARTE read transaction by setting the control
    /// values and triggering a read task
    ///
    /// SAFETY: `rx_buffer` must live until the read transaction is complete, and must
    /// not be accessed between the call to read and the transaction is complete.
    pub unsafe fn start_read(&self, rx_buffer: &mut [u8]) -> Result<(), Error> {
        // This is overly restrictive. See (similar SPIM issue):
        // https://github.com/nrf-rs/nrf52/issues/17
        if rx_buffer.len() > u8::max_value() as usize {
            return Err(Error::RxBufferTooLong);
        }

        // NOTE: RAM slice check is not necessary, as a mutable slice can only be
        // built from data located in RAM

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        compiler_fence(SeqCst);

        // Set up the DMA read
        self.uarte.rxd.ptr.write(|w|
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the UARTE transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            w.ptr().bits(rx_buffer.as_ptr() as u32));
        self.uarte.rxd.maxcnt.write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to `u8` is also fine.
            //
            // The MAXCNT field is at least 8 bits wide and accepts the full
            // range of values.
            w.maxcnt().bits(rx_buffer.len() as _));

        // Start UARTE Receive transaction
        self.uarte.tasks_startrx.write(|w|
            // `1` is a valid value to write to task registers.
            w.bits(1));

        Ok(())
    }

    // Start a write transaction
    //
    // SAFETY: `tx_buffer` must live long enough for this transaction to complete
    pub unsafe fn start_write(&self, tx_buffer: &[u8]) -> Result<(), Error> {
        if tx_buffer.len() > EASY_DMA_SIZE {
            return Err(Error::TxBufferTooLong);
        }

        // We can only DMA out of RAM.
        slice_in_ram_or(tx_buffer, Error::BufferNotInRAM)?;

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        compiler_fence(SeqCst);

        // Reset the events.
        self.uarte.events_endtx.reset();
        self.uarte.events_txstopped.reset();

        // Set up the DMA write
        self.uarte.txd.ptr.write(|w|
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the UARTE transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            w.ptr().bits(tx_buffer.as_ptr() as u32));
        self.uarte.txd.maxcnt.write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to `u8` is also fine.
            //
            // The MAXCNT field is 8 bits wide and accepts the full range of
            // values.
            w.maxcnt().bits(tx_buffer.len() as _));

        // Start UARTE Transmit transaction
        self.uarte.tasks_starttx.write(|w|
            // `1` is a valid value to write to task registers.
            w.bits(1));

        Ok(())
    }

    pub(crate) fn setup(&self, mut pins: Pins, parity: Parity, baudrate: Baudrate) {
        // Select pins
        self.uarte.psel.rxd.write(|w| {
            let w = unsafe { w.pin().bits(pins.rxd.pin()) };
            #[cfg(feature = "52840")]
            let w = w.port().bit(pins.rxd.port().bit());
            w.connect().connected()
        });
        pins.txd.set_high().unwrap();
        self.uarte.psel.txd.write(|w| {
            let w = unsafe { w.pin().bits(pins.txd.pin()) };
            #[cfg(feature = "52840")]
            let w = w.port().bit(pins.txd.port().bit());
            w.connect().connected()
        });

        // Optional pins
        self.uarte.psel.cts.write(|w| {
            if let Some(ref pin) = pins.cts {
                let w = unsafe { w.pin().bits(pin.pin()) };
                #[cfg(feature = "52840")]
                let w = w.port().bit(pin.port().bit());
                w.connect().connected()
            } else {
                w.connect().disconnected()
            }
        });

        self.uarte.psel.rts.write(|w| {
            if let Some(ref pin) = pins.rts {
                let w = unsafe { w.pin().bits(pin.pin()) };
                #[cfg(feature = "52840")]
                let w = w.port().bit(pin.port().bit());
                w.connect().connected()
            } else {
                w.connect().disconnected()
            }
        });

        // Enable UARTE instance
        self.uarte.enable.write(|w| w.enable().enabled());

        // Configure
        let hardware_flow_control = pins.rts.is_some() && pins.cts.is_some();
        self.uarte
            .config
            .write(|w| w.hwfc().bit(hardware_flow_control).parity().variant(parity));

        // Configure frequency
        self.uarte
            .baudrate
            .write(|w| w.baudrate().variant(baudrate));
    }

    /// Stop an unfinished UART read transaction and flush FIFO to DMA buffer
    pub(crate) fn cancel_read(&self) {
        self.uarte.events_rxto.write(|w| w);

        // Stop reception
        self.uarte.tasks_stoprx.write(|w| unsafe { w.bits(1) });

        // Wait for the reception to have stopped
        while self.uarte.events_rxto.read().bits() == 0 {}

        // Reset the event flag
        self.uarte.events_rxto.write(|w| w);

        // Ask UART to flush FIFO to DMA buffer
        self.uarte.tasks_flushrx.write(|w| unsafe { w.bits(1) });

        // Wait for the flush to complete.
        while self.uarte.events_endrx.read().bits() == 0 {}

        // The event flag itself is later reset by `finalize_read`.
    }

    /// Finalize a UARTE read transaction by clearing the event.
    pub fn finalize_read(&self) {
        // Reset the event, otherwise it will always read `1` from now on.
        self.uarte.events_endrx.write(|w| w);

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed.
        compiler_fence(SeqCst);
    }

    /// Stop a UARTE write transaction
    pub fn stop_write(&self) {
        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed.
        compiler_fence(SeqCst);

        // Lower power consumption by disabling the transmitter once we're
        // finished.
        self.uarte.tasks_stoptx.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) });
    }
}

pub trait Instance: Deref<Target = uarte0::RegisterBlock> {
    const INTERRUPT: Interrupt;
}

impl Instance for UARTE0 {
    #[cfg(not(feature = "9160"))]
    const INTERRUPT: Interrupt = Interrupt::UARTE0_UART0;

    #[cfg(feature = "9160")]
    const INTERRUPT: Interrupt = Interrupt::UARTE0_SPIM0_SPIS0_TWIM0_TWIS0;
}

#[cfg(any(feature = "52833", feature = "52840", feature = "9160"))]
impl Instance for UARTE1 {
    #[cfg(not(feature = "9160"))]
    const INTERRUPT: Interrupt = Interrupt::UARTE1;

    #[cfg(feature = "9160")]
    const INTERRUPT: Interrupt = Interrupt::UARTE1_SPIM1_SPIS1_TWIM1_TWIS1;
}
