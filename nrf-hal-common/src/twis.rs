//! HAL interface to the TWIS peripheral.
//!

use core::ops::Deref;
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};

use crate::pac::{twis0, P0, TWIS0};

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
use crate::pac::TWIS1;

use crate::pac::{
    generic::Reg,
    twis0::{_EVENTS_READ, _EVENTS_STOPPED, _EVENTS_WRITE, _TASKS_STOP},
};

use crate::{
    gpio::{Floating, Input, Pin},
    slice_in_ram_or,
    target_constants::EASY_DMA_SIZE,
};

/// Interface to a TWIS instance.
pub struct Twis<T>(T);

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
            let w = unsafe { w.pin().bits(pins.scl.pin()) };
            #[cfg(feature = "52840")]
            let w = w.port().bit(pins.scl.port().bit());
            w.connect().connected()
        });
        twis.psel.sda.write(|w| {
            let w = unsafe { w.pin().bits(pins.sda.pin()) };
            #[cfg(feature = "52840")]
            let w = w.port().bit(pins.sda.port().bit());
            w.connect().connected()
        });

        twis.address[0].write(|w| unsafe { w.address().bits(address0) });
        twis.config.modify(|_r, w| w.address0().enabled());

        Twis(twis)
    }

    /// Configures secondary I2C address.
    #[inline(always)]
    pub fn address1(&self, address1: u8) -> &Self {
        self.0.address[1].write(|w| unsafe { w.address().bits(address1) });
        self.0.config.modify(|_r, w| w.address1().enabled());
        self
    }

    /// Sets the over-read character (character sent on over-read of the transmit buffer).
    #[inline(always)]
    pub fn orc(&self, orc: u8) -> &Self {
        self.0.orc.write(|w| unsafe { w.orc().bits(orc) });
        self
    }

    /// Enables the TWIS instance.
    #[inline(always)]
    pub fn enable(&self) {
        self.0.enable.write(|w| w.enable().enabled());
    }

    /// Enables interrupt for specified command.
    #[inline(always)]
    pub fn enable_interrupt(&self, command: TwiEvent) -> &Self {
        self.0.intenset.modify(|_r, w| match command {
            TwiEvent::Read => w.read().set_bit(),
            TwiEvent::Write => w.write().set_bit(),
        });
        self
    }

    /// Disables interrupt for specified command.
    #[inline(always)]
    pub fn disable_interrupt(&self, command: TwiEvent) -> &Self {
        self.0.intenclr.write(|w| match command {
            TwiEvent::Read => w.read().set_bit(),
            TwiEvent::Write => w.write().set_bit(),
        });
        self
    }

    /// Resets read and write events.
    #[inline(always)]
    pub fn reset_events(&self) {
        self.0.events_read.write(|w| w);
        self.0.events_write.write(|w| w);
    }

    /// Resets specified event.
    #[inline(always)]
    pub fn reset_event(&self, event: TwiEvent) {
        match event {
            TwiEvent::Read => self.0.events_read.write(|w| w),
            TwiEvent::Write => self.0.events_write.write(|w| w),
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
            TwiEvent::Read => self.0.events_read.read().bits() != 0,
            TwiEvent::Write => self.0.events_write.read().bits() != 0,
        }
    }

    /// Returns reference to `READ` event endpoint for PPI.
    #[inline(always)]
    pub fn event_read(&self) -> &Reg<u32, _EVENTS_READ> {
        &self.0.events_read
    }

    /// Returns reference to `WRITE` event endpoint for PPI.
    #[inline(always)]
    pub fn event_write(&self) -> &Reg<u32, _EVENTS_WRITE> {
        &self.0.events_write
    }

    /// Returns reference to `STOPPED` event endpoint for PPI.
    #[inline(always)]
    pub fn event_stopped(&self) -> &Reg<u32, _EVENTS_STOPPED> {
        &self.0.events_stopped
    }

    /// Returns reference to `STOP` task endpoint for PPI.
    #[inline(always)]
    pub fn task_stop(&self) -> &Reg<u32, _TASKS_STOP> {
        &self.0.tasks_stop
    }

    /// Write to an I2C controller.
    ///
    /// The buffer must reside in RAM and have a length of at most
    /// 255 bytes on the nRF52832 and at most 65535 bytes on the nRF52840.
    pub fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
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
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
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

    /// Return the raw interface to the underlying TWIS peripheral.
    pub fn free(self) -> T {
        self.0
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

#[derive(Debug)]
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
    Read,
    Write,
}

/// Implemented by all TWIS instances
pub trait Instance: Deref<Target = twis0::RegisterBlock> {}

impl Instance for TWIS0 {}

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
impl Instance for TWIS1 {}
