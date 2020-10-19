//! HAL interface for the I2S peripheral.
//!

#[cfg(not(feature = "9160"))]
use crate::pac::{i2s, I2S as I2S_PAC};
#[cfg(feature = "9160")]
use crate::pac::{i2s_ns as i2s, I2S_NS as I2S_PAC};
use crate::{
    gpio::{Floating, Input, Output, Pin, PushPull},
    pac::generic::Reg,
    target_constants::{SRAM_LOWER, SRAM_UPPER},
};
use core::sync::atomic::{compiler_fence, Ordering};
use embedded_dma::*;

use i2s::{_EVENTS_RXPTRUPD, _EVENTS_STOPPED, _EVENTS_TXPTRUPD, _TASKS_START, _TASKS_STOP};

pub struct I2S {
    i2s: I2S_PAC,
}

// I2S EasyDMA MAXCNT bit length = 14
const MAX_DMA_MAXCNT: u32 = 1 << 14;

impl I2S {
    /// Takes ownership of the raw I2S peripheral, returning a safe wrapper in controller mode.
    pub fn new_controller(
        i2s: I2S_PAC,
        mck_pin: Option<&Pin<Output<PushPull>>>,
        sck_pin: &Pin<Output<PushPull>>,
        lrck_pin: &Pin<Output<PushPull>>,
        sdin_pin: Option<&Pin<Input<Floating>>>,
        sdout_pin: Option<&Pin<Output<PushPull>>>,
    ) -> Self {
        i2s.config.mcken.write(|w| w.mcken().enabled());
        i2s.config.mckfreq.write(|w| w.mckfreq()._32mdiv16());
        i2s.config.ratio.write(|w| w.ratio()._192x());
        i2s.config.mode.write(|w| w.mode().master());
        i2s.config.swidth.write(|w| w.swidth()._16bit());
        i2s.config.align.write(|w| w.align().left());
        i2s.config.format.write(|w| w.format().i2s());
        i2s.config.channels.write(|w| w.channels().stereo());

        if let Some(p) = mck_pin {
            i2s.psel.mck.write(|w| {
                unsafe { w.pin().bits(p.pin()) };
                #[cfg(any(feature = "52833", feature = "52840"))]
                w.port().bit(p.port().bit());
                w.connect().connected()
            });
        }

        i2s.psel.sck.write(|w| {
            unsafe { w.pin().bits(sck_pin.pin()) };
            #[cfg(any(feature = "52833", feature = "52840"))]
            w.port().bit(sck_pin.port().bit());
            w.connect().connected()
        });

        i2s.psel.lrck.write(|w| {
            unsafe { w.pin().bits(lrck_pin.pin()) };
            #[cfg(any(feature = "52833", feature = "52840"))]
            w.port().bit(lrck_pin.port().bit());
            w.connect().connected()
        });

        if let Some(p) = sdin_pin {
            i2s.psel.sdin.write(|w| {
                unsafe { w.pin().bits(p.pin()) };
                #[cfg(any(feature = "52833", feature = "52840"))]
                w.port().bit(p.port().bit());
                w.connect().connected()
            });
        }

        if let Some(p) = sdout_pin {
            i2s.psel.sdout.write(|w| {
                unsafe { w.pin().bits(p.pin()) };
                #[cfg(any(feature = "52833", feature = "52840"))]
                w.port().bit(p.port().bit());
                w.connect().connected()
            });
        }

        i2s.enable.write(|w| w.enable().enabled());
        Self { i2s }
    }

