//! HAL interface to the SPIS peripheral.
//!
//! A module for SPI communication in peripheral mode.

use core::{
    ops::Deref,
    sync::atomic::{compiler_fence, Ordering},
};

#[cfg(any(feature = "9160", feature = "5340-app", feature = "5340-net"))]
use crate::pac::{
    spis0_ns::{
        self as spis0, EVENTS_ACQUIRED, EVENTS_END, EVENTS_ENDRX, TASKS_ACQUIRE, TASKS_RELEASE,
    },
    SPIS0_NS as SPIS0,
};

#[cfg(not(any(feature = "9160", feature = "5340-app", feature = "5340-net")))]
use crate::pac::{
    spis0::{self, EVENTS_ACQUIRED, EVENTS_END, EVENTS_ENDRX, TASKS_ACQUIRE, TASKS_RELEASE},
    SPIS0,
};

#[cfg(feature = "52811")]
use crate::pac::SPIS1;

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
use crate::pac::{SPIS1, SPIS2};

use crate::{
    gpio::{Floating, Input, Pin},
    pac::Interrupt,
    target_constants::{EASY_DMA_SIZE, SRAM_LOWER, SRAM_UPPER},
};
use embedded_dma::*;

/// Interface to a SPIS instance.
pub struct Spis<T: Instance> {
    spis: T,
}

