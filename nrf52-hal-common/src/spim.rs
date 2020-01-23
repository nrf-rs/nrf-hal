//! HAL interface to the SPIM peripheral
//!
//! See product specification, chapter 31.
use core::ops::Deref;
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};

#[cfg(feature = "9160")]
use crate::target::{spim0_ns as spim0, SPIM0_NS as SPIM0};

#[cfg(not(feature = "9160"))]
use crate::target::{spim0, SPIM0};

pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
pub use spim0::frequency::FREQUENCYW as Frequency;

#[cfg(any(feature = "52832", feature = "52840"))]
use crate::target::{SPIM1, SPIM2};

use crate::gpio::{Floating, Input, Output, Pin, PushPull};
use crate::target_constants::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::{slice_in_ram, slice_in_ram_or, DmaSlice};
use embedded_hal::digital::v2::OutputPin;

/// Interface to a SPIM instance
///
/// This is a very basic interface that comes with the following limitations:
/// - The SPIM instances share the same address space with instances of SPIS,
///   SPI, TWIM, TWIS, and TWI. You need to make sure that conflicting instances
///   are disabled before using `Spim`. See product specification, section 15.2.
#[derive(Debug)]
pub struct Spim<T>(T);

/// An ongoing transfer that was initiated by a call to `.transfer_polling()`.
///
/// This transfer must be polled to completion.  Failing to poll it until completed might leave the
/// peripheral in an inconsistent state.
#[derive(Debug)]
#[must_use = "This transfer must be polled to completion.  Failing to poll it until completed might leave the peripheral in an inconsistent state."]
pub struct SpiTransfer<'a, T> {
    state: TransferState<'a, T>,
    chip_select: &'a mut Pin<Output<PushPull>>,
    chunks: core::slice::ChunksMut<'a, u8>,
}

/// An ongoing transfer that was initiated by a call to `.transfer_even_polling()`.
///
/// This transfer must be polled to completion.  Failing to poll it until completed might leave the
/// peripheral in an inconsistent state.
#[derive(Debug)]
#[must_use = "This transfer must be polled to completion.  Failing to poll it until completed might leave the peripheral in an inconsistent state."]
pub struct SpiEvenTransfer<'a, T> {
    state: TransferState<'a, T>,
    chip_select: &'a mut Pin<Output<PushPull>>,
    chunks: core::iter::Zip<core::slice::Chunks<'a, u8>, core::slice::ChunksMut<'a, u8>>,
}

/// An ongoing transfer that was initiated by a call to `.transfer_uneven_polling()`.
///
/// This transfer must be polled to completion.  Failing to poll it until completed might leave the
/// peripheral in an inconsistent state.
#[derive(Debug)]
#[must_use = "This transfer must be polled to completion.  Failing to poll it until completed might leave the peripheral in an inconsistent state."]
pub struct SpiUnevenTransfer<'a, T> {
    state: TransferState<'a, T>,
    chip_select: &'a mut Pin<Output<PushPull>>,
    chunks_tx: core::slice::Chunks<'a, u8>,
    chunks_rx: core::slice::ChunksMut<'a, u8>,
}

