//! HAL interface to the SPIM peripheral.
//!
//! See product specification, chapter 31.

use core::ops::Deref;
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};

#[cfg(feature = "9160")]
use crate::pac::{spim0_ns as spim0, SPIM0_NS as SPIM0};

#[cfg(not(feature = "9160"))]
use crate::pac::{spim0, SPIM0};

pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
pub use spim0::frequency::FREQUENCY_A as Frequency;

use core::iter::repeat_with;
use core::sync::atomic::Ordering;

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
use crate::pac::{SPIM1, SPIM2};

#[cfg(any(feature = "52833", feature = "52840"))]
use crate::pac::SPIM3;

use crate::gpio::{Floating, Input, Output, Pin, PushPull};
use crate::target_constants::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::{slice_in_ram, slice_in_ram_or, DmaSlice};
use embedded_hal::digital::v2::OutputPin;
use embedded_dma::*;
use crate::target_constants::{
    SRAM_UPPER,
    SRAM_LOWER,
};

/// Interface to a SPIM instance.
///
/// This is a very basic interface that comes with the following limitations:
/// - The SPIM instances share the same address space with instances of SPIS,
///   SPI, TWIM, TWIS, and TWI. You need to make sure that conflicting instances
///   are disabled before using `Spim`. See product specification, section 15.2.
pub struct Spim<T> {
    periph: T,
    pins: Pins,
}

impl<T> embedded_hal::blocking::spi::Transfer<u8> for Spim<T>
where
    T: Instance,
{
    type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Error> {
        // If the slice isn't in RAM, we can't write back to it at all
        slice_in_ram_or(words, Error::DMABufferNotInDataMemory)?;

        words.chunks(EASY_DMA_SIZE).try_for_each(|chunk| {
            self.do_spi_dma_transfer(DmaSlice::from_slice(chunk), DmaSlice::from_slice(chunk))
        })?;

        Ok(words)
    }
}