impl<T> Spis<T>
where
    T: Instance,
{
    /// Takes ownership of the raw SPIS peripheral and relevant pins,
    /// returning a safe wrapper.
    pub fn new(spis: T, pins: Pins) -> Self {
        spis.psel.sck.write(|w| {
            unsafe { w.bits(pins.sck.psel_bits()) };
            w.connect().connected()
        });
        spis.psel.csn.write(|w| {
            unsafe { w.bits(pins.cs.psel_bits()) };
            w.connect().connected()
        });

        if let Some(p) = &pins.copi {
            spis.psel.mosi.write(|w| {
                unsafe { w.bits(p.psel_bits()) };
                w.connect().connected()
            });
        }

        if let Some(p) = &pins.cipo {
            spis.psel.miso.write(|w| {
                unsafe { w.bits(p.psel_bits()) };
                w.connect().connected()
            });
        }

        spis.config
            .modify(|_r, w| w.cpol().bit(Polarity::ActiveHigh.into()));
        spis.config
            .modify(|_r, w| w.cpha().bit(Phase::Trailing.into()));
        spis.enable.write(|w| w.enable().enabled());
        Self { spis }
    }

    /// Sets the ´default´ character (character clocked out in case of an ignored transaction).
    #[inline(always)]
    pub fn set_default_char(&self, def: u8) -> &Self {
        self.spis.def.write(|w| unsafe { w.def().bits(def) });
        self
    }

    /// Sets the over-read character (character sent on over-read of the transmit buffer).
    #[inline(always)]
    pub fn set_orc(&self, orc: u8) -> &Self {
        self.spis.orc.write(|w| unsafe { w.orc().bits(orc) });
        self
    }

    /// Sets bit order.
    #[inline(always)]
    pub fn set_order(&self, order: Order) -> &Self {
        self.spis.config.modify(|_r, w| w.order().bit(order.into()));
        self
    }

    /// Sets serial clock (SCK) polarity.
    #[inline(always)]
    pub fn set_polarity(&self, polarity: Polarity) -> &Self {
        self.spis
            .config
            .modify(|_r, w| w.cpol().bit(polarity.into()));
        self
    }

    /// Sets serial clock (SCK) phase.
    #[inline(always)]
    pub fn set_phase(&self, phase: Phase) -> &Self {
        self.spis.config.modify(|_r, w| w.cpha().bit(phase.into()));
        self
    }

    /// Sets SPI mode.
    #[inline(always)]
    pub fn set_mode(&self, mode: Mode) -> &Self {
        match mode {
            Mode::Mode0 => {
                self.set_polarity(Polarity::ActiveHigh);
                self.set_phase(Phase::Trailing);
            }
            Mode::Mode1 => {
                self.set_polarity(Polarity::ActiveHigh);
                self.set_phase(Phase::Leading);
            }
            Mode::Mode2 => {
                self.set_polarity(Polarity::ActiveLow);
                self.set_phase(Phase::Trailing);
            }
            Mode::Mode3 => {
                self.set_polarity(Polarity::ActiveLow);
                self.set_phase(Phase::Leading);
            }
        }
        self
    }

    /// Enables the SPIS instance.
    #[inline(always)]
    pub fn enable(&self) -> &Self {
        self.spis.enable.write(|w| w.enable().enabled());
        self
    }

    /// Disables the SPIS module.
    #[inline(always)]
    pub fn disable(&self) -> &Self {
        self.spis.enable.write(|w| w.enable().disabled());
        self
    }

    /// Requests acquiring the SPIS semaphore and waits until acquired.
    #[inline(always)]
    pub fn acquire(&self) -> &Self {
        compiler_fence(Ordering::SeqCst);
        self.spis.tasks_acquire.write(|w| unsafe { w.bits(1) });
        while self.spis.events_acquired.read().bits() == 0 {}
        self
    }

    /// Requests acquiring the SPIS semaphore, returning an error if not
    /// possible.
    ///
    /// Note: The semaphore will still be requested, and will be made
    /// available at a later point.
    #[inline(always)]
    pub fn try_acquire(&self) -> Result<&Self, Error> {
        compiler_fence(Ordering::SeqCst);
        self.spis.tasks_acquire.write(|w| unsafe { w.bits(1) });
        if self.spis.events_acquired.read().bits() != 0 {
            Ok(self)
        } else {
            Err(Error::SemaphoreNotAvailable)
        }
    }

    /// Releases the SPIS semaphore, enabling the SPIS to acquire it.
    #[inline(always)]
    pub fn release(&self) -> &Self {
        self.spis.tasks_release.write(|w| unsafe { w.bits(1) });
        self
    }

    /// Enables interrupt for specified event.
    #[inline(always)]
    pub fn enable_interrupt(&self, event: SpisEvent) -> &Self {
        self.spis.intenset.modify(|_r, w| match event {
            SpisEvent::Acquired => w.acquired().set_bit(),
            SpisEvent::End => w.end().set_bit(),
            SpisEvent::EndRx => w.endrx().set_bit(),
        });
        self
    }

    /// Disables interrupt for specified event.
    #[inline(always)]
    pub fn disable_interrupt(&self, event: SpisEvent) -> &Self {
        self.spis.intenclr.write(|w| match event {
            SpisEvent::Acquired => w.acquired().set_bit(),
            SpisEvent::End => w.end().set_bit(),
            SpisEvent::EndRx => w.endrx().set_bit(),
        });
        self
    }

    /// Automatically acquire the semaphore after transfer has ended.
    #[inline(always)]
    pub fn auto_acquire(&self, enabled: bool) -> &Self {
        self.spis.shorts.write(|w| w.end_acquire().bit(enabled));
        self
    }

    /// Resets all events.
    #[inline(always)]
    pub fn reset_events(&self) {
        self.spis.events_acquired.reset();
        self.spis.events_end.reset();
        self.spis.events_endrx.reset();
    }

    /// Resets specified event.
    #[inline(always)]
    pub fn reset_event(&self, event: SpisEvent) {
        match event {
            SpisEvent::Acquired => self.spis.events_acquired.reset(),
            SpisEvent::End => self.spis.events_end.reset(),
            SpisEvent::EndRx => self.spis.events_endrx.reset(),
        };
    }

    /// Checks if specified event has been triggered.
    #[inline(always)]
    pub fn is_event_triggered(&self, event: SpisEvent) -> bool {
        match event {
            SpisEvent::Acquired => self.spis.events_acquired.read().bits() != 0,
            SpisEvent::End => self.spis.events_end.read().bits() != 0,
            SpisEvent::EndRx => self.spis.events_endrx.read().bits() != 0,
        }
    }

    /// Checks if the granted transfer is done.
    #[inline(always)]
    pub fn is_done(&self) -> bool {
        self.spis.events_end.read().bits() != 0 || self.spis.events_endrx.read().bits() != 0
    }

    /// Checks if the semaphore is acquired.
    #[inline(always)]
    pub fn is_acquired(&self) -> bool {
        self.spis.events_acquired.read().bits() != 0
    }

    /// Checks if last transaction overread.
    #[inline(always)]
    pub fn is_overread(&self) -> bool {
        self.spis.status.read().overread().is_present()
    }

    /// Checks if last transaction overflowed.
    #[inline(always)]
    pub fn is_overflow(&self) -> bool {
        self.spis.status.read().overflow().is_present()
    }

    /// Returns number of bytes received in last granted transaction.
    #[inline(always)]
    pub fn amount(&self) -> u32 {
        self.spis.rxd.amount.read().bits()
    }

    /// Returns the semaphore status.
    #[inline(always)]
    pub fn semaphore_status(&self) -> SemaphoreStatus {
        match self.spis.semstat.read().bits() {
            0 => SemaphoreStatus::Free,
            1 => SemaphoreStatus::CPU,
            2 => SemaphoreStatus::SPIS,
            _ => SemaphoreStatus::CPUPending,
        }
    }

    /// Returns reference to `Acquired` event endpoint for PPI.
    #[inline(always)]
    pub fn event_acquired(&self) -> &EVENTS_ACQUIRED {
        &self.spis.events_acquired
    }

    /// Returns reference to `End` event endpoint for PPI.
    #[inline(always)]
    pub fn event_end(&self) -> &EVENTS_END {
        &self.spis.events_end
    }

    /// Returns reference to `EndRx` event endpoint for PPI.
    #[inline(always)]
    pub fn event_end_rx(&self) -> &EVENTS_ENDRX {
        &self.spis.events_endrx
    }

    /// Returns reference to `Acquire` task endpoint for PPI.
    #[inline(always)]
    pub fn task_acquire(&self) -> &TASKS_ACQUIRE {
        &self.spis.tasks_acquire
    }

    /// Returns reference to `Release` task endpoint for PPI.
    #[inline(always)]
    pub fn task_release(&self) -> &TASKS_RELEASE {
        &self.spis.tasks_release
    }

    /// Full duplex DMA transfer.
    /// Transmits the given buffer while simultaneously receiving data into the same buffer until it is filled.
    /// Buffer must be located in RAM.
    /// Returns a value that represents the in-progress DMA transfer.
    #[allow(unused_mut)]
    pub fn transfer<W, B>(mut self, mut buffer: B) -> Result<Transfer<T, B>, (Error, Spis<T>, B)>
    where
        B: WriteBuffer<Word = W> + 'static,
    {
        let (ptr, len) = unsafe { buffer.write_buffer() };
        let maxcnt = len * core::mem::size_of::<W>();
        if maxcnt > EASY_DMA_SIZE {
            return Err((Error::BufferTooLong, self, buffer));
        }
        compiler_fence(Ordering::SeqCst);
        self.spis
            .txd
            .ptr
            .write(|w| unsafe { w.ptr().bits(ptr as u32) });
        self.spis
            .rxd
            .ptr
            .write(|w| unsafe { w.ptr().bits(ptr as u32) });
        self.spis
            .txd
            .maxcnt
            .write(|w| unsafe { w.bits(maxcnt as u32) });
        self.spis
            .rxd
            .maxcnt
            .write(|w| unsafe { w.bits(maxcnt as u32) });

        self.release();
        Ok(Transfer {
            inner: Some(Inner { buffer, spis: self }),
        })
    }

    /// Full duplex DMA transfer.
    /// Transmits the given `tx_buffer` while simultaneously receiving data
    /// into the given `rx_buffer` until it is filled.
    /// The buffers must be located in RAM.
    /// Returns a value that represents the in-progress DMA transfer.
    #[allow(unused_mut)]
    pub fn transfer_split<TxW, RxW, TxB, RxB>(
        mut self,
        tx_buffer: TxB,
        mut rx_buffer: RxB,
    ) -> Result<TransferSplit<T, TxB, RxB>, (Error, Spis<T>, TxB, RxB)>
    where
        TxB: ReadBuffer<Word = TxW> + 'static,
        RxB: WriteBuffer<Word = RxW> + 'static,
    {
        let (rx_ptr, rx_len) = unsafe { rx_buffer.write_buffer() };
        let (tx_ptr, tx_len) = unsafe { tx_buffer.read_buffer() };
        let rx_maxcnt = rx_len * core::mem::size_of::<RxW>();
        let tx_maxcnt = tx_len * core::mem::size_of::<TxW>();
        if rx_maxcnt.max(tx_maxcnt) > EASY_DMA_SIZE {
            return Err((Error::BufferTooLong, self, tx_buffer, rx_buffer));
        }
        if (tx_ptr as usize) < SRAM_LOWER || (tx_ptr as usize) > SRAM_UPPER {
            return Err((Error::DMABufferNotInDataMemory, self, tx_buffer, rx_buffer));
        }
        compiler_fence(Ordering::SeqCst);
        self.spis
            .txd
            .ptr
            .write(|w| unsafe { w.ptr().bits(tx_ptr as u32) });
        self.spis
            .rxd
            .ptr
            .write(|w| unsafe { w.ptr().bits(rx_ptr as u32) });
        self.spis
            .rxd
            .maxcnt
            .write(|w| unsafe { w.bits(rx_maxcnt as u32) });
        self.spis
            .txd
            .maxcnt
            .write(|w| unsafe { w.bits(tx_maxcnt as u32) });

        self.release();
        Ok(TransferSplit {
            inner: Some(InnerSplit {
                tx_buffer,
                rx_buffer,
                spis: self,
            }),
        })
    }

    /// Returns the raw interface to the underlying SPIS peripheral.
    pub fn free(self) -> (T, Pins) {
        let sck = self.spis.psel.sck.read();
        let cs = self.spis.psel.csn.read();
        let copi = self.spis.psel.mosi.read();
        let cipo = self.spis.psel.miso.read();
        self.spis.psel.sck.reset();
        self.spis.psel.csn.reset();
        self.spis.psel.mosi.reset();
        self.spis.psel.miso.reset();
        (
            self.spis,
            Pins {
                sck: unsafe { Pin::from_psel_bits(sck.bits()) },
                cs: unsafe { Pin::from_psel_bits(cs.bits()) },
                copi: if copi.connect().is_connected() {
                    Some(unsafe { Pin::from_psel_bits(copi.bits()) })
                } else {
                    None
                },
                cipo: if cipo.connect().is_connected() {
                    Some(unsafe { Pin::from_psel_bits(cipo.bits()) })
                } else {
                    None
                },
            },
        )
    }
}

