use core::{
    ops::Deref,
    sync::atomic::{compiler_fence, Ordering},
};

#[cfg(feature = "9160")]
use crate::pac::{
    spis0_ns::{
        self as spis0, _EVENTS_ACQUIRED, _EVENTS_END, _EVENTS_ENDRX, _TASKS_ACQUIRE, _TASKS_RELEASE,
    },
    SPIS0_NS as SPIS0,
};

#[cfg(not(feature = "9160"))]
use crate::pac::{
    spis0::{self, _EVENTS_ACQUIRED, _EVENTS_END, _EVENTS_ENDRX, _TASKS_ACQUIRE, _TASKS_RELEASE},
    SPIS0,
};

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
use crate::pac::{SPIS1, SPIS2};

use crate::{
    gpio::{Floating, Input, Pin},
    pac::{generic::Reg, Interrupt},
    target_constants::{EASY_DMA_SIZE, SRAM_LOWER, SRAM_UPPER},
};
use embedded_dma::*;

pub struct Spis<T: Instance>(T);

impl<T> Spis<T>
where
    T: Instance,
{
    /// Takes ownership of the raw SPIS peripheral, returning a safe wrapper.
    pub fn new(
        spis: T,
        sck_pin: &Pin<Input<Floating>>,
        cs_pin: &Pin<Input<Floating>>,
        copi_pin: Option<&Pin<Input<Floating>>>,
        cipo_pin: Option<&Pin<Input<Floating>>>,
    ) -> Self {
        spis.psel.sck.write(|w| {
            unsafe { w.pin().bits(sck_pin.pin()) };
            #[cfg(any(feature = "52833", feature = "52840"))]
            w.port().bit(sck_pin.port().bit());
            w.connect().connected()
        });
        spis.psel.csn.write(|w| {
            unsafe { w.pin().bits(cs_pin.pin()) };
            #[cfg(any(feature = "52833", feature = "52840"))]
            w.port().bit(cs_pin.port().bit());
            w.connect().connected()
        });

        if let Some(p) = copi_pin {
            spis.psel.mosi.write(|w| {
                unsafe { w.pin().bits(p.pin()) };
                #[cfg(any(feature = "52833", feature = "52840"))]
                w.port().bit(p.port().bit());
                w.connect().connected()
            });
        }

        if let Some(p) = cipo_pin {
            spis.psel.miso.write(|w| {
                unsafe { w.pin().bits(p.pin()) };
                #[cfg(any(feature = "52833", feature = "52840"))]
                w.port().bit(p.port().bit());
                w.connect().connected()
            });
        }

        spis.config
            .modify(|_r, w| w.cpol().bit(Polarity::ActiveHigh.into()));
        spis.config
            .modify(|_r, w| w.cpha().bit(Phase::Trailing.into()));
        spis.enable.write(|w| w.enable().enabled());
        Self(spis)
    }

    /// Sets the ´default´ character (character clocked out in case of an ignored transaction).
    #[inline(always)]
    pub fn set_default_char(&self, def: u8) -> &Self {
        self.0.def.write(|w| unsafe { w.def().bits(def) });
        self
    }

    /// Sets the over-read character (character sent on over-read of the transmit buffer).
    #[inline(always)]
    pub fn set_orc(&self, orc: u8) -> &Self {
        self.0.orc.write(|w| unsafe { w.orc().bits(orc) });
        self
    }

    /// Sets bit order.
    #[inline(always)]
    pub fn set_order(&self, order: Order) -> &Self {
        self.0.config.modify(|_r, w| w.order().bit(order.into()));
        self
    }

    /// Sets serial clock (SCK) polarity.
    #[inline(always)]
    pub fn set_polarity(&self, polarity: Polarity) -> &Self {
        self.0.config.modify(|_r, w| w.cpol().bit(polarity.into()));
        self
    }

    /// Sets serial clock (SCK) phase.
    #[inline(always)]
    pub fn set_phase(&self, phase: Phase) -> &Self {
        self.0.config.modify(|_r, w| w.cpha().bit(phase.into()));
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
        self.0.enable.write(|w| w.enable().enabled());
        self
    }

    /// Disables the SPIS module.
    #[inline(always)]
    pub fn disable(&self) -> &Self {
        self.0.enable.write(|w| w.enable().disabled());
        self
    }

    /// Requests acquiring the SPIS semaphore and waits until acquired.
    #[inline(always)]
    pub fn acquire(&self) -> &Self {
        self.enable();
        self.0.tasks_acquire.write(|w| unsafe { w.bits(1) });
        while self.0.events_acquired.read().bits() == 0 {}
        self
    }

    /// Releases the SPIS semaphore, enabling the SPIS to acquire it.
    #[inline(always)]
    pub fn release(&self) -> &Self {
        self.0.tasks_release.write(|w| unsafe { w.bits(1) });
        self
    }

    /// Enables interrupt for specified event.
    #[inline(always)]
    pub fn enable_interrupt(&self, command: SpisEvent) -> &Self {
        self.0.intenset.modify(|_r, w| match command {
            SpisEvent::Acquired => w.acquired().set_bit(),
            SpisEvent::End => w.end().set_bit(),
            SpisEvent::EndRx => w.endrx().set_bit(),
        });
        self
    }

    /// Disables interrupt for specified event.
    #[inline(always)]
    pub fn disable_interrupt(&self, command: SpisEvent) -> &Self {
        self.0.intenclr.write(|w| match command {
            SpisEvent::Acquired => w.acquired().set_bit(),
            SpisEvent::End => w.end().set_bit(),
            SpisEvent::EndRx => w.endrx().set_bit(),
        });
        self
    }

    /// Automatically acquire the semaphore after transfer has ended.
    #[inline(always)]
    pub fn auto_acquire(&self, enabled: bool) -> &Self {
        self.0.shorts.write(|w| w.end_acquire().bit(enabled));
        self
    }

    /// Resets all events.
    #[inline(always)]
    pub fn reset_events(&self) {
        self.0.events_acquired.reset();
        self.0.events_end.reset();
        self.0.events_endrx.reset();
    }

    /// Resets specified event.
    #[inline(always)]
    pub fn reset_event(&self, event: SpisEvent) {
        match event {
            SpisEvent::Acquired => self.0.events_acquired.reset(),
            SpisEvent::End => self.0.events_end.reset(),
            SpisEvent::EndRx => self.0.events_endrx.reset(),
        };
    }

    /// Checks if specified event has been triggered.
    #[inline(always)]
    pub fn is_event_triggered(&self, event: SpisEvent) -> bool {
        match event {
            SpisEvent::Acquired => self.0.events_acquired.read().bits() != 0,
            SpisEvent::End => self.0.events_end.read().bits() != 0,
            SpisEvent::EndRx => self.0.events_endrx.read().bits() != 0,
        }
    }

    /// Checks if the granted transfer is done.
    #[inline(always)]
    pub fn is_done(&self) -> bool {
        self.0.events_end.read().bits() != 0 || self.0.events_endrx.read().bits() != 0
    }

    /// Checks if the semaphore is acquired.
    #[inline(always)]
    pub fn is_acquired(&self) -> bool {
        self.0.events_acquired.read().bits() != 0
    }

    /// Checks if last transaction overread.
    #[inline(always)]
    pub fn is_overread(&self) -> bool {
        self.0.status.read().overread().is_present()
    }

    /// Checks if last transaction overflowed.
    #[inline(always)]
    pub fn is_overflow(&self) -> bool {
        self.0.status.read().overflow().is_present()
    }

    /// Returns number of bytes received in last granted transaction.
    #[inline(always)]
    pub fn amount(&self) -> u32 {
        self.0.rxd.amount.read().bits()
    }

    /// Returns the semaphore status.
    #[inline(always)]
    pub fn semaphore_status(&self) -> SemaphoreStatus {
        match self.0.semstat.read().bits() {
            0 => SemaphoreStatus::Free,
            1 => SemaphoreStatus::CPU,
            2 => SemaphoreStatus::SPIS,
            _ => SemaphoreStatus::CPUPending,
        }
    }

    /// Returns reference to `Acquired` event endpoint for PPI.
    #[inline(always)]
    pub fn event_acquired(&self) -> &Reg<u32, _EVENTS_ACQUIRED> {
        &self.0.events_acquired
    }

    /// Returns reference to `End` event endpoint for PPI.
    #[inline(always)]
    pub fn event_end(&self) -> &Reg<u32, _EVENTS_END> {
        &self.0.events_end
    }

    /// Returns reference to `EndRx` event endpoint for PPI.
    #[inline(always)]
    pub fn event_end_rx(&self) -> &Reg<u32, _EVENTS_ENDRX> {
        &self.0.events_endrx
    }

    /// Returns reference to `Acquire` task endpoint for PPI.
    #[inline(always)]
    pub fn task_acquire(&self) -> &Reg<u32, _TASKS_ACQUIRE> {
        &self.0.tasks_acquire
    }

    /// Returns reference to `Release` task endpoint for PPI.
    #[inline(always)]
    pub fn task_release(&self) -> &Reg<u32, _TASKS_RELEASE> {
        &self.0.tasks_release
    }

    /// Receives data into the given `buffer` until it's filled.
    /// Buffer must be located in RAM.
    /// Returns a value that represents the in-progress DMA transfer.
    #[allow(unused_mut)]
    pub fn rx<W, B>(mut self, mut buffer: B) -> Result<Transfer<T, B>, Error>
    where
        B: WriteBuffer<Word = W>,
    {
        let (ptr, len) = unsafe { buffer.write_buffer() };
        let maxcnt = len * core::mem::size_of::<W>();
        if maxcnt > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }
        self.0
            .rxd
            .ptr
            .write(|w| unsafe { w.ptr().bits(ptr as u32) });
        self.0
            .rxd
            .maxcnt
            .write(|w| unsafe { w.bits(maxcnt as u32) });

        self.release();
        Ok(Transfer {
            inner: Some(Inner { buffer, spis: self }),
        })
    }

    /// Full duplex DMA transfer.
    /// Transmits the given buffer while simultaneously receiving data into the same buffer until it is filled.
    /// Buffer must be located in RAM.
    /// Returns a value that represents the in-progress DMA transfer.
    #[allow(unused_mut)]
    pub fn transfer<W, B>(mut self, mut buffer: B) -> Result<Transfer<T, B>, Error>
    where
        B: WriteBuffer<Word = W>,
    {
        let (ptr, len) = unsafe { buffer.write_buffer() };
        let maxcnt = len * core::mem::size_of::<W>();
        if maxcnt > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }
        self.0
            .txd
            .ptr
            .write(|w| unsafe { w.ptr().bits(ptr as u32) });
        self.0
            .rxd
            .ptr
            .write(|w| unsafe { w.ptr().bits(ptr as u32) });
        self.0
            .txd
            .maxcnt
            .write(|w| unsafe { w.bits(maxcnt as u32) });
        self.0
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
    ) -> Result<TransferSplit<T, TxB, RxB>, Error>
    where
        TxB: ReadBuffer<Word = TxW>,
        RxB: WriteBuffer<Word = RxW>,
    {
        let (rx_ptr, rx_len) = unsafe { rx_buffer.write_buffer() };
        let (tx_ptr, tx_len) = unsafe { tx_buffer.read_buffer() };
        let rx_maxcnt = rx_len * core::mem::size_of::<RxW>();
        let tx_maxcnt = tx_len * core::mem::size_of::<TxW>();
        if rx_maxcnt > EASY_DMA_SIZE || tx_maxcnt > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }
        if (tx_ptr as usize) < SRAM_LOWER || (tx_ptr as usize) > SRAM_UPPER {
            return Err(Error::DMABufferNotInDataMemory);
        }

        self.0
            .txd
            .ptr
            .write(|w| unsafe { w.ptr().bits(tx_ptr as u32) });
        self.0
            .rxd
            .ptr
            .write(|w| unsafe { w.ptr().bits(rx_ptr as u32) });
        self.0
            .rxd
            .maxcnt
            .write(|w| unsafe { w.bits(rx_maxcnt as u32) });
        self.0
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

    /// Transmits the given `tx_buffer`. Buffer must be located in RAM.
    /// Returns a value that represents the in-progress DMA transfer.
    #[allow(unused_mut)]
    pub fn tx<W, B>(mut self, buffer: B) -> Result<Transfer<T, B>, Error>
    where
        B: ReadBuffer<Word = W>,
    {
        let (ptr, len) = unsafe { buffer.read_buffer() };
        let maxcnt = len * core::mem::size_of::<W>();
        if maxcnt > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }
        if (ptr as usize) < SRAM_LOWER || (ptr as usize) > SRAM_UPPER {
            return Err(Error::DMABufferNotInDataMemory);
        }

        self.0
            .txd
            .ptr
            .write(|w| unsafe { w.ptr().bits(ptr as u32) });
        self.0
            .txd
            .maxcnt
            .write(|w| unsafe { w.bits(maxcnt as u32) });

        self.release();
        Ok(Transfer {
            inner: Some(Inner { buffer, spis: self }),
        })
    }

    /// Returns the raw interface to the underlying SPIS peripheral.
    pub fn free(self) -> T {
        self.0
    }
}

