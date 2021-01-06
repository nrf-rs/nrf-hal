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
pub struct Uarte<T>(T);

impl<T> Uarte<T>
where
    T: Instance,
{
    pub fn new(uarte: T, mut pins: Pins, parity: Parity, baudrate: Baudrate) -> Self {
        // Is the UART already on? It might be if you had a bootloader
        if uarte.enable.read().bits() != 0 {
            uarte.tasks_stoptx.write(|w| unsafe { w.bits(1) });
            while uarte.events_txstopped.read().bits() == 0 {
                // Spin
            }
    
            // Disable UARTE instance
            uarte.enable.write(|w| w.enable().disabled());
        }

        // Select pins
        uarte.psel.rxd.write(|w| {
            unsafe { w.bits(pins.rxd.psel_bits()) };
            w.connect().connected()
        });
        pins.txd.set_high().unwrap();
        uarte.psel.txd.write(|w| {
            unsafe { w.bits(pins.txd.psel_bits()) };
            w.connect().connected()
        });

        // Optional pins
        uarte.psel.cts.write(|w| {
            if let Some(ref pin) = pins.cts {
                unsafe { w.bits(pin.psel_bits()) };
                w.connect().connected()
            } else {
                w.connect().disconnected()
            }
        });

        uarte.psel.rts.write(|w| {
            if let Some(ref pin) = pins.rts {
                unsafe { w.bits(pin.psel_bits()) };
                w.connect().connected()
            } else {
                w.connect().disconnected()
            }
        });

        // Configure.
        let hardware_flow_control = pins.rts.is_some() && pins.cts.is_some();
        uarte
            .config
            .write(|w| w.hwfc().bit(hardware_flow_control).parity().variant(parity));

        // Configure frequency.
        uarte.baudrate.write(|w| w.baudrate().variant(baudrate));
        
        let mut u = Uarte(uarte);
        
        u.apply_workaround_for_enable_anomaly();

        // Enable UARTE instance.
        u.0.enable.write(|w| w.enable().enabled());

        u
    }

    #[cfg(not(any(feature = "9160", feature = "5340")))]
    fn apply_workaround_for_enable_anomaly(&mut self)
    {
        // Do nothing
    }

    #[cfg(any(feature = "9160", feature = "5340"))]
    fn apply_workaround_for_enable_anomaly(&mut self)
    {
        // Apply workaround for anomalies:
        // - nRF9160 - anomaly 23
        // - nRF5340 - anomaly 44
        let rxenable_reg: *const u32 = ((self.0.deref() as *const _ as usize) + 0x564) as *const u32;
        let txenable_reg: *const u32 = ((self.0.deref() as *const _ as usize) + 0x568) as *const u32;

        // NB Safety: This is taken from Nordic's driver -
        // https://github.com/NordicSemiconductor/nrfx/blob/master/drivers/src/nrfx_uarte.c#L197
        if unsafe { core::ptr::read_volatile(txenable_reg) } == 1 {
            self.0.tasks_stoptx.write(|w| unsafe { w.bits(1) });
        }

        // NB Safety: This is taken from Nordic's driver -
        // https://github.com/NordicSemiconductor/nrfx/blob/master/drivers/src/nrfx_uarte.c#L197
        if unsafe { core::ptr::read_volatile(rxenable_reg) } == 1 {
            self.0.enable.write(|w| w.enable().enabled());
            self.0.tasks_stoprx.write(|w| unsafe { w.bits(1) });


            let mut workaround_succeded = false;
            // The UARTE is able to receive up to four bytes after the STOPRX task has been triggered.
            // On lowest supported baud rate (1200 baud), with parity bit and two stop bits configured
            // (resulting in 12 bits per data byte sent), this may take up to 40 ms.
            for _ in 0..40000 {
                // NB Safety: This is taken from Nordic's driver -
                // https://github.com/NordicSemiconductor/nrfx/blob/master/drivers/src/nrfx_uarte.c#L197
                if unsafe { core::ptr::read_volatile(rxenable_reg) } == 0 {
                    workaround_succeded = true;
                    break;
                }
                else
                {
                    // Need to sleep for 1us here
                }
            }

            if !workaround_succeded
            {
                panic!("Failed to apply workaround for UART");
            }

            let errors = self.0.errorsrc.read().bits();
            // NB Safety: safe to write back the bits we just read to clear them
            self.0.errorsrc.write(|w| unsafe { w.bits(errors) }); 
            self.0.enable.write(|w| w.enable().disabled());
        }
    }

    /// Write via UARTE.
    ///
    /// This method uses transmits all bytes in `tx_buffer`.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn write(&mut self, tx_buffer: &[u8]) -> Result<(), Error> {
        if tx_buffer.len() > EASY_DMA_SIZE {
            return Err(Error::TxBufferTooLong);
        }

        // We can only DMA out of RAM.
        slice_in_ram_or(tx_buffer, Error::BufferNotInRAM)?;

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started.
        compiler_fence(SeqCst);

        
        // Set up the DMA write.
        self.0.txd.ptr.write(|w|
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the UARTE transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            unsafe { w.ptr().bits(tx_buffer.as_ptr() as u32) });
        self.0.txd.maxcnt.write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to `u8` is also fine.
            //
            // The MAXCNT field is 8 bits wide and accepts the full range of
            // values.
            unsafe { w.maxcnt().bits(tx_buffer.len() as _) });

        // Reset the event
        self.0.events_endtx.reset();

        // Start UARTE Transmit transaction.
        self.0.tasks_starttx.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) });

                   
        // Wait for transmission to end.
        while self.0.events_endtx.read().bits() == 0 {
            // TODO: Do something here which uses less power. Like `wfi`.
        }

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed.
        compiler_fence(SeqCst);

        // Reset the event
        self.0.events_txstopped.reset();

        // Lower power consumption by disabling the transmitter once we're
        // finished.
        self.0.tasks_stoptx.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) });

        // Wait for transmitter to stop.
        while self.0.events_txstopped.read().bits() == 0 {
            // Spin
        }

        Ok(())
    }

    /// Read via UARTE.
    ///
    /// This method fills all bytes in `rx_buffer`, and blocks
    /// until the buffer is full.
    ///
    /// The buffer must have a length of at most 255 bytes.
    pub fn read(&mut self, rx_buffer: &mut [u8]) -> Result<(), Error> {
        self.start_read(rx_buffer)?;

        // Wait for transmission to end.
        while self.0.events_endrx.read().bits() == 0 {}

        self.finalize_read();

        if self.0.rxd.amount.read().bits() != rx_buffer.len() as u32 {
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
        self.start_read(rx_buffer)?;

        // Start the timeout timer.
        timer.start(cycles);

        // Wait for transmission to end.
        let mut event_complete = false;
        let mut timeout_occured = false;

        loop {
            event_complete |= self.0.events_endrx.read().bits() != 0;
            timeout_occured |= timer.wait().is_ok();
            if event_complete || timeout_occured {
                break;
            }
        }

        if !event_complete {
            // Cancel the reception if it did not complete until now.
            self.cancel_read();
        }

        // Cleanup, even in the error case.
        self.finalize_read();

        let bytes_read = self.0.rxd.amount.read().bits() as usize;

        if timeout_occured && !event_complete {
            return Err(Error::Timeout(bytes_read));
        }

        if bytes_read != rx_buffer.len() as usize {
            return Err(Error::Receive);
        }

        Ok(())
    }

    /// Start a UARTE read transaction by setting the control
    /// values and triggering a read task.
    fn start_read(&mut self, rx_buffer: &mut [u8]) -> Result<(), Error> {
        if rx_buffer.len() > EASY_DMA_SIZE {
            return Err(Error::RxBufferTooLong);
        }

        // NOTE: RAM slice check is not necessary, as a mutable slice can only be
        // built from data located in RAM.

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started.
        compiler_fence(SeqCst);

        // Set up the DMA read
        self.0.rxd.ptr.write(|w|
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the UARTE transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            unsafe { w.ptr().bits(rx_buffer.as_ptr() as u32) });
        self.0.rxd.maxcnt.write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to `u8` is also fine.
            //
            // The MAXCNT field is at least 8 bits wide and accepts the full
            // range of values.
            unsafe { w.maxcnt().bits(rx_buffer.len() as _) });

        // Start UARTE Receive transaction.
        self.0.tasks_startrx.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) });

        Ok(())
    }

    /// Finalize a UARTE read transaction by clearing the event.
    fn finalize_read(&mut self) {
        // Reset the event, otherwise it will always read `1` from now on.
        self.0.events_endrx.write(|w| w);

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed.
        compiler_fence(SeqCst);
    }

    /// Stop an unfinished UART read transaction and flush FIFO to DMA buffer.
    fn cancel_read(&mut self) {
        // Stop reception.
        self.0.tasks_stoprx.write(|w| unsafe { w.bits(1) });

        // Wait for the reception to have stopped.
        while self.0.events_rxto.read().bits() == 0 {}

        // Reset the event flag.
        self.0.events_rxto.write(|w| w);

        // Ask UART to flush FIFO to DMA buffer.
        self.0.tasks_flushrx.write(|w| unsafe { w.bits(1) });

        // Wait for the flush to complete.
        while self.0.events_endrx.read().bits() == 0 {}

        // The event flag itself is later reset by `finalize_read`.
    }

    /// Return the raw interface to the underlying UARTE peripheral.
    pub fn free(self) -> (T, Pins) {
        let rxd = self.0.psel.rxd.read();
        let txd = self.0.psel.txd.read();
        let cts = self.0.psel.cts.read();
        let rts = self.0.psel.rts.read();
        (
            self.0,
            Pins {
                rxd: unsafe { Pin::from_psel_bits(rxd.bits()) },
                txd: unsafe { Pin::from_psel_bits(txd.bits()) },
                cts: if cts.connect().bit_is_set() {
                    Some(unsafe { Pin::from_psel_bits(cts.bits()) })
                } else {
                    None
                },
                rts: if rts.connect().bit_is_set() {
                    Some(unsafe { Pin::from_psel_bits(rts.bits()) })
                } else {
                    None
                },
            },
        )
    }

    // Split into implementations of embedded_hal::serial traits
    pub fn split(self) -> (UarteTx<T>, UarteRx<T>) {
        let tx = UarteTx::new();
        let rx = UarteRx::new();
        (tx, rx)
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

pub trait Instance: Deref<Target = uarte0::RegisterBlock> + sealed::Sealed {
    fn ptr() -> *const uarte0::RegisterBlock;
}

mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for UARTE0 {}
impl Instance for UARTE0 {
    fn ptr() -> *const uarte0::RegisterBlock {
        UARTE0::ptr()
    }
}

#[cfg(any(feature = "52833", feature = "52840", feature = "9160"))]
mod _uarte1 {
    use super::*;
    impl sealed::Sealed for UARTE1 {}
    impl Instance for UARTE1 {
        fn ptr() -> *const uarte0::RegisterBlock {
            UARTE1::ptr()
        }
    }
}

/// Interface for the TX part of a UART instance that can be used independently of the RX part.
pub struct UarteTx<T> {
    _marker: core::marker::PhantomData<T>,
    tx_buf: [u8; 1],
}

/// Interface for the RX part of a UART instance that can be used independently of the TX part.
pub struct UarteRx<T> {
    _marker: core::marker::PhantomData<T>,
    rx_buf: [u8; 1],
}

impl<T> UarteTx<T>
where
    T: Instance,
{
    fn new() -> UarteTx<T> {
        let tx = UarteTx {
            _marker: core::marker::PhantomData,
            tx_buf: [0; 1],
        };
        tx
    }
}

impl<T> UarteRx<T>
where
    T: Instance,
{
    fn new() -> UarteRx<T> {
        let rx = UarteRx {
            _marker: core::marker::PhantomData,
            rx_buf: [0; 1],
        };
        rx
    }
}

pub mod serial {

    ///! Implementation of the embedded_hal::serial::* traits for UartTx and UartRx.
    use super::*;
    use embedded_hal::serial;
    use nb;

    impl<T> serial::Write<u8> for UarteTx<T>
    where
        T: Instance,
    {
        type Error = Error;

        /// Write a single byte non-blocking. Returns nb::Error::WouldBlock if not yet done.
        fn write(&mut self, b: u8) -> nb::Result<(), Self::Error> {
            let uarte = unsafe { &*T::ptr() };

            // If txstarted is set, we are in the process of transmitting.
            let in_progress = uarte.events_txstarted.read().bits() == 1;

            if in_progress {
                self.flush()
            } else {
                // Start a new transmission, copy value into transmit buffer.

                let tx_buffer = &mut self.tx_buf;
                tx_buffer[0] = b;

                // Conservative compiler fence to prevent optimizations that do not
                // take in to account actions by DMA. The fence has been placed here,
                // before any DMA action has started.
                compiler_fence(SeqCst);

                // Reset the events.
                uarte.events_endtx.reset();
                uarte.events_txstopped.reset();

                // Set up the DMA write.
                // We're giving the register a pointer to the tx buffer.
                //
                // The PTR field is a full 32 bits wide and accepts the full range
                // of values.
                uarte
                    .txd
                    .ptr
                    .write(|w| unsafe { w.ptr().bits(tx_buffer.as_ptr() as u32) });

                // We're giving it the length of the buffer, so no danger of
                // accessing invalid memory. We have verified that the length of the
                // buffer fits in an `u8`, so the cast to `u8` is also fine.
                //
                // The MAXCNT field is 8 bits wide and accepts the full range of
                // values.
                uarte
                    .txd
                    .maxcnt
                    .write(|w| unsafe { w.maxcnt().bits(tx_buffer.len() as _) });

                // Start UARTE Transmit transaction.
                // `1` is a valid value to write to task registers.
                uarte.tasks_starttx.write(|w| unsafe { w.bits(1) });
                Err(nb::Error::WouldBlock)
            }
        }

        /// Flush the TX buffer non-blocking. Returns nb::Error::WouldBlock if not yet flushed.
        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            let uarte = unsafe { &*T::ptr() };

            let in_progress = uarte.events_txstarted.read().bits() == 1;
            let endtx = uarte.events_endtx.read().bits() != 0;
            let txstopped = uarte.events_txstopped.read().bits() != 0;
            if in_progress {
                if endtx || txstopped {
                    // We are done, cleanup the state.
                    uarte.events_txstarted.reset();
                    // Conservative compiler fence to prevent optimizations that do not
                    // take in to account actions by DMA. The fence has been placed here,
                    // after all possible DMA actions have completed.
                    compiler_fence(SeqCst);

                    if txstopped {
                        return Err(nb::Error::Other(Error::Transmit));
                    }

                    // Lower power consumption by disabling the transmitter once we're
                    // finished.
                    // `1` is a valid value to write to task registers.
                    uarte.tasks_stoptx.write(|w| unsafe { w.bits(1) });
                    Ok(())
                } else {
                    // Still not done, don't block.
                    Err(nb::Error::WouldBlock)
                }
            } else {
                Ok(())
            }
        }
    }

    impl<T> core::fmt::Write for UarteTx<T>
    where
        T: Instance,
    {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            s.as_bytes()
                .iter()
                .try_for_each(|c| nb::block!(self.write(*c)))
                .map_err(|_| core::fmt::Error)
        }
    }

    impl<T> serial::Read<u8> for UarteRx<T>
    where
        T: Instance,
    {
        type Error = Error;
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            let uarte = unsafe { &*T::ptr() };

            compiler_fence(SeqCst);

            let in_progress = uarte.events_rxstarted.read().bits() == 1;
            if in_progress && uarte.events_endrx.read().bits() == 0 {
                return Err(nb::Error::WouldBlock);
            }

            if in_progress {
                let b = self.rx_buf[0];
                uarte.events_rxstarted.write(|w| w);
                uarte.events_endrx.write(|w| w);

                compiler_fence(SeqCst);
                if uarte.rxd.amount.read().bits() != 1 as u32 {
                    return Err(nb::Error::Other(Error::Receive));
                }
                Ok(b)
            } else {
                let rx_buf = &mut self.rx_buf;

                // We're giving the register a pointer to the rx buffer.
                //
                // The PTR field is a full 32 bits wide and accepts the full range
                // of values.
                uarte
                    .rxd
                    .ptr
                    .write(|w| unsafe { w.ptr().bits(rx_buf.as_ptr() as u32) });

                // We're giving it the length of the buffer, so no danger of
                // accessing invalid memory.
                //
                // The MAXCNT field is at least 8 bits wide and accepts the full
                // range of values.
                uarte
                    .rxd
                    .maxcnt
                    .write(|w| unsafe { w.maxcnt().bits(rx_buf.len() as _) });

                // Start UARTE Receive transaction.
                // `1` is a valid value to write to task registers.
                uarte.tasks_startrx.write(|w| unsafe { w.bits(1) });
                // Conservative compiler fence to prevent optimizations that do not
                // take in to account actions by DMA. The fence has been placed here,
                // after all possible DMA actions have completed.

                compiler_fence(SeqCst);

                Err(nb::Error::WouldBlock)
            }
        }
    }
}