/// A DMA transfer
pub struct Transfer<T: Instance, B> {
    // FIXME: Always `Some`, only using `Option` here to allow moving fields out of `inner`.
    inner: Option<Inner<T, B>>,
}

struct Inner<T: Instance, B> {
    buffer: B,
    spis: Spis<T>,
}

impl<T: Instance, B> Transfer<T, B> {
    /// Blocks until the transfer is done and returns the buffer.
    pub fn wait(mut self) -> (B, Spis<T>) {
        compiler_fence(Ordering::SeqCst);
        let inner = self
            .inner
            .take()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        while !inner.spis.is_done() {}
        inner.spis.acquire();
        (inner.buffer, inner.spis)
    }

    /// Checks if the granted transfer is done.
    #[inline(always)]
    pub fn is_done(&mut self) -> bool {
        let inner = self
            .inner
            .as_mut()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        inner.spis.is_done()
    }
}

impl<T: Instance, B> Drop for Transfer<T, B> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            compiler_fence(Ordering::SeqCst);
            while !inner.spis.is_done() {}
            inner.spis.disable();
        }
    }
}
/// A full duplex DMA transfer
pub struct TransferSplit<T: Instance, TxB, RxB> {
    // FIXME: Always `Some`, only using `Option` here to allow moving fields out of `inner`.
    inner: Option<InnerSplit<T, TxB, RxB>>,
}

