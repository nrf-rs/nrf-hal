//! HAL interface to the SPIM peripheral
//!
//! See product specification, chapter 31.
use core::ops::Deref;
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};
use core::cmp::min;
pub use crate::target::spim0::frequency::FREQUENCYW as Frequency;
pub use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};

use crate::target::{spim0, SPIM0};

#[cfg(any(feature = "52832", feature = "52840"))]
use crate::target::{SPIM1, SPIM2};

use crate::target_constants::{EASY_DMA_SIZE,SRAM_LOWER,SRAM_UPPER,FORCE_COPY_BUFFER_SIZE};
use crate::prelude::*;
use crate::gpio::{
    Pin,
    Floating,
    Input,
    Output,
    PushPull,
};

pub trait SpimExt : Deref<Target=spim0::RegisterBlock> + Sized {
    fn constrain(self, pins: Pins, frequency: Frequency, mode: Mode, orc: u8) -> Spim<Self>;
}

macro_rules! impl_spim_ext {
    ($($spim:ty,)*) => {
        $(
            impl SpimExt for $spim {
                fn constrain(self, pins: Pins, frequency: Frequency, mode: Mode, orc: u8) -> Spim<Self> {
                    Spim::new(self, pins, frequency, mode, orc)
                }
            }
        )*
    }
}

impl_spim_ext!(SPIM0,);

#[cfg(any(feature = "52832", feature = "52840"))]
impl_spim_ext!(
    SPIM1,
    SPIM2,
);

/// Interface to a SPIM instance
///
/// This is a very basic interface that comes with the following limitations:
/// - The SPIM instances share the same address space with instances of SPIS,
///   SPI, TWIM, TWIS, and TWI. You need to make sure that conflicting instances
///   are disabled before using `Spim`. See product specification, section 15.2.
/// - The SPI mode is hardcoded to SPI mode 0.
/// - The frequency is hardcoded to 500 kHz.
/// - The over-read character is hardcoded to `0`.
pub struct Spim<T>(T);

impl<T> embedded_hal::blocking::spi::Transfer<u8> for Spim<T> where T: SpimExt
{
   type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Error> {
        let mut offset:usize = 0;
        while offset < words.len() {
            let data_len = min(EASY_DMA_SIZE, words.len() - offset);
            let data_ptr = offset + (words.as_ptr() as usize);
            offset += data_len;

            self.do_spi_dma_transfer(data_ptr as u32,data_len as u32,data_ptr as u32,data_len as u32,|_|{})?;

        }
        Ok(words)
    }
}
impl<T> embedded_hal::blocking::spi::Write<u8> for Spim<T> where T: SpimExt
{
   type Error = Error;