/// The state of a more advanced transfer.
#[derive(Debug)]
enum TransferState<'a, T> {
    /// The spim is idle and awaiting the next chunk, no transfer is happening.
    Idle(&'a mut T),
    /// The spim is currently performing the specified transfer.
    Ongoing(SpiSingleTransfer<'a, T>),
    /// The spim transfer misbehaved, errored or panicked in a way that it cannot be completed.
    Inconsistent,
    /// The spim is done, and a new transfer can be started.
    Done,
}

/// An internal structure corresponding to a single in-progress EasyDMA transfer
#[derive(Debug)]
struct SpiSingleTransfer<'a, T> {
    spim: &'a mut T,
    tx_len: u32,
    rx_len: u32,
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

    pub fn new(spim: T, pins: Pins, frequency: Frequency, mode: Mode, orc: u8) -> Self {
        // Select pins
        spim.psel.sck.write(|w| {
            let w = unsafe { w.pin().bits(pins.sck.pin) };
            #[cfg(feature = "52840")]
            let w = w.port().bit(pins.sck.port);
            w.connect().connected()
        });

        match pins.mosi {
            Some(mosi) => spim.psel.mosi.write(|w| {
                let w = unsafe { w.pin().bits(mosi.pin) };
                #[cfg(feature = "52840")]
                let w = w.port().bit(mosi.port);
                w.connect().connected()
            }),
            None => spim.psel.mosi.write(|w| w.connect().disconnected()),
        }
        match pins.miso {
            Some(miso) => spim.psel.miso.write(|w| {
                let w = unsafe { w.pin().bits(miso.pin) };
                #[cfg(feature = "52840")]
                let w = w.port().bit(miso.port);
                w.connect().connected()
            }),
            None => spim.psel.miso.write(|w| w.connect().disconnected()),
        }

        // Enable SPIM instance
        spim.enable.write(|w| w.enable().enabled());

        // Configure mode
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

        // Configure frequency
        spim.frequency.write(|w| w.frequency().variant(frequency));

        // Set over-read character to `0`
        spim.orc.write(|w|
            // The ORC field is 8 bits long, so `0` is a valid value to write
            // there.
            unsafe { w.orc().bits(orc) });

        Spim(spim)
    }

    /// Internal helper function to setup and execute SPIM DMA transfer
    fn do_spi_dma_transfer(&mut self, tx: DmaSlice, rx: DmaSlice) -> Result<(), Error> {
        let mut transfer = SpiSingleTransfer::new(&mut self.0, tx, rx);

        while !transfer.poll_complete()? {
            core::sync::atomic::spin_loop_hint();
        }

        Ok(())
    }

    /// Read from an SPI slave
    ///
    /// This method is deprecated. Consider using `transfer` or `transfer_split`
    #[inline(always)]
    pub fn read(
        &mut self,
        chip_select: &mut Pin<Output<PushPull>>,
        tx_buffer: &[u8],
        rx_buffer: &mut [u8],
    ) -> Result<(), Error> {
        self.transfer_split_uneven(chip_select, tx_buffer, rx_buffer)
    }

    /// Read and write from a SPI slave, using a single buffer
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
        let mut transfer = SpiTransfer::new(&mut self.0, chip_select, buffer)?;

        while !transfer.poll_complete()? {
            core::sync::atomic::spin_loop_hint();
        }

        Ok(())
    }

    /// Read and write from a SPI slave, using a single buffer.
    ///
    /// This is an async polling version of `transfer()`.  You need to call
    /// `.poll_complete()` on the returned object until it returns `true`.  A
    /// good time to do that would be after receiving a SPI interrupt, for
    /// example.
    ///
    /// This method implements a complete read transaction, which consists of
    /// the master transmitting what it wishes to read, and the slave responding
    /// with the requested data.
    ///
    /// Uses the provided chip select pin to initiate the transaction. Transmits
    /// all bytes in `buffer`, then receives an equal number of bytes.
    pub fn transfer_polling<'a>(
        &'a mut self,
        chip_select: &'a mut Pin<Output<PushPull>>,
        buffer: &'a mut [u8],
    ) -> Result<SpiTransfer<'a, T>, Error> {
        SpiTransfer::new(&mut self.0, chip_select, buffer)
    }

    /// Read and write from a SPI slave, using separate read and write buffers
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
        let mut transfer = SpiEvenTransfer::new(&mut self.0, chip_select, tx_buffer, rx_buffer)?;

        while !transfer.poll_complete()? {
            core::sync::atomic::spin_loop_hint();
        }

        Ok(())
    }

    /// Read and write from a SPI slave, using separate read and write buffers
    ///
    /// This is an async polling version of `transfer_split_even()`.  You need to
    /// call `.poll_complete()` on the returned object until it returns `true`.
    /// A good time to do that would be after receiving a SPI interrupt, for
    /// example.
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
    pub fn transfer_split_even_polling<'a>(
        &'a mut self,
        chip_select: &'a mut Pin<Output<PushPull>>,
        tx_buffer: &'a [u8],
        rx_buffer: &'a mut [u8],
    ) -> Result<SpiEvenTransfer<'a, T>, Error> {
        SpiEvenTransfer::new(&mut self.0, chip_select, tx_buffer, rx_buffer)
    }

    /// Read and write from a SPI slave, using separate read and write buffers
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
        let mut transfer = SpiUnevenTransfer::new(&mut self.0, chip_select, tx_buffer, rx_buffer)?;

        while !transfer.poll_complete()? {
            core::sync::atomic::spin_loop_hint();
        }

        Ok(())
    }

    /// Read and write from a SPI slave, using separate read and write buffers
    ///
    /// This is an async polling version of `transfer_split_uneven()`.  You need to
    /// call `.poll_complete()` on the returned object until it returns `true`.
    /// A good time to do that would be after receiving a SPI interrupt, for
    /// example.
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
    pub fn transfer_split_uneven_polling<'a>(
        &'a mut self,
        chip_select: &'a mut Pin<Output<PushPull>>,
        tx_buffer: &'a [u8],
        rx_buffer: &'a mut [u8],
    ) -> Result<SpiUnevenTransfer<'a, T>, Error> {
        SpiUnevenTransfer::new(&mut self.0, chip_select, tx_buffer, rx_buffer)
    }

    /// Write to an SPI slave
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

    /// Return the raw interface to the underlying SPIM peripheral
    pub fn free(self) -> T {
        self.0
    }
}

