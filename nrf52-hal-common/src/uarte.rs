//! HAL interface to the UARTE peripheral
//!
//! See product specification:
//!
//! - nrf52832: Section 35
//! - nrf52840: Section 6.34
use core::ops::Deref;
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};

use crate::target::{
    uarte0,
    UARTE0,
};

use crate::target_constants::EASY_DMA_SIZE;
use crate::prelude::*;
use crate::gpio::{
    p0::P0_Pin,
    Output,
    PushPull,
    Input,
    Floating,
};
use crate::timer::Timer;

// Re-export SVD variants to allow user to directly set values
pub use crate::target::uarte0::{
    baudrate::BAUDRATEW as Baudrate,
    config::PARITYW as Parity,
};

pub trait UarteExt: Deref<Target = uarte0::RegisterBlock> + Sized {
    fn constrain(self, pins: Pins, parity: Parity, baudrate: Baudrate) -> Uarte<Self>;
}

impl UarteExt for UARTE0 {
    fn constrain(self, pins: Pins, parity: Parity, baudrate: Baudrate) -> Uarte<Self> {
        Uarte::new(self, pins, parity, baudrate)
    }
}

/// Interface to a UARTE instance
///
/// This is a very basic interface that comes with the following limitations:
/// - The UARTE instances share the same address space with instances of UART.
///   You need to make sure that conflicting instances
///   are disabled before using `Uarte`. See product specification:
///     - nrf52832: Section 15.2
///     - nrf52840: Section 6.1.2
pub struct Uarte<T>(T);

impl<T> Uarte<T> where T: UarteExt {
    pub fn new(uarte: T, mut pins: Pins, parity: Parity, baudrate: Baudrate) -> Self {
        // Select pins
        uarte.psel.rxd.write(|w| {
            let w = unsafe { w.pin().bits(pins.rxd.pin) };
            w.connect().connected()
        });
        pins.txd.set_high();
        uarte.psel.txd.write(|w| {
            let w = unsafe { w.pin().bits(pins.txd.pin) };
            w.connect().connected()
        });

        // Optional pins
        uarte.psel.cts.write(|w| {
            if let Some(ref pin) = pins.cts {
                let w = unsafe { w.pin().bits(pin.pin) };
                w.connect().connected()
            } else {
                w.connect().disconnected()
            }
        });

        uarte.psel.rts.write(|w| {
            if let Some(ref pin) = pins.rts {
                let w = unsafe { w.pin().bits(pin.pin) };
                w.connect().connected()
            } else {
                w.connect().disconnected()
            }
        });

        // Enable UARTE instance
        uarte.enable.write(|w|
            w.enable().enabled()
        );

        // Configure
        let hardware_flow_control = pins.rts.is_some() && pins.cts.is_some();
        uarte.config.write(|w|
            w.hwfc().bit(hardware_flow_control)
             .parity().variant(parity)
        );

        // Configure frequency
        uarte.baudrate.write(|w|
            w.baudrate().variant(baudrate)
        );

        Uarte(uarte)
    }

    /// Write via UARTE
    ///
    /// This method uses transmits all bytes in `tx_buffer`
    ///
    /// The buffer must have a length of at most 255 bytes
    pub fn write(&mut self,
        tx_buffer  : &[u8],
    )
        -> Result<(), Error>
    {
        if tx_buffer.len() > EASY_DMA_SIZE {
            return Err(Error::TxBufferTooLong);
        }

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        compiler_fence(SeqCst);

        // Set up the DMA write
        self.0.txd.ptr.write(|w|
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the UARTE transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            unsafe { w.ptr().bits(tx_buffer.as_ptr() as u32) }
        );
        self.0.txd.maxcnt.write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to `u8` is also fine.
            //
            // The MAXCNT field is 8 bits wide and accepts the full range of
            // values.
            unsafe { w.maxcnt().bits(tx_buffer.len() as _) });

