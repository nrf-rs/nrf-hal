//! HAL interface to the TWIS peripheral.
//!

use core::{
    ops::Deref,
    sync::atomic::{compiler_fence, Ordering::SeqCst},
};

#[cfg(any(feature = "9160", feature = "5340-app", feature = "5340-net"))]
use crate::pac::{twis0_ns as twis0, P0_NS as P0, TWIS0_NS as TWIS0};

#[cfg(not(any(feature = "9160", feature = "5340-app", feature = "5340-net")))]
use crate::pac::{twis0, P0, TWIS0};

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
use crate::pac::TWIS1;

use twis0::{
    EVENTS_ERROR, EVENTS_READ, EVENTS_RXSTARTED, EVENTS_STOPPED, EVENTS_TXSTARTED, EVENTS_WRITE,
    TASKS_PREPARERX, TASKS_PREPARETX, TASKS_RESUME, TASKS_STOP, TASKS_SUSPEND,
};

use crate::{
    gpio::{Floating, Input, Pin},
    pac::Interrupt,
    slice_in_ram_or,
    target_constants::{EASY_DMA_SIZE, SRAM_LOWER, SRAM_UPPER},
};
use embedded_dma::*;

/// Interface to a TWIS instance.
pub struct Twis<T: Instance>(T);