    fn write<'w>(&mut self, words: &'w [u8]) -> Result<(), Error> {
        let p = words.as_ptr() as usize;
        // Mask on segment where Data RAM is located on nrf52840 and nrf52832
        // Upper limit is choosen to entire area where DataRam can be placed
        if SRAM_LOWER <= p && p < SRAM_UPPER {
            let mut offset:usize = 0;
            while offset < words.len() {
                let data_len = min(EASY_DMA_SIZE, words.len() - offset);
                let data_ptr = offset + (words.as_ptr() as usize);
                offset += data_len;

                // setup spi dma tx buffer and 0 for read buffer length
                self.do_spi_dma_transfer(data_ptr as u32,data_len as u32,0,0,|_|{})?;
            }
        } else {
            // Force copy from flash mode.
            let blocksize = min(EASY_DMA_SIZE,FORCE_COPY_BUFFER_SIZE);
            let mut buffer:[u8;FORCE_COPY_BUFFER_SIZE] = [0;FORCE_COPY_BUFFER_SIZE];
            let mut offset:usize = 0;
            while offset < words.len() {
                let data_len = min(blocksize, words.len() - offset);
                for i in 0..data_len{
                    buffer[i] = words[offset+i];
                }
                offset += data_len;
                // setup spi dma tx buffer and 0 for read buffer length
                self.do_spi_dma_transfer(buffer.as_ptr() as u32,data_len as u32,0,0,|_|{})?;
            }
        }
        Ok(())

    }
}
impl<T> Spim<T> where T: SpimExt {
    pub fn new(spim: T, pins: Pins, frequency: Frequency, mode: Mode, orc: u8) -> Self {
        // Select pins
        spim.psel.sck.write(|w| {
            let w = unsafe { w.pin().bits(pins.sck.pin) };
            #[cfg(feature = "52840")]
            let w = w.port().bit(pins.sck.port);
            w.connect().connected()
        });

        match pins.mosi {
            Some(mosi) => {
                spim.psel.mosi.write(|w| {
                    let w = unsafe { w.pin().bits(mosi.pin) };
                    #[cfg(feature = "52840")]
                    let w = w.port().bit(mosi.port);
                    w.connect().connected()
                })
            }
            None =>{
                spim.psel.mosi.write(|w| {
                    w.connect().disconnected()
                })
            }
        }
        match pins.miso {
            Some(miso) => {
                spim.psel.miso.write(|w| {
                    let w = unsafe { w.pin().bits(miso.pin) };
                    #[cfg(feature = "52840")]
                    let w = w.port().bit(miso.port);
                    w.connect().connected()
                })
            }
            None => {
                spim.psel.miso.write(|w| {
                    w.connect().disconnected()
                })
            }
        }

        // Enable SPIM instance
        spim.enable.write(|w|
            w.enable().enabled()
        );

        // Configure mode
        spim.config.write(|w| {
            // Can't match on `mode` due to embedded-hal, see https://github.com/rust-embedded/embedded-hal/pull/126
            if mode == MODE_0 {
                w.order().msb_first().cpol().active_high().cpha().leading()
            } else if mode == MODE_1 {
                w.order().msb_first().cpol().active_high().cpha().trailing()
            } else if mode == MODE_2 {
                w.order().msb_first().cpol().active_low().cpha().leading()
            } else {
                w.order().msb_first().cpol().active_low().cpha().trailing()
            }
        });

        // Configure frequency
        spim.frequency.write(|w|
                w.frequency().variant(frequency)
        );

        // Set over-read character to `0`
        spim.orc.write(|w|
            // The ORC field is 8 bits long, so `0` is a valid value to write
            // there.
            unsafe { w.orc().bits(orc) });

        Spim(spim)
    }