    /// Takes ownership of the raw I2S peripheral, returning a safe wrapper in peripheral mode.
    pub fn new_peripheral(
        i2s: I2S_PAC,
        mck_pin: Option<&Pin<Input<Floating>>>,
        sck_pin: &Pin<Input<Floating>>,
        lrck_pin: &Pin<Input<Floating>>,
        sdin_pin: Option<&Pin<Input<Floating>>>,
        sdout_pin: Option<&Pin<Output<PushPull>>>,
    ) -> Self {
        i2s.config.txen.write(|w| w.txen().enabled());
        i2s.config.rxen.write(|w| w.rxen().enabled());
        i2s.config.mode.write(|w| w.mode().slave());
        i2s.config.swidth.write(|w| w.swidth()._16bit());
        i2s.config.align.write(|w| w.align().left());
        i2s.config.format.write(|w| w.format().i2s());
        i2s.config.channels.write(|w| w.channels().stereo());

        if let Some(p) = mck_pin {
            i2s.psel.mck.write(|w| {
                unsafe { w.pin().bits(p.pin()) };
                #[cfg(any(feature = "52833", feature = "52840"))]
                w.port().bit(p.port().bit());
                w.connect().connected()
            });
        }

        i2s.psel.sck.write(|w| {
            unsafe { w.pin().bits(sck_pin.pin()) };
            #[cfg(any(feature = "52833", feature = "52840"))]
            w.port().bit(sck_pin.port().bit());
            w.connect().connected()
        });

        i2s.psel.lrck.write(|w| {
            unsafe { w.pin().bits(lrck_pin.pin()) };
            #[cfg(any(feature = "52833", feature = "52840"))]
            w.port().bit(lrck_pin.port().bit());
            w.connect().connected()
        });

        if let Some(p) = sdin_pin {
            i2s.psel.sdin.write(|w| {
                unsafe { w.pin().bits(p.pin()) };
                #[cfg(any(feature = "52833", feature = "52840"))]
                w.port().bit(p.port().bit());
                w.connect().connected()
            });
        }

        if let Some(p) = sdout_pin {
            i2s.psel.sdout.write(|w| {
                unsafe { w.pin().bits(p.pin()) };
                #[cfg(any(feature = "52833", feature = "52840"))]
                w.port().bit(p.port().bit());
                w.connect().connected()
            });
        }

        i2s.enable.write(|w| w.enable().enabled());
        Self { i2s }
    }

    /// Enables the I2S module.
    #[inline(always)]
    pub fn enable(&self) -> &Self {
        self.i2s.enable.write(|w| w.enable().enabled());
        self
    }

    /// Disables the I2S module.
    #[inline(always)]
    pub fn disable(&self) -> &Self {
        self.i2s.enable.write(|w| w.enable().disabled());
        self
    }

    /// Starts I2S transfer.
    #[inline(always)]
    pub fn start(&self) -> &Self {
        self.enable();
        self.i2s.tasks_start.write(|w| unsafe { w.bits(1) });
        self
    }

    /// Stops the I2S transfer and waits until it has stopped.
    #[inline(always)]
    pub fn stop(&self) -> &Self {
        compiler_fence(Ordering::SeqCst);
        self.i2s.tasks_stop.write(|w| unsafe { w.bits(1) });
        while self.i2s.events_stopped.read().bits() == 0 {}
        self
    }

    /// Enables/disables I2S transmission (TX).
    #[inline(always)]
    pub fn set_tx_enabled(&self, enabled: bool) -> &Self {
        self.i2s.config.txen.write(|w| w.txen().bit(enabled));
        self
    }

    /// Enables/disables I2S reception (RX).
    #[inline(always)]
    pub fn set_rx_enabled(&self, enabled: bool) -> &Self {
        self.i2s.config.rxen.write(|w| w.rxen().bit(enabled));
        self
    }

    /// Sets MCK generator frequency.
    #[inline(always)]
    pub fn set_mck_frequency(&self, freq: MckFreq) -> &Self {
        self.i2s
            .config
            .mckfreq
            .write(|w| unsafe { w.mckfreq().bits(freq.into()) });
        self
    }

    /// Sets MCK / LRCK ratio.
    #[inline(always)]
    pub fn set_ratio(&self, ratio: Ratio) -> &Self {
        self.i2s
            .config
            .ratio
            .write(|w| unsafe { w.ratio().bits(ratio.into()) });
        self
    }

    /// Sets sample width.
    #[inline(always)]
    pub fn set_sample_width(&self, width: SampleWidth) -> &Self {
        self.i2s
            .config
            .swidth
            .write(|w| unsafe { w.swidth().bits(width.into()) });
        self
    }

    /// Sets the sample alignment within a frame.
    #[inline(always)]
    pub fn set_align(&self, align: Align) -> &Self {
        self.i2s.config.align.write(|w| w.align().bit(align.into()));
        self
    }

    /// Sets the frame format.
    #[inline(always)]
    pub fn set_format(&self, format: Format) -> &Self {
        self.i2s
            .config
            .format
            .write(|w| w.format().bit(format.into()));
        self
    }

    /// Sets the I2S channel configuation.
    #[inline(always)]
    pub fn set_channels(&self, channels: Channels) -> &Self {
        self.i2s
            .config
            .channels
            .write(|w| unsafe { w.channels().bits(channels.into()) });
        self
    }

    /// Returns the I2S channel configuation.
    #[inline(always)]
    pub fn channels(&self) -> Channels {
        match self.i2s.config.channels.read().bits() {
            0 => Channels::Stereo,
            1 => Channels::Left,
            _ => Channels::Right,
        }
    }