struct InnerSplit<T: Instance, TxB, RxB> {
    tx_buffer: TxB,
    rx_buffer: RxB,
    spis: Spis<T>,
}

impl<T: Instance, TxB, RxB> TransferSplit<T, TxB, RxB> {
    /// Blocks until the transfer is done and returns the buffer.
    pub fn wait(mut self) -> (TxB, RxB, Spis<T>) {
        compiler_fence(Ordering::SeqCst);
        let inner = self
            .inner
            .take()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        while !inner.spis.is_done() {}
        inner.spis.acquire();
        (inner.tx_buffer, inner.rx_buffer, inner.spis)
    }

    /// Checks if the granted transfer is done.
    #[inline(always)]
    pub fn is_done(&mut self) -> bool {
        let inner = self
            .inner
            .as_mut()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        inner.spis.is_done()
    }
}

impl<T: Instance, TxB, RxB> Drop for TransferSplit<T, TxB, RxB> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            compiler_fence(Ordering::SeqCst);
            while !inner.spis.is_done() {}
            inner.spis.disable();
        }
    }
}

/// SPIS events
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SpisEvent {
    End,
    EndRx,
    Acquired,
}
/// Semaphore status
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SemaphoreStatus {
    Free,
    CPU,
    SPIS,
    CPUPending,
}
/// Bit order
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Order {
    MsbFirst,
    LsbFirst,
}
impl From<Order> for bool {
    fn from(variant: Order) -> Self {
        match variant {
            Order::MsbFirst => false,
            Order::LsbFirst => true,
        }
    }
}