impl<T> Twis<T>
where
    T: Instance,
{
    pub fn new(twis: T, pins: Pins, address0: u8) -> Self {
        // The TWIS peripheral requires the pins to be in a mode that is not
        // exposed through the GPIO API, and might it might not make sense to
        // expose it there.
        //
        // Until we've figured out what to do about this, let's just configure
        // the pins through the raw peripheral API. All of the following is
        // safe, as we own the pins now and have exclusive access to their
        // registers.
        for &pin in &[pins.scl.pin(), pins.sda.pin()] {
            unsafe { &*P0::ptr() }.pin_cnf[pin as usize].write(|w| {
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

        twis.psel.scl.write(|w| {
            unsafe { w.bits(pins.scl.psel_bits()) };
            w.connect().connected()
        });
        twis.psel.sda.write(|w| {
            unsafe { w.bits(pins.sda.psel_bits()) };
            w.connect().connected()
        });

        twis.address[0].write(|w| unsafe { w.address().bits(address0) });
        twis.config.modify(|_r, w| w.address0().enabled());

        Twis(twis)
    }

    /// Configures secondary I2C address.
    #[inline(always)]
    pub fn set_address1(&self, address1: u8) -> &Self {
        self.0.address[1].write(|w| unsafe { w.address().bits(address1) });
        self.0.config.modify(|_r, w| w.address1().enabled());
        self
    }

    /// Sets the over-read character (character sent on over-read of the transmit buffer).
    #[inline(always)]
    pub fn set_orc(&self, orc: u8) -> &Self {
        self.0.orc.write(|w| unsafe { w.orc().bits(orc) });
        self
    }

    /// Enables the TWIS instance.
    #[inline(always)]
    pub fn enable(&self) -> &Self {
        self.0.enable.write(|w| w.enable().enabled());
        self
    }

    /// Disables the TWIS instance.
    #[inline(always)]
    pub fn disable(&self) -> &Self {
        self.0.enable.write(|w| w.enable().disabled());
        self
    }

    /// Stops the TWI transaction and waits until it has stopped.
    #[inline(always)]
    pub fn stop(&self) -> &Self {
        compiler_fence(SeqCst);
        self.0.tasks_stop.write(|w| unsafe { w.bits(1) });
        while self.0.events_stopped.read().bits() == 0 {}
        self
    }

    /// Enables interrupt for specified command.
    #[inline(always)]
    pub fn enable_interrupt(&self, event: TwiEvent) -> &Self {
        self.0.intenset.modify(|_r, w| match event {
            TwiEvent::Error => w.error().set_bit(),
            TwiEvent::Stopped => w.stopped().set_bit(),
            TwiEvent::RxStarted => w.rxstarted().set_bit(),
            TwiEvent::TxStarted => w.txstarted().set_bit(),
            TwiEvent::Write => w.write().set_bit(),
            TwiEvent::Read => w.read().set_bit(),
        });
        self
    }

    /// Disables interrupt for specified command.
    #[inline(always)]
    pub fn disable_interrupt(&self, event: TwiEvent) -> &Self {
        self.0.intenclr.write(|w| match event {
            TwiEvent::Error => w.error().set_bit(),
            TwiEvent::Stopped => w.stopped().set_bit(),
            TwiEvent::RxStarted => w.rxstarted().set_bit(),
            TwiEvent::TxStarted => w.txstarted().set_bit(),
            TwiEvent::Write => w.write().set_bit(),
            TwiEvent::Read => w.read().set_bit(),
        });
        self
    }

    /// Resets read and write events.
    #[inline(always)]
    pub fn reset_events(&self) {
        self.0.events_error.reset();
        self.0.events_stopped.reset();
        self.0.events_rxstarted.reset();
        self.0.events_txstarted.reset();
        self.0.events_write.reset();
        self.0.events_read.reset();
    }

    /// Resets specified event.
    #[inline(always)]
    pub fn reset_event(&self, event: TwiEvent) {
        match event {
            TwiEvent::Error => self.0.events_error.reset(),
            TwiEvent::Stopped => self.0.events_stopped.reset(),
            TwiEvent::RxStarted => self.0.events_rxstarted.reset(),
            TwiEvent::TxStarted => self.0.events_txstarted.reset(),
            TwiEvent::Write => self.0.events_write.reset(),
            TwiEvent::Read => self.0.events_read.reset(),
        };
    }

    /// Returns matched address for latest command.
    #[inline(always)]
    pub fn address_match(&self) -> u8 {
        self.0.address[self.0.match_.read().match_().bits() as usize]
            .read()
            .address()
            .bits()
    }

    /// Checks if specified event has been triggered.
    #[inline(always)]
    pub fn is_event_triggered(&self, event: TwiEvent) -> bool {
        match event {
            TwiEvent::Error => self.0.events_error.read().bits() != 0,
            TwiEvent::Stopped => self.0.events_stopped.read().bits() != 0,
            TwiEvent::RxStarted => self.0.events_rxstarted.read().bits() != 0,
            TwiEvent::TxStarted => self.0.events_txstarted.read().bits() != 0,
            TwiEvent::Write => self.0.events_write.read().bits() != 0,
            TwiEvent::Read => self.0.events_read.read().bits() != 0,
        }
    }

    /// Checks if the TWI transaction is done.
    #[inline(always)]
    pub fn is_done(&self) -> bool {
        self.0.events_stopped.read().bits() != 0
    }

    /// Returns number of bytes received in last granted transaction.
    #[inline(always)]
    pub fn amount(&self) -> u32 {
        self.0.rxd.amount.read().bits()
    }

    /// Checks if RX buffer overflow was detected.
    #[inline(always)]
    pub fn is_overflow(&self) -> bool {
        self.0.errorsrc.read().overflow().bit()
    }

    /// Checks if NACK was sent after receiving a data byte.
    #[inline(always)]
    pub fn is_data_nack(&self) -> bool {
        self.0.errorsrc.read().dnack().bit()
    }

    /// Checks if TX buffer over-read was detected and ORC was clocked out.
    #[inline(always)]
    pub fn is_overread(&self) -> bool {
        self.0.errorsrc.read().overread().bit()
    }

    /// Returns reference to `READ` event endpoint for PPI.
    #[inline(always)]
    pub fn event_read(&self) -> &EVENTS_READ {
        &self.0.events_read
    }

    /// Returns reference to `WRITE` event endpoint for PPI.
    #[inline(always)]
    pub fn event_write(&self) -> &EVENTS_WRITE {
        &self.0.events_write
    }

    /// Returns reference to `STOPPED` event endpoint for PPI.
    #[inline(always)]
    pub fn event_stopped(&self) -> &EVENTS_STOPPED {
        &self.0.events_stopped
    }

    /// Returns reference to `ERROR` event endpoint for PPI.
    #[inline(always)]
    pub fn event_error(&self) -> &EVENTS_ERROR {
        &self.0.events_error
    }

    /// Returns reference to `RXSTARTED` event endpoint for PPI.
    #[inline(always)]
    pub fn event_rx_started(&self) -> &EVENTS_RXSTARTED {
        &self.0.events_rxstarted
    }

    /// Returns reference to `TXSTARTED` event endpoint for PPI.
    #[inline(always)]
    pub fn event_tx_started(&self) -> &EVENTS_TXSTARTED {
        &self.0.events_txstarted
    }

    /// Returns reference to `STOP` task endpoint for PPI.
    #[inline(always)]
    pub fn task_stop(&self) -> &TASKS_STOP {
        &self.0.tasks_stop
    }

    /// Returns reference to `SUSPEND` task endpoint for PPI.
    #[inline(always)]
    pub fn task_suspend(&self) -> &TASKS_SUSPEND {
        &self.0.tasks_suspend
    }

    /// Returns reference to `RESUME` task endpoint for PPI.
    #[inline(always)]
    pub fn task_resume(&self) -> &TASKS_RESUME {
        &self.0.tasks_resume
    }

    /// Returns reference to `PREPARERX` task endpoint for PPI.
    #[inline(always)]
    pub fn task_prepare_rx(&self) -> &TASKS_PREPARERX {
        &self.0.tasks_preparerx
    }

    /// Returns reference to `PREPARETX` task endpoint for PPI.
    #[inline(always)]
    pub fn task_prepare_tx(&self) -> &TASKS_PREPARETX {
        &self.0.tasks_preparetx
    }

    /// Write to an I2C controller.
    ///
    /// The buffer must reside in RAM and have a length of at most
    /// 255 bytes on the nRF52832 and at most 65535 bytes on the nRF52840.
    pub fn tx_blocking(&mut self, buffer: &[u8]) -> Result<(), Error> {
        slice_in_ram_or(buffer, Error::DMABufferNotInDataMemory)?;

        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::TxBufferTooLong);
        }

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started.
        compiler_fence(SeqCst);

        // Set up the DMA write.
        self.0
            .txd
            .ptr
            .write(|w| unsafe { w.ptr().bits(buffer.as_ptr() as u32) });
        self.0
            .txd
            .maxcnt
            .write(|w| unsafe { w.maxcnt().bits(buffer.len() as _) });

        // Clear errors.
        self.0.errorsrc.write(|w| w);

        // Start write operation.
        self.0.tasks_preparetx.write(|w| unsafe { w.bits(1) });

        // Wait until write operation has ended.
        while self.0.events_stopped.read().bits() == 0 {}
        self.0.events_stopped.write(|w| w); // reset event

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed.
        compiler_fence(SeqCst);

        if self.0.errorsrc.read().dnack().bits() {
            return Err(Error::DataNack);
        }

        if self.0.errorsrc.read().overflow().bits() {
            return Err(Error::OverFlow);
        }

        if self.0.txd.amount.read().bits() != buffer.len() as u32 {
            return Err(Error::Transmit);
        }

        Ok(())
    }

    /// Read from an I2C controller.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn rx_blocking(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        // NOTE: RAM slice check is not necessary, as a mutable slice can only be
        // built from data located in RAM.

        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::RxBufferTooLong);
        }

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started.
        compiler_fence(SeqCst);

        // Set up the DMA read.
        self.0
            .rxd
            .ptr
            .write(|w| unsafe { w.ptr().bits(buffer.as_mut_ptr() as u32) });
        self.0
            .rxd
            .maxcnt
            .write(|w| unsafe { w.maxcnt().bits(buffer.len() as _) });

        // Clear errors.
        self.0.errorsrc.write(|w| w);

        // Start read operation.
        self.0.tasks_preparerx.write(|w| unsafe { w.bits(1) });

        // Wait until read operation has ended.
        while self.0.events_stopped.read().bits() == 0 {}
        self.0.events_stopped.write(|w| w); // reset event

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed.
        compiler_fence(SeqCst);

        if self.0.errorsrc.read().overread().bits() {
            return Err(Error::OverRead);
        }

        if self.0.rxd.amount.read().bits() != buffer.len() as u32 {
            return Err(Error::Receive);
        }

        Ok(())
    }

    /// Receives data into the given `buffer`. Buffer must be located in RAM.
    /// Returns a value that represents the in-progress DMA transfer.
    pub fn rx<W, B>(self, mut buffer: B) -> Result<Transfer<T, B>, Error>
    where
        B: WriteBuffer<Word = W> + 'static,
    {
        let (ptr, len) = unsafe { buffer.write_buffer() };
        let maxcnt = len * core::mem::size_of::<W>();
        if maxcnt > EASY_DMA_SIZE {
            return Err(Error::RxBufferTooLong);
        }
        compiler_fence(SeqCst);
        self.0
            .rxd
            .ptr
            .write(|w| unsafe { w.ptr().bits(ptr as u32) });
        self.0
            .rxd
            .maxcnt
            .write(|w| unsafe { w.bits(maxcnt as u32) });
        self.0.errorsrc.reset();
        self.0.tasks_preparerx.write(|w| unsafe { w.bits(1) });
        Ok(Transfer {
            inner: Some(Inner { buffer, twis: self }),
        })
    }

    /// Transmits data from the given `buffer`. Buffer must be located in RAM.
    /// Returns a value that represents the in-progress DMA transfer.
    pub fn tx<W, B>(self, buffer: B) -> Result<Transfer<T, B>, Error>
    where
        B: ReadBuffer<Word = W> + 'static,
    {
        let (ptr, len) = unsafe { buffer.read_buffer() };
        let maxcnt = len * core::mem::size_of::<W>();
        if maxcnt > EASY_DMA_SIZE {
            return Err(Error::TxBufferTooLong);
        }
        if (ptr as usize) < SRAM_LOWER || (ptr as usize) > SRAM_UPPER {
            return Err(Error::DMABufferNotInDataMemory);
        }
        compiler_fence(SeqCst);
        self.0
            .txd
            .ptr
            .write(|w| unsafe { w.ptr().bits(ptr as u32) });
        self.0
            .txd
            .maxcnt
            .write(|w| unsafe { w.bits(maxcnt as u32) });

        self.0.errorsrc.reset();
        self.0.tasks_preparetx.write(|w| unsafe { w.bits(1) });
        Ok(Transfer {
            inner: Some(Inner { buffer, twis: self }),
        })
    }

    /// Return the raw interface to the underlying TWIS peripheral.
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