    /// Receives data into the given `buffer` until it's filled.
    /// Buffer address must be 4 byte aligned and located in RAM.
    /// Returns a value that represents the in-progress DMA transfer.
    #[allow(unused_mut)]
    pub fn rx<W, B>(mut self, mut buffer: B) -> Result<Transfer<B>, Error>
    where
        W: SupportedWordSize,
        B: WriteBuffer<Word = W> + 'static,
    {
        let (ptr, len) = unsafe { buffer.write_buffer() };
        if ptr as u32 % 4 != 0 {
            return Err(Error::BufferMisaligned);
        }
        let maxcnt = (len / (core::mem::size_of::<u32>() / core::mem::size_of::<W>())) as u32;
        if maxcnt > MAX_DMA_MAXCNT {
            return Err(Error::BufferTooLong);
        }
        self.i2s
            .rxd
            .ptr
            .write(|w| unsafe { w.ptr().bits(ptr as u32) });
        self.i2s.rxtxd.maxcnt.write(|w| unsafe { w.bits(maxcnt) });
        Ok(Transfer {
            inner: Some(Inner { buffer, i2s: self }),
        })
    }

    /// Full duplex DMA transfer.
    /// Transmits the given `tx_buffer` while simultaneously receiving data
    /// into the given `rx_buffer` until it is filled.
    /// The buffers must be of equal size and their addresses must be 4 byte aligned and located in RAM.
    /// Returns a value that represents the in-progress DMA transfer.
    #[allow(unused_mut)]
    pub fn transfer<W, TxB, RxB>(
        mut self,
        tx_buffer: TxB,
        mut rx_buffer: RxB,
    ) -> Result<TransferFullDuplex<TxB, RxB>, Error>
    where
        W: SupportedWordSize,
        TxB: ReadBuffer<Word = W> + 'static,
        RxB: WriteBuffer<Word = W> + 'static,
    {
        let (rx_ptr, rx_len) = unsafe { rx_buffer.write_buffer() };
        let (tx_ptr, tx_len) = unsafe { tx_buffer.read_buffer() };
        if tx_ptr as u32 % 4 != 0 || rx_ptr as u32 % 4 != 0 {
            return Err(Error::BufferMisaligned);
        }
        let maxcnt = (tx_len / (core::mem::size_of::<u32>() / core::mem::size_of::<W>())) as u32;
        if tx_len != rx_len {
            return Err(Error::BuffersDontMatch);
        }
        if maxcnt > MAX_DMA_MAXCNT {
            return Err(Error::BufferTooLong);
        }
        if (tx_ptr as usize) < SRAM_LOWER || (tx_ptr as usize) > SRAM_UPPER {
            return Err(Error::DMABufferNotInDataMemory);
        }

        self.i2s
            .txd
            .ptr
            .write(|w| unsafe { w.ptr().bits(tx_ptr as u32) });
        self.i2s
            .rxd
            .ptr
            .write(|w| unsafe { w.ptr().bits(rx_ptr as u32) });
        self.i2s.rxtxd.maxcnt.write(|w| unsafe { w.bits(maxcnt) });

        Ok(TransferFullDuplex {
            inner: Some(InnerFullDuplex {
                tx_buffer,
                rx_buffer,
                i2s: self,
            }),
        })
    }

    /// Transmits the given `tx_buffer`.
    /// Buffer address must be 4 byte aligned and located in RAM.
    /// Returns a value that represents the in-progress DMA transfer.
    #[allow(unused_mut)]
    pub fn tx<W, B>(mut self, buffer: B) -> Result<Transfer<B>, Error>
    where
        W: SupportedWordSize,
        B: ReadBuffer<Word = W> + 'static,
    {
        let (ptr, len) = unsafe { buffer.read_buffer() };
        if ptr as u32 % 4 != 0 {
            return Err(Error::BufferMisaligned);
        }
        let maxcnt = (len / (core::mem::size_of::<u32>() / core::mem::size_of::<W>())) as u32;
        if maxcnt > MAX_DMA_MAXCNT {
            return Err(Error::BufferTooLong);
        }
        if (ptr as usize) < SRAM_LOWER || (ptr as usize) > SRAM_UPPER {
            return Err(Error::DMABufferNotInDataMemory);
        }

        self.i2s
            .txd
            .ptr
            .write(|w| unsafe { w.ptr().bits(ptr as u32) });
        self.i2s.rxtxd.maxcnt.write(|w| unsafe { w.bits(maxcnt) });

        Ok(Transfer {
            inner: Some(Inner { buffer, i2s: self }),
        })
    }