impl<T> embedded_hal::blocking::spi::Write<u8> for Spim<T>
where
    T: Instance,
{
    type Error = Error;

    fn write<'w>(&mut self, words: &'w [u8]) -> Result<(), Error> {
        // Mask on segment where Data RAM is located on nrf52840 and nrf52832
        // Upper limit is choosen to entire area where DataRam can be placed
        let needs_copy = !slice_in_ram(words);

        let chunk_sz = if needs_copy {
            FORCE_COPY_BUFFER_SIZE
        } else {
            EASY_DMA_SIZE
        };

        let step = if needs_copy {
            Self::spi_dma_copy
        } else {
            Self::spi_dma_no_copy
        };

        words.chunks(chunk_sz).try_for_each(|c| step(self, c))
    }
}
impl<T> Spim<T>
where
    T: Instance,
{
    fn spi_dma_no_copy(&mut self, chunk: &[u8]) -> Result<(), Error> {
        self.do_spi_dma_transfer(DmaSlice::from_slice(chunk), DmaSlice::null())
    }

    fn spi_dma_copy(&mut self, chunk: &[u8]) -> Result<(), Error> {
        let mut buf = [0u8; FORCE_COPY_BUFFER_SIZE];
        buf[..chunk.len()].copy_from_slice(chunk);

        self.do_spi_dma_transfer(DmaSlice::from_slice(&buf[..chunk.len()]), DmaSlice::null())
    }

    pub fn new(spim: T, mut pins: Pins, frequency: Frequency, mode: Mode, orc: u8) -> Self {
        // Select pins.
        spim.psel.sck.write(|w| {
            let w = unsafe { w.pin().bits(pins.sck.pin()) };
            #[cfg(any(feature = "52843", feature = "52840"))]
            let w = w.port().bit(pins.sck.port().bit());
            w.connect().connected()
        });

        match pins.mosi.as_mut() {
            Some(mosi) => spim.psel.mosi.write(|w| {
                let w = unsafe { w.pin().bits(mosi.pin()) };
                #[cfg(any(feature = "52843", feature = "52840"))]
                let w = w.port().bit(mosi.port().bit());
                w.connect().connected()
            }),
            None => spim.psel.mosi.write(|w| w.connect().disconnected()),
        }
        match pins.miso.as_mut() {
            Some(miso) => spim.psel.miso.write(|w| {
                let w = unsafe { w.pin().bits(miso.pin()) };
                #[cfg(any(feature = "52843", feature = "52840"))]
                let w = w.port().bit(miso.port().bit());
                w.connect().connected()
            }),
            None => spim.psel.miso.write(|w| w.connect().disconnected()),
        }

        // Enable SPIM instance.
        spim.enable.write(|w| w.enable().enabled());

        // Configure mode.
        spim.config.write(|w| {
            // Can't match on `mode` due to embedded-hal, see https://github.com/rust-embedded/embedded-hal/pull/126
            if mode == MODE_0 {
                w.order().msb_first();
                w.cpol().active_high();
                w.cpha().leading();
            } else if mode == MODE_1 {
                w.order().msb_first();
                w.cpol().active_high();
                w.cpha().trailing();
            } else if mode == MODE_2 {
                w.order().msb_first();
                w.cpol().active_low();
                w.cpha().leading();
            } else {
                w.order().msb_first();
                w.cpol().active_low();
                w.cpha().trailing();
            }
            w
        });

        // Configure frequency.
        spim.frequency.write(|w| w.frequency().variant(frequency));

        // Set over-read character to `0`.
        spim.orc.write(|w|
            // The ORC field is 8 bits long, so `0` is a valid value to write
            // there.
            unsafe { w.orc().bits(orc) });

        Spim {
            periph: spim,
            pins,
        }
    }

    /// Internal helper function to setup and execute SPIM DMA transfer.
    fn do_spi_dma_transfer(&mut self, tx: DmaSlice, rx: DmaSlice) -> Result<(), Error> {
        self.start_spi_dma_transfer(&tx, &rx);

        // Wait for END event.
        //
        // This event is triggered once both transmitting and receiving are
        // done.
        while !self.is_spi_dma_transfer_complete() {}

        self.complete_spi_dma_transfer(&tx, &rx).map(drop)
    }

    fn start_spi_dma_transfer(&mut self, tx: &DmaSlice, rx: &DmaSlice) {
        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started.
        compiler_fence(SeqCst);

        // Set up the DMA write.
        self.periph.txd.ptr.write(|w| unsafe { w.ptr().bits(tx.ptr) });

        self.periph.txd.maxcnt.write(|w|
            // Note that that nrf52840 maxcnt is a wider.
            // type than a u8, so we use a `_` cast rather than a `u8` cast.
            // The MAXCNT field is thus at least 8 bits wide and accepts the full
            // range of values that fit in a `u8`.
            unsafe { w.maxcnt().bits(tx.len as _ ) });

        // Set up the DMA read.
        self.periph.rxd.ptr.write(|w|
            // This is safe for the same reasons that writing to TXD.PTR is
            // safe. Please refer to the explanation there.
            unsafe { w.ptr().bits(rx.ptr) });
        self.periph.rxd.maxcnt.write(|w|
            // This is safe for the same reasons that writing to TXD.MAXCNT is
            // safe. Please refer to the explanation there.
            unsafe { w.maxcnt().bits(rx.len as _) });

        // Start SPI transaction.
        self.periph.tasks_start.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) });

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed.
        compiler_fence(SeqCst);
    }

    #[inline(always)]
    fn is_spi_dma_transfer_complete(&mut self) -> bool {
        self.periph.events_end.read().bits() != 0
    }

    fn complete_spi_dma_transfer(&mut self, tx: &DmaSlice, rx: &DmaSlice) -> Result<(usize, usize), Error> {
        // Reset the event, otherwise it will always read `1` from now on.
        self.periph.events_end.write(|w| w);

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed.
        compiler_fence(SeqCst);

        let rx_amt = self.periph.rxd.amount.read().bits();
        let tx_amt = self.periph.txd.amount.read().bits();

        if tx_amt != tx.len {
            return Err(Error::Transmit);
        }
        if rx_amt != rx.len {
            return Err(Error::Receive);
        }

        Ok((rx_amt as usize, tx_amt as usize))
    }

    /// Read and write from a SPI slave, using a single buffer.
    ///
    /// This method implements a complete read transaction, which consists of
    /// the master transmitting what it wishes to read, and the slave responding
    /// with the requested data.
    ///
    /// Uses the provided chip select pin to initiate the transaction. Transmits
    /// all bytes in `buffer`, then receives an equal number of bytes.
    pub fn transfer(
        &mut self,
        chip_select: &mut Pin<Output<PushPull>>,
        buffer: &mut [u8],
    ) -> Result<(), Error> {
        slice_in_ram_or(buffer, Error::DMABufferNotInDataMemory)?;

        chip_select.set_low().unwrap();

        // Don't return early, as we must reset the CS pin.
        let res = buffer.chunks(EASY_DMA_SIZE).try_for_each(|chunk| {
            self.do_spi_dma_transfer(DmaSlice::from_slice(chunk), DmaSlice::from_slice(chunk))
        });

        chip_select.set_high().unwrap();

        res
    }

    /// Read and write from a SPI slave, using separate read and write buffers.
    ///
    /// This method implements a complete read transaction, which consists of
    /// the master transmitting what it wishes to read, and the slave responding
    /// with the requested data.
    ///
    /// Uses the provided chip select pin to initiate the transaction. Transmits
    /// all bytes in `tx_buffer`, then receives bytes until `rx_buffer` is full.
    ///
    /// If `tx_buffer.len() != rx_buffer.len()`, the transaction will stop at the
    /// smaller of either buffer.
    pub fn transfer_split_even(
        &mut self,
        chip_select: &mut Pin<Output<PushPull>>,
        tx_buffer: &[u8],
        rx_buffer: &mut [u8],
    ) -> Result<(), Error> {
        // NOTE: RAM slice check for `rx_buffer` is not necessary, as a mutable
        // slice can only be built from data located in RAM.
        slice_in_ram_or(tx_buffer, Error::DMABufferNotInDataMemory)?;

        let txi = tx_buffer.chunks(EASY_DMA_SIZE);
        let rxi = rx_buffer.chunks_mut(EASY_DMA_SIZE);

        chip_select.set_low().unwrap();

        // Don't return early, as we must reset the CS pin
        let res = txi.zip(rxi).try_for_each(|(t, r)| {
            self.do_spi_dma_transfer(DmaSlice::from_slice(t), DmaSlice::from_slice(r))
        });

        chip_select.set_high().unwrap();

        res
    }

    /// Read and write from a SPI slave, using separate read and write buffers.
    ///
    /// This method implements a complete read transaction, which consists of
    /// the master transmitting what it wishes to read, and the slave responding
    /// with the requested data.
    ///
    /// Uses the provided chip select pin to initiate the transaction. Transmits
    /// all bytes in `tx_buffer`, then receives bytes until `rx_buffer` is full.
    ///
    /// This method is more complicated than the other `transfer` methods because
    /// it is allowed to perform transactions where `tx_buffer.len() != rx_buffer.len()`.
    /// If this occurs, extra incoming bytes will be discarded, OR extra outgoing bytes
    /// will be filled with the `orc` value.
    pub fn transfer_split_uneven(
        &mut self,
        chip_select: &mut Pin<Output<PushPull>>,
        tx_buffer: &[u8],
        rx_buffer: &mut [u8],
    ) -> Result<(), Error> {
        // NOTE: RAM slice check for `rx_buffer` is not necessary, as a mutable
        // slice can only be built from data located in RAM.
        slice_in_ram_or(tx_buffer, Error::DMABufferNotInDataMemory)?;

        // For the tx and rx, we want to return Some(chunk)
        // as long as there is data to send. We then chain a repeat to
        // the end so once all chunks have been exhausted, we will keep
        // getting Nones out of the iterators.
        let txi = tx_buffer
            .chunks(EASY_DMA_SIZE)
            .map(Some)
            .chain(repeat_with(|| None));

        let rxi = rx_buffer
            .chunks_mut(EASY_DMA_SIZE)
            .map(Some)
            .chain(repeat_with(|| None));

        chip_select.set_low().unwrap();

        // We then chain the iterators together, and once BOTH are feeding
        // back Nones, then we are done sending and receiving.
        //
        // Don't return early, as we must reset the CS pin.
        let res = txi
            .zip(rxi)
            .take_while(|(t, r)| t.is_some() || r.is_some())
            // We also turn the slices into either a DmaSlice (if there was data), or a null
            // DmaSlice (if there is no data).
            .map(|(t, r)| {
                (
                    t.map(|t| DmaSlice::from_slice(t))
                        .unwrap_or_else(DmaSlice::null),
                    r.map(|r| DmaSlice::from_slice(r))
                        .unwrap_or_else(DmaSlice::null),
                )
            })
            .try_for_each(|(t, r)| self.do_spi_dma_transfer(t, r));

        chip_select.set_high().unwrap();

        res
    }

    /// Write to an SPI slave.
    ///
    /// This method uses the provided chip select pin to initiate the
    /// transaction, then transmits all bytes in `tx_buffer`. All incoming
    /// bytes are discarded.
    pub fn write(
        &mut self,
        chip_select: &mut Pin<Output<PushPull>>,
        tx_buffer: &[u8],
    ) -> Result<(), Error> {
        slice_in_ram_or(tx_buffer, Error::DMABufferNotInDataMemory)?;
        self.transfer_split_uneven(chip_select, tx_buffer, &mut [0u8; 0])
    }

    /// Return the raw interface to the underlying SPIM peripheral.
    pub fn free(self) -> (T, Pins) {
        (self.periph, self.pins)
    }

    pub fn dma_transfer_split<TxW, RxW, TxB, RxB>(
        mut self,
        tx_buffer: TxB,
        mut rx_buffer: RxB,
    ) -> Result<TransferSplit<T, TxB, RxB>, (Self, Error)>
    where
        TxB: ReadBuffer<Word = TxW>,
        RxB: WriteBuffer<Word = RxW>,
    {

        let tx_dma = rb_to_dma_slice(&tx_buffer);
        let rx_dma = wb_to_dma_slice(&mut rx_buffer);

        if rx_dma.len.max(tx_dma.len) as usize > EASY_DMA_SIZE {
            return Err((self, Error::TxBufferTooLong));
        }
        if (tx_dma.ptr as usize) < SRAM_LOWER || (tx_dma.ptr as usize) > SRAM_UPPER {
            return Err((self, Error::DMABufferNotInDataMemory));
        }

        // tx, rx
        self.start_spi_dma_transfer(
            &tx_dma,
            &rx_dma,
        );

        Ok(TransferSplit { inner: Some(InnerSplit {
            tx_buffer,
            rx_buffer,
            spim: self,
        })})
    }
}

