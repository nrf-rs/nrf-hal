//! HAL interface to the TWIM peripheral
//!
//! See product specification:
//!
//! - nrf52832: Section 33
//! - nrf52840: Section 6.31
use core::ops::Deref;
use core::cmp::min;

use core::sync::atomic::{compiler_fence, Ordering::SeqCst};

use crate::target::{
    twim0,
    P0,
    TWIM0,
    TWIM1,
};

use crate::gpio::{
    p0::P0_Pin,
    Floating,
    Input,
};

use crate::target_constants::EASY_DMA_SIZE;

pub use crate::target::twim0::frequency::FREQUENCYW as Frequency;


pub trait TwimExt: Deref<Target=twim0::RegisterBlock> + Sized {
    fn constrain(self, pins: Pins, frequency: Frequency)
        -> Twim<Self>;
}

macro_rules! impl_twim_ext {
    ($($twim:ty,)*) => {
        $(
            impl TwimExt for $twim {
                fn constrain(self, pins: Pins, frequency: Frequency)
                    -> Twim<Self>
                {
                    Twim::new(self, pins, frequency)
                }
            }
        )*
    }
}

impl_twim_ext!(
    TWIM0,
    TWIM1,
);


/// Interface to a TWIM instance
///
/// This is a very basic interface that comes with the following limitation:
/// The TWIM instances share the same address space with instances of SPIM,
/// SPIS, SPI, TWIS, and TWI. For example, TWIM0 conflicts with SPIM0, SPIS0,
/// etc.; TWIM1 conflicts with SPIM1, SPIS1, etc. You need to make sure that
/// conflicting instances are disabled before using `Twim`. Please refer to the
/// product specification for more information (section 15.2 for nRF52832,
/// section 6.1.2 for nRF52840).
pub struct Twim<T>(T);

impl<T> Twim<T> where T: TwimExt {
    pub fn new(twim: T, pins: Pins, frequency: Frequency) -> Self {
        // The TWIM peripheral requires the pins to be in a mode that is not
        // exposed through the GPIO API, and might it might not make sense to
        // expose it there.
        //
        // Until we've figured out what to do about this, let's just configure
        // the pins through the raw peripheral API. All of the following is
        // safe, as we own the pins now and have exclusive access to their
        // registers.
        for &pin in &[pins.scl.pin, pins.sda.pin] {
            unsafe { &*P0::ptr() }.pin_cnf[pin as usize].write(|w|
                w
                    .dir().input()
                    .input().connect()
                    .pull().pullup()
                    .drive().s0d1()
                    .sense().disabled()
            );
        }

        // Select pins
        twim.psel.scl.write(|w| {
            let w = unsafe { w.pin().bits(pins.scl.pin) };
            w.connect().connected()
        });
        twim.psel.sda.write(|w| {
            let w = unsafe { w.pin().bits(pins.sda.pin) };
            w.connect().connected()
        });

        // Enable TWIM instance
        twim.enable.write(|w|
            w.enable().enabled()
        );

        // Configure frequency
        twim.frequency.write(|w| w.frequency().variant(frequency));


        Twim(twim)
    }

    /// Write to an I2C slave
    ///
    /// The buffer must have a length of at most 255 bytes.
    pub fn write(&mut self,
        address: u8,
        buffer:  &[u8],
    )
        -> Result<(), Error>
    {
        let mut offset = 0;
        while offset < buffer.len() {
            let datalen = min(EASY_DMA_SIZE, buffer.len() - offset);
            let dataptr = offset + (buffer.as_ptr() as usize);
            offset += datalen;
            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // before any DMA action has started
            compiler_fence(SeqCst);

            self.0.address.write(|w| unsafe { w.address().bits(address) });

            // Set up the DMA write
            self.0.txd.ptr.write(|w|
                // We're giving the register a pointer to the stack. Since we're
                // waiting for the I2C transaction to end before this stack pointer
                // becomes invalid, there's nothing wrong here.
                //
                // The PTR field is a full 32 bits wide and accepts the full range
                // of values.
                unsafe { w.ptr().bits(dataptr as u32) }
            );
            self.0.txd.maxcnt.write(|w|
                // We're giving it the length of the buffer, so no danger of
                // accessing invalid memory. We have verified that the length of the
                // buffer fits in an `u8`, so the cast to `u8` is also fine.
                //
                // The MAXCNT field is 8 bits wide and accepts the full range of
                // values.
                unsafe { w.maxcnt().bits(datalen as _) }
            );

            // Start write operation
            self.0.tasks_starttx.write(|w|
                // `1` is a valid value to write to task registers.
                unsafe { w.bits(1) }
            );

            // Wait until write operation is about to end
            while self.0.events_lasttx.read().bits() == 0 {}
            self.0.events_lasttx.write(|w| w); // reset event

            // Stop read operation
            self.0.tasks_stop.write(|w|
                // `1` is a valid value to write to task registers.
                unsafe { w.bits(1) }
            );

            // Wait until write operation has ended
            while self.0.events_stopped.read().bits() == 0 {}
            self.0.events_stopped.write(|w| w); // reset event

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // after all possible DMA actions have completed
            compiler_fence(SeqCst);

            if self.0.txd.amount.read().bits() != datalen as u32 {
                return Err(Error::Transmit);
            }
        }
        Ok(())
    }

