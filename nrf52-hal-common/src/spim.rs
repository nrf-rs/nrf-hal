//! HAL interface to the SPIM peripheral
//!
//! See product specification, chapter 31.
use embedded_hal::blocking::spi::Transfer;
use core::ops::Deref;
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};
use core::cmp::min;

use crate::target::{
    spim0,
    SPIM0,
    SPIM1,
    SPIM2,
};

use crate::prelude::*;
use crate::gpio::{
    p0::P0_Pin,
    Floating,
    Input,
    Output,
    PushPull,
};


pub trait SpimExt : Deref<Target=spim0::RegisterBlock> + Sized {
    fn constrain(self, pins: Pins) -> Spim<Self>;
}

macro_rules! impl_spim_ext {
    ($($spim:ty,)*) => {
        $(
            impl SpimExt for $spim {
                fn constrain(self, pins: Pins) -> Spim<Self> {
                    Spim::new(self, pins)
                }
            }
        )*
    }
}

impl_spim_ext!(
    SPIM0,
    SPIM1,
    SPIM2,
);

#[cfg(feature = "52832")]
const MAX_SPI_DMA_SIZE: u32 = 255;    // NRF52832 1..0xFF
#[cfg(feature = "52840")]
const MAX_SPI_DMA_SIZE: u32 = 65535; // NRF52840 1..0xFFFF

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

impl<T> Transfer<u8> for Spim<T> where T: SpimExt
{
   type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Error> {
        let mut offset:u32 = 0;
        while offset < words.len() as u32 {
            let datalen = min(MAX_SPI_DMA_SIZE, (words.len() as u32) - offset);
            let dataptr = offset + (words.as_ptr() as u32);
            offset += MAX_SPI_DMA_SIZE;

            self.do_spi_dma_transfer(dataptr,datalen,dataptr,datalen)?;

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account DMA
            compiler_fence(SeqCst);
        }
        Ok(words)
    }
}

impl<T> Spim<T> where T: SpimExt {
    pub fn new(spim: T, pins: Pins) -> Self {
        // Select pins
        spim.psel.sck.write(|w| {
            let w = unsafe { w.pin().bits(pins.sck.pin) };
            w.connect().connected()
        });
        spim.psel.mosi.write(|w| {
            let w = unsafe { w.pin().bits(pins.mosi.pin) };
            w.connect().connected()
        });
        spim.psel.miso.write(|w| {
            let w = unsafe { w.pin().bits(pins.miso.pin) };
            w.connect().connected()
        });

        // Enable SPIM instance
        spim.enable.write(|w|
            w.enable().enabled()
        );

        // Set to SPI mode 0
        spim.config.write(|w|
            w
                .order().msb_first()
                .cpha().leading()
                .cpol().active_high()
        );

        // Configure frequency
        spim.frequency.write(|w|
            w.frequency().k500() // 500 kHz
        );

        // Set over-read character to `0`
        spim.orc.write(|w|
            // The ORC field is 8 bits long, so `0` is a valid value to write
            // there.
            unsafe { w.orc().bits(0) }
        );

        Spim(spim)
    }

    /// Internal helper function to setup SPIM DMA transfer
    fn  do_spi_dma_transfer(&mut self,
            tx_data_ptr:u32,
            tx_len:u32,
            rx_data_ptr:u32,
            rx_len:u32) -> Result<(), Error>
    {
        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        compiler_fence(SeqCst);

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
        chip_select: &mut P0_Pin<Output<PushPull>>,
        tx_buffer  : &[u8],
        rx_buffer  : &mut [u8],
    )
        -> Result<(), Error>
    {
        if tx_buffer.len() > MAX_SPI_DMA_SIZE as usize {
            return Err(Error::TxBufferTooLong);
        }
        if rx_buffer.len() > MAX_SPI_DMA_SIZE as usize {
            return Err(Error::RxBufferTooLong);
        }

        // Pull chip select pin low, which is the active state
        chip_select.set_low();

        let err = self.do_spi_dma_transfer(
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

            //tx_len:
            tx_buffer.len() as _,

        // Set up the DMA read
            // This is safe for the same reasons that writing to TXD.PTR is
            // safe. Please refer to the explanation there.

            //rx_data_ptr:
            rx_buffer.as_mut_ptr() as u32,
            // This is safe for the same reasons that writing to TXD.MAXCNT is
            // safe. Please refer to the explanation there.

            //rx_len:
            rx_buffer.len() as _
        );

        // End SPI transaction
        chip_select.set_high();
        return err;
    }

    /// Write to an SPI slave
    ///
    /// This method uses the provided chip select pin to initiate the
    /// transaction, then transmits all bytes in `tx_buffer`.
    ///

    pub fn write(&mut self,
        chip_select: &mut P0_Pin<Output<PushPull>>,
        tx_buffer  : &[u8],
    )
        -> Result<(), Error>
    {

        if tx_buffer.len() > MAX_SPI_DMA_SIZE as usize {
            return Err(Error::TxBufferTooLong);
        }

        chip_select.set_low();
        // Set up the DMA write
        let err = self.do_spi_dma_transfer(

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
            0 , 0
        );
        // End SPI transaction
        chip_select.set_high();
        return err;
    }

    /// Return the raw interface to the underlying SPIM peripheral
    pub fn free(self) -> T {
        self.0
    }
}


pub struct Pins {
    // SPI clock
    pub sck: P0_Pin<Output<PushPull>>,

    // Master out, slave in
    pub mosi: P0_Pin<Output<PushPull>>,

    // Master in, slave out
    pub miso: P0_Pin<Input<Floating>>,
}


#[derive(Debug)]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    Transmit,
    Receive,
}
