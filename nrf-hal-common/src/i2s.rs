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
use i2s::{_EVENTS_RXPTRUPD, _EVENTS_STOPPED, _EVENTS_TXPTRUPD, _TASKS_START, _TASKS_STOP};

pub struct I2S {
    i2s: I2S_PAC,
}

// I2S EasyDMA MAXCNT bit length = 14
const MAX_DMA_MAXCNT: u32 = 16_384;

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

        Self { i2s }
    }

    /// Takes ownership of the raw I2S peripheral, returning a safe wrapper i peripheral mode.
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
    pub fn start(&self) {
        self.i2s.tasks_start.write(|w| unsafe { w.bits(1) });
    }

    /// Stops the I2S transfer and waits until it has stopped.
    #[inline(always)]
    pub fn stop(&self) {
        compiler_fence(Ordering::SeqCst);
        self.i2s.tasks_stop.write(|w| unsafe { w.bits(1) });
        while self.i2s.events_stopped.read().bits() == 0 {}
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

    /// Sets the transmit data buffer (TX).
    /// NOTE: The TX buffer must live until the transfer is done, or corrupted data will be transmitted.
    #[inline(always)]
    pub fn tx_buffer<B: I2SBuffer + ?Sized>(&self, buf: &B) -> Result<(), Error> {
        if (buf.ptr() as usize) < SRAM_LOWER || (buf.ptr() as usize) > SRAM_UPPER {
            return Err(Error::DMABufferNotInDataMemory);
        }

        if buf.maxcnt() > MAX_DMA_MAXCNT {
            return Err(Error::BufferTooLong);
        }

        self.i2s
            .txd
            .ptr
            .write(|w| unsafe { w.ptr().bits(buf.ptr()) });
        self.i2s
            .rxtxd
            .maxcnt
            .write(|w| unsafe { w.bits(buf.maxcnt()) });

        Ok(())
    }

    /// Sets the receive data buffer (RX).
    #[inline(always)]
    pub fn rx_buffer<B: I2SBuffer + ?Sized>(&self, buf: &'static mut B) -> Result<(), Error> {
        if (buf.ptr() as usize) < SRAM_LOWER || (buf.ptr() as usize) > SRAM_UPPER {
            return Err(Error::DMABufferNotInDataMemory);
        }

        if buf.maxcnt() > MAX_DMA_MAXCNT {
            return Err(Error::BufferTooLong);
        }

        self.i2s
            .rxd
            .ptr
            .write(|w| unsafe { w.ptr().bits(buf.ptr()) });
        self.i2s
            .rxtxd
            .maxcnt
            .write(|w| unsafe { w.bits(buf.maxcnt()) });

        Ok(())
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
    Left,
    Right,
    Stereo,
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

/// Trait to represent valid sample buffers.
pub trait I2SBuffer {
    fn ptr(&self) -> u32;
    fn maxcnt(&self) -> u32;
}

impl I2SBuffer for [i8] {
    fn ptr(&self) -> u32 {
        self.as_ptr() as u32
    }
    fn maxcnt(&self) -> u32 {
        self.len() as u32 / 4
    }
}

impl I2SBuffer for [i16] {
    fn ptr(&self) -> u32 {
        self.as_ptr() as u32
    }
    fn maxcnt(&self) -> u32 {
        self.len() as u32 / 2
    }
}

impl I2SBuffer for [i32] {
    fn ptr(&self) -> u32 {
        self.as_ptr() as u32
    }
    fn maxcnt(&self) -> u32 {
        self.len() as u32
    }
}

impl I2SBuffer for [u8] {
    fn ptr(&self) -> u32 {
        self.as_ptr() as u32
    }
    fn maxcnt(&self) -> u32 {
        self.len() as u32 / 4
    }
}

impl I2SBuffer for [u16] {
    fn ptr(&self) -> u32 {
        self.as_ptr() as u32
    }
    fn maxcnt(&self) -> u32 {
        self.len() as u32 / 2
    }
}

impl I2SBuffer for [u32] {
    fn ptr(&self) -> u32 {
        self.as_ptr() as u32
    }
    fn maxcnt(&self) -> u32 {
        self.len() as u32
    }
}