    /// Read from an I2C slave
    pub fn read(&mut self,
        address: u8,
        buffer:  &mut [u8],
    )
        -> Result<(), Error>
    {
        let mut offset = 0;
        while offset < buffer.len() {
            let datalen = min(EASY_DMA_SIZE, buffer.len() - offset);
            let dataptr = offset + (buffer.as_ptr() as usize);
            offset += datalen;

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // before any DMA action has started
            compiler_fence(SeqCst);

            self.0.address.write(|w| unsafe { w.address().bits(address) });

            // Set up the DMA read
            self.0.rxd.ptr.write(|w|
                // We're giving the register a pointer to the stack. Since we're
                // waiting for the I2C transaction to end before this stack pointer
                // becomes invalid, there's nothing wrong here.
                //
                // The PTR field is a full 32 bits wide and accepts the full range
                // of values.
                unsafe { w.ptr().bits(dataptr as u32) }
            );
            self.0.rxd.maxcnt.write(|w|
                // We're giving it the length of the buffer, so no danger of
                // accessing invalid memory. We have verified that the length of the
                // buffer fits in an `u8`, so the cast to the type of maxcnt
                // is also fine.
                //
                // Note that that nrf52840 maxcnt is a wider
                // type than a u8, so we use a `_` cast rather than a `u8` cast.
                // The MAXCNT field is thus at least 8 bits wide and accepts the
                // full range of values that fit in a `u8`.
                unsafe { w.maxcnt().bits(datalen as _) }
            );

            // Start read operation
            self.0.tasks_startrx.write(|w|
                // `1` is a valid value to write to task registers.
                unsafe { w.bits(1) }
            );

            // Wait until read operation is about to end
            while self.0.events_lastrx.read().bits() == 0 {}
            self.0.events_lastrx.write(|w| w); // reset event

            // Stop read operation
            self.0.tasks_stop.write(|w|
                // `1` is a valid value to write to task registers.
                unsafe { w.bits(1) }
            );

            // Wait until read operation has ended
            while self.0.events_stopped.read().bits() == 0 {}
            self.0.events_stopped.write(|w| w); // reset event

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // after all possible DMA actions have completed
            compiler_fence(SeqCst);

            if self.0.rxd.amount.read().bits() != datalen as u32 {
                return Err(Error::Receive);
            }
        }
        Ok(())
    }

    /// Write data to an I2C slave, then read data from the slave without
    /// triggering a stop condition between the two
    ///
    /// The buffer must have a length of at most 255 bytes.
    pub fn write_then_read(&mut self,
        address: u8,
        wr_buffer:  &[u8],
        rd_buffer: &mut [u8],
    )
        -> Result<(), Error>
    {
        if wr_buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        if rd_buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        compiler_fence(SeqCst);

        self.0.address.write(|w| unsafe { w.address().bits(address) });

        // Set up the DMA write
        self.0.txd.ptr.write(|w|
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the I2C transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            unsafe { w.ptr().bits(wr_buffer.as_ptr() as u32) }
        );
        self.0.txd.maxcnt.write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to `u8` is also fine.
            //
            // The MAXCNT field is 8 bits wide and accepts the full range of
            // values.
            unsafe { w.maxcnt().bits(wr_buffer.len() as _) }
        );

        // Set up the DMA read
        self.0.rxd.ptr.write(|w|
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the I2C transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            unsafe { w.ptr().bits(rd_buffer.as_mut_ptr() as u32) }
        );
        self.0.rxd.maxcnt.write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to the type of maxcnt
            // is also fine.
            //
            // Note that that nrf52840 maxcnt is a wider
            // type than a u8, so we use a `_` cast rather than a `u8` cast.
            // The MAXCNT field is thus at least 8 bits wide and accepts the
            // full range of values that fit in a `u8`.
            unsafe { w.maxcnt().bits(rd_buffer.len() as _) }
        );

        // Immediately start RX after TX, then stop
        self.0.shorts.modify(|_r, w|
            w.lasttx_startrx().enabled()
             .lastrx_stop().enabled()
        );

        // Start write operation
        self.0.tasks_starttx.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) }
        );

        // Wait until total operation has ended
        while self.0.events_stopped.read().bits() == 0 {}

        self.0.events_lasttx.write(|w| w); // reset event
        self.0.events_lastrx.write(|w| w); // reset event
        self.0.events_stopped.write(|w| w); // reset event
        self.0.shorts.write(|w| w);

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed
        compiler_fence(SeqCst);

        let bad_write = self.0.txd.amount.read().bits() != wr_buffer.len() as u32;
        let bad_read  = self.0.rxd.amount.read().bits() != rd_buffer.len() as u32;

        if bad_write {
            return Err(Error::Transmit);
        }

        if bad_read {
            return Err(Error::Receive);
        }

        Ok(())
    }

    /// Return the raw interface to the underlying TWIM peripheral
    pub fn free(self) -> T {
        self.0
    }
}


/// The pins used by the TWIN peripheral
///
/// Currently, only P0 pins are supported.
pub struct Pins {
    // Serial Clock Line
    pub scl: P0_Pin<Input<Floating>>,

    // Serial Data Line
    pub sda: P0_Pin<Input<Floating>>,
}


#[derive(Debug)]
pub enum Error {
    BufferTooLong,
    Transmit,
    Receive,
}