/// Serial clock (SCK) phase
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Phase {
    Trailing,
    Leading,
}
impl From<Phase> for bool {
    fn from(variant: Phase) -> Self {
        match variant {
            Phase::Trailing => false,
            Phase::Leading => true,
        }
    }
}

/// Serial clock (SCK) polarity
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Polarity {
    ActiveHigh,
    ActiveLow,
}
impl From<Polarity> for bool {
    fn from(variant: Polarity) -> Self {
        match variant {
            Polarity::ActiveHigh => false,
            Polarity::ActiveLow => true,
        }
    }
}

/// SPI mode
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Mode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    DMABufferNotInDataMemory,
    BufferTooLong,
    SemaphoreNotAvailable,
}

/// GPIO pins for SPIS interface.
pub struct Pins {
    /// SPI clock
    pub sck: Pin<Input<Floating>>,
    /// Chip select
    pub cs: Pin<Input<Floating>>,
    /// COPI Controller out, peripheral in
    /// None if unused
    pub copi: Option<Pin<Input<Floating>>>,
    /// CIPO Controller in, peripheral out
    /// None if unused
    pub cipo: Option<Pin<Input<Floating>>>,
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::SPIS0 {}
    #[cfg(not(any(
        feature = "9160",
        feature = "5340-app",
        feature = "5340-net",
        feature = "52810"
    )))]
    impl Sealed for super::SPIS1 {}
    #[cfg(not(any(
        feature = "9160",
        feature = "5340-app",
        feature = "5340-net",
        feature = "52811",
        feature = "52810"
    )))]
    impl Sealed for super::SPIS2 {}
}

pub trait Instance: sealed::Sealed + Deref<Target = spis0::RegisterBlock> {
    const INTERRUPT: Interrupt;
}

impl Instance for SPIS0 {
    #[cfg(not(any(
        feature = "9160",
        feature = "5340-app",
        feature = "5340-net",
        feature = "52811",
        feature = "52810"
    )))]
    const INTERRUPT: Interrupt = Interrupt::SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0;
    #[cfg(feature = "9160")]
    const INTERRUPT: Interrupt = Interrupt::UARTE0_SPIM0_SPIS0_TWIM0_TWIS0;
    #[cfg(any(feature = "5340-app", feature = "5340-net"))]
    const INTERRUPT: Interrupt = Interrupt::SERIAL0;
    #[cfg(feature = "52810")]
    const INTERRUPT: Interrupt = Interrupt::SPIM0_SPIS0_SPI0;
    #[cfg(feature = "52811")]
    const INTERRUPT: Interrupt = Interrupt::TWIM0_TWIS0_TWI0_SPIM0_SPIS0_SPI0;
}

#[cfg(not(any(
    feature = "9160",
    feature = "5340-app",
    feature = "5340-net",
    feature = "52810"
)))]
impl Instance for SPIS1 {
    #[cfg(not(feature = "52811"))]
    const INTERRUPT: Interrupt = Interrupt::SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1;
    #[cfg(feature = "52811")]
    const INTERRUPT: Interrupt = Interrupt::SPIM1_SPIS1_SPI1;
}

#[cfg(not(any(
    feature = "9160",
    feature = "5340-app",
    feature = "5340-net",
    feature = "52811",
    feature = "52810"
)))]
impl Instance for SPIS2 {
    const INTERRUPT: Interrupt = Interrupt::SPIM2_SPIS2_SPI2;
}