impl<'a, T> SpiTransfer<'a, T>
where
    T: Instance,
{
    pub fn new(
        spim: &'a mut T,
        chip_select: &'a mut Pin<Output<PushPull>>,
        buffer: &'a mut [u8],
    ) -> Result<Self, Error> {
        slice_in_ram_or(buffer, Error::DMABufferNotInDataMemory)?;

        let chunks = buffer.chunks_mut(EASY_DMA_SIZE);

        chip_select.set_low().unwrap();

        let state = TransferState::new(spim);

        Ok(Self {
            state,
            chip_select,
            chunks,
        })
    }

    pub fn poll_complete(&mut self) -> Result<bool, Error> {
        let chunks = &mut self.chunks;
        self.state.advance(|| {
            chunks
                .next()
                .map(|chunk| (DmaSlice::from_slice(chunk), DmaSlice::from_slice(chunk)))
        })
    }
}

impl<'a, T> Drop for SpiTransfer<'a, T> {
    fn drop(&mut self) {
        self.chip_select.set_high().unwrap();
    }
}

impl<'a, T> SpiEvenTransfer<'a, T>
where
    T: Instance,
{
    pub fn new(
        spim: &'a mut T,
        chip_select: &'a mut Pin<Output<PushPull>>,
        tx_buffer: &'a [u8],
        rx_buffer: &'a mut [u8],
    ) -> Result<Self, Error> {
        // NOTE: RAM slice check for `rx_buffer` is not necessary, as a mutable
        // slice can only be built from data located in RAM
        slice_in_ram_or(tx_buffer, Error::DMABufferNotInDataMemory)?;

        let txi = tx_buffer.chunks(EASY_DMA_SIZE);
        let rxi = rx_buffer.chunks_mut(EASY_DMA_SIZE);
        let chunks = txi.zip(rxi);

        chip_select.set_low().unwrap();

        let state = TransferState::new(spim);

        Ok(Self {
            state,
            chip_select,
            chunks,
        })
    }

    pub fn poll_complete(&mut self) -> Result<bool, Error> {
        let chunks = &mut self.chunks;
        self.state.advance(|| {
            chunks
                .next()
                .map(|(tx, rx)| (DmaSlice::from_slice(tx), DmaSlice::from_slice(rx)))
        })
    }
}

impl<'a, T> Drop for SpiEvenTransfer<'a, T> {
    fn drop(&mut self) {
        self.chip_select.set_high().unwrap();
    }
}

impl<'a, T> SpiUnevenTransfer<'a, T>
where
    T: Instance,
{
    pub fn new(
        spim: &'a mut T,
        chip_select: &'a mut Pin<Output<PushPull>>,
        tx_buffer: &'a [u8],
        rx_buffer: &'a mut [u8],
    ) -> Result<Self, Error> {
        // NOTE: RAM slice check for `rx_buffer` is not necessary, as a mutable
        // slice can only be built from data located in RAM
        slice_in_ram_or(tx_buffer, Error::DMABufferNotInDataMemory)?;

        let chunks_tx = tx_buffer.chunks(EASY_DMA_SIZE);
        let chunks_rx = rx_buffer.chunks_mut(EASY_DMA_SIZE);

        chip_select.set_low().unwrap();

        let state = TransferState::new(spim);

        Ok(Self {
            state,
            chip_select,
            chunks_tx,
            chunks_rx,
        })
    }

    pub fn poll_complete(&mut self) -> Result<bool, Error> {
        let chunks_tx = &mut self.chunks_tx;
        let chunks_rx = &mut self.chunks_rx;
        self.state
            .advance(|| match (chunks_tx.next(), chunks_rx.next()) {
                (None, None) => None,
                (tx, rx) => Some((
                    tx.map(|tx| DmaSlice::from_slice(tx))
                        .unwrap_or(DmaSlice::null()),
                    rx.map(|rx| DmaSlice::from_slice(rx))
                        .unwrap_or(DmaSlice::null()),
                )),
            })
    }
}