/// A DMA transfer
pub struct Transfer<T: Instance, B> {
    inner: Option<Inner<T, B>>,
}

struct Inner<T: Instance, B> {
    buffer: B,
    spis: Spis<T>,
}

impl<T: Instance, B> Transfer<T, B> {
    /// Blocks until the transfer is done and returns the buffer.
    pub fn wait(mut self) -> (B, Spis<T>) {
        let inner = self
            .inner
            .take()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        while !inner.spis.is_done() {}
        compiler_fence(Ordering::Acquire);
        (inner.buffer, inner.spis)
    }

    /// Checks if the granted transfer is done.
    #[inline(always)]
    pub fn is_done(&mut self) -> bool {
        let inner = self
            .inner
            .take()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        inner.spis.is_done()
    }
}

impl<T: Instance, B> Drop for Transfer<T, B> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            while !inner.spis.is_done() {}
            inner.spis.disable();
            compiler_fence(Ordering::Acquire);
        }
    }
}
/// A full duplex DMA transfer
pub struct TransferSplit<T: Instance, TxB, RxB> {
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
        let inner = self
            .inner
            .take()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        while !inner.spis.is_done() {}
        compiler_fence(Ordering::Acquire);
        (inner.tx_buffer, inner.rx_buffer, inner.spis)
    }

    /// Checks if the granted transfer is done.
    #[inline(always)]
    pub fn is_done(&mut self) -> bool {
        let inner = self
            .inner
            .take()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        inner.spis.is_done()
    }
}