/// GPIO pins for SPIM interface
pub struct Pins {
    /// SPI clock
    pub sck: Pin<Output<PushPull>>,

    /// MOSI Master out, slave in
    /// None if unused
    pub mosi: Option<Pin<Output<PushPull>>>,

    /// MISO Master in, slave out
    /// None if unused
    pub miso: Option<Pin<Input<Floating>>>,
}

#[derive(Debug)]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    /// EasyDMA can only read from data memory, read only buffers in flash will fail.
    DMABufferNotInDataMemory,
    Transmit,
    Receive,
}

/// Implemented by all SPIM instances.
pub trait Instance: Deref<Target = spim0::RegisterBlock> {}

impl Instance for SPIM0 {}

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
impl Instance for SPIM1 {}

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
impl Instance for SPIM2 {}

#[cfg(any(feature = "52833", feature = "52840"))]
impl Instance for SPIM3 {}

pub struct TransferSplit<T: Instance, TxB, RxB>
where
    TxB: ReadBuffer,
    RxB: WriteBuffer,
{
    inner: Option<InnerSplit<T, TxB, RxB>>,
}

pub struct InnerSplit<T: Instance, TxB, RxB>
where
    TxB: ReadBuffer,
    RxB: WriteBuffer,
{
    tx_buffer: TxB,
    rx_buffer: RxB,
    spim: Spim<T>,
}