impl<'a, T> Drop for SpiUnevenTransfer<'a, T> {
    fn drop(&mut self) {
        self.chip_select.set_high().unwrap();
    }
}

impl<'a, T> TransferState<'a, T>
where
    T: Instance,
{
    fn new(spim: &'a mut T) -> Self {
        TransferState::Idle(spim)
    }

    fn advance(
        &mut self,
        mut next_chunk: impl FnMut() -> Option<(DmaSlice, DmaSlice)>,
    ) -> Result<bool, Error> {
        loop {
            match core::mem::replace(self, TransferState::Inconsistent) {
                TransferState::Idle(spim) => match next_chunk() {
                    Some((tx, rx)) => {
                        let transfer = SpiSingleTransfer::new(spim, tx, rx);
                        *self = TransferState::Ongoing(transfer);
                        return Ok(false);
                    }
                    None => *self = TransferState::Done,
                },
                TransferState::Ongoing(mut transfer) => {
                    if transfer.poll_complete()? {
                        *self = TransferState::Idle(transfer.spim)
                    } else {
                        *self = TransferState::Ongoing(transfer);
                        return Ok(false);
                    }
                }
                TransferState::Done => {
                    *self = TransferState::Done;
                    return Ok(true);
                }
                TransferState::Inconsistent => return Err(Error::InconsistentState),
            }
        }
    }
}

impl<'a, T> SpiSingleTransfer<'a, T>
where
    T: Instance,
{
    fn new(spim: &'a mut T, tx: DmaSlice, rx: DmaSlice) -> Self {
        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        compiler_fence(SeqCst);

        // Set up the DMA write
        spim.txd.ptr.write(|w| unsafe { w.ptr().bits(tx.ptr) });

        spim.txd.maxcnt.write(|w|
            // Note that that nrf52840 maxcnt is a wider
            // type than a u8, so we use a `_` cast rather than a `u8` cast.
            // The MAXCNT field is thus at least 8 bits wide and accepts the full
            // range of values that fit in a `u8`.
            unsafe { w.maxcnt().bits(tx.len as _) });

        // Set up the DMA read
        spim.rxd.ptr.write(|w|
            // This is safe for the same reasons that writing to TXD.PTR is
            // safe. Please refer to the explanation there.
            unsafe { w.ptr().bits(rx.ptr) });
        spim.rxd.maxcnt.write(|w|
            // This is safe for the same reasons that writing to TXD.MAXCNT is
            // safe. Please refer to the explanation there.
            unsafe { w.maxcnt().bits(rx.len as _) });

        // Start SPI transaction
        spim.tasks_start.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) });

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed
        compiler_fence(SeqCst);

        let tx_len = tx.len;
        let rx_len = rx.len;

        SpiSingleTransfer {
            spim,
            tx_len,
            rx_len,
        }
    }

    fn poll_complete(&mut self) -> Result<bool, Error> {
        // Check for END event
        //
        // This event is triggered once both transmitting and receiving are
        // done.
        if self.spim.events_end.read().bits() == 0 {
            // Reset the event, otherwise it will always read `1` from now on.
            self.spim.events_end.write(|w| w);

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // after all possible DMA actions have completed
            compiler_fence(SeqCst);

            if self.spim.txd.amount.read().bits() != self.tx_len {
                return Err(Error::Transmit);
            }

            if self.spim.rxd.amount.read().bits() != self.rx_len {
                return Err(Error::Receive);
            }

            Ok(true)
        } else {
            Ok(false)
        }
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
    /// EasyDMA can only read from data memory, read only buffers in flash will fail
    DMABufferNotInDataMemory,
    Transmit,
    Receive,
    /// The peripheral is in an inconsistent state, because of encountering an error mid-transfer.
    InconsistentState,
}

/// Implemented by all SPIM instances
pub trait Instance: Deref<Target = spim0::RegisterBlock> {}

impl Instance for SPIM0 {}

#[cfg(any(feature = "52832", feature = "52840"))]
impl Instance for SPIM1 {}

#[cfg(any(feature = "52832", feature = "52840"))]
impl Instance for SPIM2 {}