/// The pins used by the TWIS peripheral.
///
/// Currently, only P0 pins are supported.
pub struct Pins {
    // Serial Clock Line.
    pub scl: Pin<Input<Floating>>,

    // Serial Data Line.
    pub sda: Pin<Input<Floating>>,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    Transmit,
    Receive,
    DMABufferNotInDataMemory,
    DataNack,
    OverFlow,
    OverRead,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TwiEvent {
    Stopped,
    Error,
    RxStarted,
    TxStarted,
    Write,
    Read,
}

/// A DMA transfer
pub struct Transfer<T: Instance, B> {
    // FIXME: Always `Some`, only using `Option` here to allow moving fields out of `inner`.
    inner: Option<Inner<T, B>>,
}

struct Inner<T: Instance, B> {
    buffer: B,
    twis: Twis<T>,
}

impl<T: Instance, B> Transfer<T, B> {
    /// Blocks until the transaction is done and returns the buffer.
    pub fn wait(mut self) -> (B, Twis<T>) {
        compiler_fence(SeqCst);
        let inner = self
            .inner
            .take()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        while !inner.twis.is_done() {}
        (inner.buffer, inner.twis)
    }

    /// Checks if the granted transaction is done.
    #[inline(always)]
    pub fn is_done(&mut self) -> bool {
        let inner = self
            .inner
            .as_mut()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        inner.twis.is_done()
    }
}

impl<T: Instance, B> Drop for Transfer<T, B> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            compiler_fence(SeqCst);
            inner.twis.stop();
            inner.twis.disable();
        }
    }
}