    /// Sets the transmit buffer RAM start address.
    #[inline(always)]
    pub fn set_tx_ptr(&self, addr: u32) -> Result<(), Error> {
        if (addr as usize) < SRAM_LOWER || (addr as usize) > SRAM_UPPER {
            return Err(Error::DMABufferNotInDataMemory);
        }
        self.i2s.txd.ptr.write(|w| unsafe { w.ptr().bits(addr) });
        Ok(())
    }

    /// Sets the receive buffer RAM start address.
    #[inline(always)]
    pub unsafe fn set_rx_ptr(&self, addr: u32) -> Result<(), Error> {
        if (addr as usize) < SRAM_LOWER || (addr as usize) > SRAM_UPPER {
            return Err(Error::DMABufferNotInDataMemory);
        }
        self.i2s.rxd.ptr.write(|w| w.ptr().bits(addr));
        Ok(())
    }

    /// Sets the size (in 32bit words) of the receive and transmit buffers.
    #[inline(always)]
    pub unsafe fn set_buffersize(&self, n_32bit: u32) -> Result<(), Error> {
        if n_32bit > MAX_DMA_MAXCNT {
            return Err(Error::BufferTooLong);
        }
        self.i2s.rxtxd.maxcnt.write(|w| w.bits(n_32bit));
        Ok(())
    }

    /// Checks if an event has been triggered.
    #[inline(always)]
    pub fn is_event_triggered(&self, event: I2SEvent) -> bool {
        match event {
            I2SEvent::Stopped => self.i2s.events_stopped.read().bits() != 0,
            I2SEvent::RxPtrUpdated => self.i2s.events_rxptrupd.read().bits() != 0,
            I2SEvent::TxPtrUpdated => self.i2s.events_txptrupd.read().bits() != 0,
        }
    }

    /// Marks event as handled.
    #[inline(always)]
    pub fn reset_event(&self, event: I2SEvent) {
        match event {
            I2SEvent::Stopped => self.i2s.events_stopped.reset(),
            I2SEvent::RxPtrUpdated => self.i2s.events_rxptrupd.reset(),
            I2SEvent::TxPtrUpdated => self.i2s.events_txptrupd.reset(),
        }
    }

    /// Enables interrupt triggering on the specified event.
    #[inline(always)]
    pub fn enable_interrupt(&self, event: I2SEvent) -> &Self {
        match event {
            I2SEvent::Stopped => self.i2s.intenset.modify(|_r, w| w.stopped().set()),
            I2SEvent::RxPtrUpdated => self.i2s.intenset.modify(|_r, w| w.rxptrupd().set()),
            I2SEvent::TxPtrUpdated => self.i2s.intenset.modify(|_r, w| w.txptrupd().set()),
        };
        self
    }

    /// Disables interrupt triggering on the specified event.
    #[inline(always)]
    pub fn disable_interrupt(&self, event: I2SEvent) -> &Self {
        match event {
            I2SEvent::Stopped => self.i2s.intenclr.modify(|_r, w| w.stopped().clear()),
            I2SEvent::RxPtrUpdated => self.i2s.intenclr.modify(|_r, w| w.rxptrupd().clear()),
            I2SEvent::TxPtrUpdated => self.i2s.intenclr.modify(|_r, w| w.txptrupd().clear()),
        };
        self
    }

    /// Returns reference to `Stopped` event endpoint for PPI.
    #[inline(always)]
    pub fn event_stopped(&self) -> &Reg<u32, _EVENTS_STOPPED> {
        &self.i2s.events_stopped
    }

    /// Returns reference to `RxPtrUpdated` event endpoint for PPI.
    #[inline(always)]
    pub fn event_rx_ptr_updated(&self) -> &Reg<u32, _EVENTS_RXPTRUPD> {
        &self.i2s.events_rxptrupd
    }

    /// Returns reference to `TxPtrUpdated` event endpoint for PPI.
    #[inline(always)]
    pub fn event_tx_ptr_updated(&self) -> &Reg<u32, _EVENTS_TXPTRUPD> {
        &self.i2s.events_txptrupd
    }

    /// Returns reference to `Start` task endpoint for PPI.
    #[inline(always)]
    pub fn task_start(&self) -> &Reg<u32, _TASKS_START> {
        &self.i2s.tasks_start
    }

    /// Returns reference to `Stop` task endpoint for PPI.
    #[inline(always)]
    pub fn task_stop(&self) -> &Reg<u32, _TASKS_STOP> {
        &self.i2s.tasks_stop
    }

    /// Consumes `self` and returns back the raw peripheral.
    pub fn free(self) -> I2S_PAC {
        self.disable();
        self.i2s
    }
}

#[derive(Debug)]
pub enum Error {
    DMABufferNotInDataMemory,
    BufferTooLong,
    BuffersDontMatch,
    BufferMisaligned,
}