#[inline(always)]
fn rb_to_dma_slice<RB: ReadBuffer>(rb: &RB) -> DmaSlice {
    let (ptr, len) = unsafe { rb.read_buffer() };
    DmaSlice {
        ptr: ptr as usize as u32,
        len: (len * core::mem::size_of::<RB::Word>()) as u32,
    }
}

#[inline(always)]
fn wb_to_dma_slice<WB: WriteBuffer>(wb: &mut WB) -> DmaSlice {
    let (ptr, len) = unsafe { wb.write_buffer() };
    DmaSlice {
        ptr: ptr as usize as u32,
        len: (len * core::mem::size_of::<WB::Word>()) as u32,
    }
}

impl<T: Instance, TxB, RxB> TransferSplit<T, TxB, RxB>
where
    TxB: ReadBuffer,
    RxB: WriteBuffer,
{
    /// Blocks until the transfer is done and returns the buffer.
    pub fn wait(mut self) -> (TxB, RxB, Spim<T>) {
        compiler_fence(Ordering::SeqCst);

        let mut inner = self
            .inner
            .take()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });

        while !inner.spim.is_spi_dma_transfer_complete() {}

        // tx, rx
        inner.spim.complete_spi_dma_transfer(
            &rb_to_dma_slice(&inner.tx_buffer),
            &wb_to_dma_slice(&mut inner.rx_buffer),
        ).ok();

        (inner.tx_buffer, inner.rx_buffer, inner.spim)
    }

    // TODO: We should probably add `bail` method like `spis`, but it would
    // require thinking about how to clean up, and potentially re-enable.

    /// Checks if the granted transfer is done.
    #[inline(always)]
    pub fn is_done(&mut self) -> bool {
        let inner = self
            .inner
            .as_mut()
            .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
        inner.spim.is_spi_dma_transfer_complete()
    }
}

impl<T: Instance, TxB, RxB> Drop for TransferSplit<T, TxB, RxB>
where
    TxB: ReadBuffer,
    RxB: WriteBuffer,
{
    fn drop(&mut self) {
        if let Some(_inner) = self.inner.take() {
            compiler_fence(Ordering::SeqCst);
            inner.spim.periph.enable.write(|w| w.enable().disabled());
        }
    }
}