/// Implemented by all TWIS instances
pub trait Instance: sealed::Sealed + Deref<Target = twis0::RegisterBlock> {
    const INTERRUPT: Interrupt;
}

impl Instance for TWIS0 {
    #[cfg(not(any(
        feature = "9160",
        feature = "5340-app",
        feature = "5340-net",
        feature = "52810",
        feature = "52811"
    )))]
    const INTERRUPT: Interrupt = Interrupt::SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0;
    #[cfg(any(feature = "5340-app", feature = "5340-net"))]
    const INTERRUPT: Interrupt = Interrupt::SERIAL0;
    #[cfg(feature = "9160")]
    const INTERRUPT: Interrupt = Interrupt::UARTE0_SPIM0_SPIS0_TWIM0_TWIS0;
    #[cfg(feature = "52810")]
    const INTERRUPT: Interrupt = Interrupt::TWIM0_TWIS0_TWI0;
    #[cfg(feature = "52811")]
    const INTERRUPT: Interrupt = Interrupt::TWIM0_TWIS0_TWI0_SPIM0_SPIS0_SPI0;
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::TWIS0 {}
}

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
mod _twis1 {
    use super::*;
    impl sealed::Sealed for TWIS1 {}
    impl Instance for TWIS1 {
        const INTERRUPT: Interrupt = Interrupt::SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1;
    }
}