/// I2S Mode
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Mode {
    Controller,
    Peripheral,
}

/// Master clock generator frequency.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum MckFreq {
    _32MDiv8,
    _32MDiv10,
    _32MDiv11,
    _32MDiv15,
    _32MDiv16,
    _32MDiv21,
    _32MDiv23,
    _32MDiv30,
    _32MDiv31,
    _32MDiv32,
    _32MDiv42,
    _32MDiv63,
    _32MDiv125,
}
impl From<MckFreq> for u32 {
    fn from(variant: MckFreq) -> Self {
        variant as _
    }
}

/// MCK / LRCK ratio.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Ratio {
    _32x,
    _48x,
    _64x,
    _96x,
    _128x,
    _192x,
    _256x,
    _384x,
    _512x,
}
impl From<Ratio> for u8 {
    fn from(variant: Ratio) -> Self {
        variant as _
    }
}

/// Sample width.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SampleWidth {
    _8bit,
    _16bit,
    _24bit,
}
impl From<SampleWidth> for u8 {
    fn from(variant: SampleWidth) -> Self {
        variant as _
    }
}

/// Alignment of sample within a frame.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Align {
    Left,
    Right,
}
impl From<Align> for bool {
    fn from(variant: Align) -> Self {
        match variant {
            Align::Left => false,
            Align::Right => true,
        }
    }
}

/// Frame format.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Format {
    I2S,
    Aligned,
}
impl From<Format> for bool {
    fn from(variant: Format) -> Self {
        match variant {
            Format::I2S => false,
            Format::Aligned => true,
        }
    }
}

/// Enable channels.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Channels {
    Stereo,
    Left,
    Right,
}
impl From<Channels> for u8 {
    fn from(variant: Channels) -> Self {
        variant as _
    }
}

/// I2S events
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum I2SEvent {
    RxPtrUpdated,
    TxPtrUpdated,
    Stopped,
}

/// A DMA transfer
pub struct Transfer<B> {
    // FIXME: Always `Some`, only using `Option` here to allow moving fields out of `inner`.
    inner: Option<Inner<B>>,
}

struct Inner<B> {
    buffer: B,
    i2s: I2S,
}

impl<B> Transfer<B> {
    /// Blocks until the transfer is done and returns the buffer.
    pub fn wait(mut self) -> (B, I2S) {
        let inner = self
            .inner
            .take()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        while !(inner.i2s.is_event_triggered(I2SEvent::RxPtrUpdated)
            || inner.i2s.is_event_triggered(I2SEvent::TxPtrUpdated))
        {}
        compiler_fence(Ordering::Acquire);
        (inner.buffer, inner.i2s)
    }
}

impl<B> Drop for Transfer<B> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            inner.i2s.stop();
            compiler_fence(Ordering::Acquire);
        }
    }
}
/// A full duplex DMA transfer
pub struct TransferFullDuplex<TxB, RxB> {
    // FIXME: Always `Some`, only using `Option` here to allow moving fields out of `inner`.
    inner: Option<InnerFullDuplex<TxB, RxB>>,
}

struct InnerFullDuplex<TxB, RxB> {
    tx_buffer: TxB,
    rx_buffer: RxB,
    i2s: I2S,
}

impl<TxB, RxB> TransferFullDuplex<TxB, RxB> {
    /// Blocks until the transfer is done and returns the buffer.
    pub fn wait(mut self) -> (TxB, RxB, I2S) {
        let inner = self
            .inner
            .take()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        while !(inner.i2s.is_event_triggered(I2SEvent::RxPtrUpdated)
            || inner.i2s.is_event_triggered(I2SEvent::TxPtrUpdated))
        {}
        compiler_fence(Ordering::Acquire);
        (inner.tx_buffer, inner.rx_buffer, inner.i2s)
    }
}

impl<TxB, RxB> Drop for TransferFullDuplex<TxB, RxB> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            inner.i2s.stop();
            compiler_fence(Ordering::Acquire);
        }
    }
}

pub trait SupportedWordSize: private::Sealed {}
impl private::Sealed for i8 {}
impl SupportedWordSize for i8 {}
impl private::Sealed for u8 {}
impl SupportedWordSize for u8 {}
impl private::Sealed for i16 {}
impl SupportedWordSize for i16 {}
impl private::Sealed for u16 {}
impl SupportedWordSize for u16 {}
impl private::Sealed for i32 {}
impl SupportedWordSize for i32 {}
impl private::Sealed for u32 {}
impl SupportedWordSize for u32 {}

mod private {
    /// Prevents code outside of the parent module from implementing traits.
    pub trait Sealed {}
}