        // Start UARTE Transmit transaction
        self.0.tasks_starttx.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) });

        // Wait for transmission to end
        while self.0.events_endtx.read().bits() == 0 {}

        // Reset the event, otherwise it will always read `1` from now on.
        self.0.events_endtx.write(|w| w);

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed
        compiler_fence(SeqCst);

        if self.0.txd.amount.read().bits() != tx_buffer.len() as u32 {
            return Err(Error::Transmit);
        }

        Ok(())
    }

    /// Read via UARTE
    ///
    /// This method fills all bytes in `rx_buffer`, and blocks
    /// until the buffer is full.
    ///
    /// The buffer must have a length of at most 255 bytes
    pub fn read(&mut self,
        rx_buffer  : &mut [u8],
    )
        -> Result<(), Error>
    {
        self.start_read(rx_buffer)?;

        // Wait for transmission to end
        while self.0.events_endrx.read().bits() == 0 {}

        self.finalize_read();

        if self.0.rxd.amount.read().bits() != rx_buffer.len() as u32 {
            return Err(Error::Receive);
        }

        Ok(())
    }

    /// Read via UARTE
    ///
    /// This method fills all bytes in `rx_buffer`, and blocks
    /// until the buffer is full or the timeout expires, whichever
    /// comes first.
    ///
    /// If the timeout occurs, an `Error::Timeout(n)` will be returned,
    /// where `n` is the number of bytes read successfully.
    ///
    /// This method assumes the interrupt for the given timer is NOT enabled,
    /// and in cases where a timeout does NOT occur, the timer will be left running
    /// until completion.
    ///
    /// The buffer must have a length of at most 255 bytes
    pub fn read_timeout<I>(
        &mut self,
        rx_buffer: &mut [u8],
        timer: &mut Timer<I>,
        cycles: u32
    ) -> Result<(), Error> where I: TimerExt
    {
        // Start the read
        self.start_read(rx_buffer)?;

        // Start the timeout timer
        timer.start(cycles);

        // Wait for transmission to end
        let mut event_complete = false;
        let mut timeout_occured = false;

        loop {
            event_complete |= self.0.events_endrx.read().bits() != 0;
            timeout_occured |= timer.wait().is_ok();
            if event_complete || timeout_occured {
                break;
            }
        }

        // Cleanup, even in the error case
        self.finalize_read();

        let bytes_read = self.0.rxd.amount.read().bits() as usize;

        if timeout_occured {
            return Err(Error::Timeout(bytes_read));
        }

        if bytes_read != rx_buffer.len() as usize {
            return Err(Error::Receive);
        }

        Ok(())
    }

    /// Start a UARTE read transaction by setting the control
    /// values and triggering a read task
    fn start_read(&mut self, rx_buffer: &mut [u8]) -> Result<(), Error> {
        // This is overly restrictive. See (similar SPIM issue):
        // https://github.com/nrf-rs/nrf52/issues/17
        if rx_buffer.len() > u8::max_value() as usize {
            return Err(Error::TxBufferTooLong);
        }

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        compiler_fence(SeqCst);

        // Set up the DMA read
        self.0.rxd.ptr.write(|w|
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the UARTE transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            unsafe { w.ptr().bits(rx_buffer.as_ptr() as u32) }
        );
        self.0.rxd.maxcnt.write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to `u8` is also fine.
            //
            // The MAXCNT field is at least 8 bits wide and accepts the full
            // range of values.
            unsafe { w.maxcnt().bits(rx_buffer.len() as _) });

        // Start UARTE Receive transaction
        self.0.tasks_startrx.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) });

        Ok(())
    }

    /// Finalize a UARTE read transaction by clearing the event
    fn finalize_read(&mut self) {
        // Reset the event, otherwise it will always read `1` from now on.
        self.0.events_endrx.write(|w| w);

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed
        compiler_fence(SeqCst);
    }

    /// Return the raw interface to the underlying UARTE peripheral
    pub fn free(self) -> T {
        self.0
    }
}


pub struct Pins {
    pub rxd: P0_Pin<Input<Floating>>,
    pub txd: P0_Pin<Output<PushPull>>,
    pub cts: Option<P0_Pin<Input<Floating>>>,
    pub rts: Option<P0_Pin<Output<PushPull>>>,
}


#[derive(Debug)]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    Transmit,
    Receive,
    Timeout(usize),
}