    /// Internal helper function to setup and execute SPIM DMA transfer
    fn  do_spi_dma_transfer<CSFun>(&mut self,
            tx_data_ptr:u32,
            tx_len:u32,
            rx_data_ptr:u32,
            rx_len:u32,
            mut cs_n:CSFun
            ) -> Result<(), Error>
            where CSFun: FnMut(bool)
    {
        // Check If buffer is in data RAM, compiler sometimes put static data
        // in flash this area is not accessable by EasyDMA
        if (tx_data_ptr as usize) < SRAM_LOWER || tx_data_ptr as usize >= SRAM_UPPER {
            return Err(Error::DMABufferNotInDataMemory)
        }
        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        compiler_fence(SeqCst);
        // set CS inactive
        cs_n(false);
        // Set up the DMA write
        self.0.txd.ptr.write(|w|
            unsafe { w.ptr().bits(tx_data_ptr) }
        );
        self.0.txd.maxcnt.write(|w|
            // Note that that nrf52840 maxcnt is a wider
            // type than a u8, so we use a `_` cast rather than a `u8` cast.
            // The MAXCNT field is thus at least 8 bits wide and accepts the full
            // range of values that fit in a `u8`.
            unsafe { w.maxcnt().bits(tx_len as _ ) }
        );

        // Set up the DMA read
        self.0.rxd.ptr.write(|w|
            // This is safe for the same reasons that writing to TXD.PTR is
            // safe. Please refer to the explanation there.
            unsafe { w.ptr().bits(rx_data_ptr ) }
        );
        self.0.rxd.maxcnt.write(|w|
            // This is safe for the same reasons that writing to TXD.MAXCNT is
            // safe. Please refer to the explanation there.
            unsafe { w.maxcnt().bits(rx_len as _) }
        );

        // Set CS active
        cs_n(true);
        // Start SPI transaction
        self.0.tasks_start.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) }
        );

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed
        compiler_fence(SeqCst);

        // Wait for END event
        //
        // This event is triggered once both transmitting and receiving are
        // done.
        while self.0.events_end.read().bits() == 0 {}

        // Reset the event, otherwise it will always read `1` from now on.
        self.0.events_end.write(|w| w);

        // Transfer done - set cs inactive
        cs_n(false);
        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed
        compiler_fence(SeqCst);

        if self.0.txd.amount.read().bits() != tx_len {
            return Err(Error::Transmit);
        }
        if self.0.rxd.amount.read().bits() != rx_len {
            return Err(Error::Receive);
        }
        Ok(())
    }

    /// Read from an SPI slave
    ///
    /// This method implements a complete read transaction, which consists of
    /// the master transmitting what it wishes to read, and the slave responding
    /// with the requested data.
    ///
    /// Uses the provided chip select pin to initiate the transaction. Transmits
    /// all bytes in `tx_buffer`, then receives bytes until `rx_buffer` is full.
    /// Both buffer must have a length of at most 255 bytes.
    pub fn read(&mut self,
        chip_select: &mut Pin<Output<PushPull>>,
        tx_buffer  : &[u8],
        rx_buffer  : &mut [u8],
    )
        -> Result<(), Error>
    {
        if tx_buffer.len() > EASY_DMA_SIZE {
            return Err(Error::TxBufferTooLong);
        }
        if rx_buffer.len() > EASY_DMA_SIZE {
            return Err(Error::RxBufferTooLong);
        }

        self.do_spi_dma_transfer(
            // Set up the DMA write
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the SPI transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.

            // tx_data_ptr:
            tx_buffer.as_ptr() as u32,

            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to the type of maxcnt
            // is also fine.
            //
            // Note that that nrf52840 maxcnt is a wider
            // type than a u8, so we use a `_` cast rather than a `u8` cast.
            // The MAXCNT field is thus at least 8 bits wide and accepts the full
            // range of values that fit in a `u8`.

            // tx_len:
            tx_buffer.len() as _,

            // Set up the DMA read
            // This is safe for the same reasons that writing to TXD.PTR is
            // safe. Please refer to the explanation there.

            // rx_data_ptr:
            rx_buffer.as_mut_ptr() as u32,
            // This is safe for the same reasons that writing to TXD.MAXCNT is
            // safe. Please refer to the explanation there.

            // rx_len:
            rx_buffer.len() as _,
            // chip select callback
            |cs|{if cs {chip_select.set_low()} else {chip_select.set_high()} }
        )
    }

    /// Write to an SPI slave
    ///
    /// This method uses the provided chip select pin to initiate the
    /// transaction, then transmits all bytes in `tx_buffer`.
    ///

    pub fn write(&mut self,
        chip_select: &mut Pin<Output<PushPull>>,
        tx_buffer  : &[u8],
    )
        -> Result<(), Error>
    {

        if tx_buffer.len() > EASY_DMA_SIZE {
            return Err(Error::TxBufferTooLong);
        }

        // Set up the DMA write
        self.do_spi_dma_transfer(
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the SPI transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            tx_buffer.as_ptr() as u32,
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to `u8` is also fine.
            //
            // The MAXCNT field is 8 bits wide and accepts the full range of
            // values.
            tx_buffer.len() as _,
            // Tell the RXD channel it doesn't need to read anything
            0 , 0,
            |cs|{if cs {chip_select.set_low()} else {chip_select.set_high()} }
        )
    }

    /// Return the raw interface to the underlying SPIM peripheral
    pub fn free(self) -> T {
        self.0
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
}