impl<T: Instance, TxB, RxB> Drop for TransferSplit<T, TxB, RxB> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            while !inner.spis.is_done() {}
            inner.spis.disable();
            compiler_fence(Ordering::Acquire);
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
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::SPIS0 {}
    #[cfg(not(any(feature = "9160", feature = "52810")))]
    impl Sealed for super::SPIS1 {}
    #[cfg(not(any(feature = "9160", feature = "52810")))]
    impl Sealed for super::SPIS2 {}
}

pub trait Instance: sealed::Sealed + Deref<Target = spis0::RegisterBlock> {
    const INTERRUPT: Interrupt;
}

impl Instance for SPIS0 {
    #[cfg(not(any(feature = "9160", feature = "52810")))]
    const INTERRUPT: Interrupt = Interrupt::SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0;
    #[cfg(feature = "9160")]
    const INTERRUPT: Interrupt = Interrupt::UARTE0_SPIM0_SPIS0_TWIM0_TWIS0;
    #[cfg(feature = "52810")]
    const INTERRUPT: Interrupt = Interrupt::SPIM0_SPIS0_SPI0;
}

#[cfg(not(any(feature = "9160", feature = "52810")))]
impl Instance for SPIS1 {
    const INTERRUPT: Interrupt = Interrupt::SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1;
}

#[cfg(not(any(feature = "9160", feature = "52810")))]
impl Instance for SPIS2 {
    const INTERRUPT: Interrupt = Interrupt::SPIM2_SPIS2_SPI2;
}
