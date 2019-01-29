//! HAL interface to the UARTE peripheral
//!
//! See product specification:
//!
//! - nrf52832: Section 35
//! - nrf52840: Section 6.34
use core::ops::Deref;
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};
use core::cmp::min;

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
};

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
        pins.rxd.set_high();
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

    pub fn write(&mut self,
        tx_buffer  : &[u8],
    )
        -> Result<(), Error>
    {
        let mut offset = 0;
        while offset < tx_buffer.len() {
            let datalen = min(EASY_DMA_SIZE, tx_buffer.len() - offset);
            let dataptr = offset + (tx_buffer.as_ptr() as usize);
            offset += EASY_DMA_SIZE;
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
                unsafe { w.ptr().bits(dataptr as u32) }
            );
            self.0.txd.maxcnt.write(|w|
                // We're giving it the length of the buffer, so no danger of
                // accessing invalid memory. We have verified that the length of the
                // buffer fits in an `u8`, so the cast to `u8` is also fine.
                //
                // The MAXCNT field is 8 bits wide and accepts the full range of
                // values.
                unsafe { w.maxcnt().bits(datalen as _) });

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

            if self.0.txd.amount.read().bits() != datalen as u32 {
                return Err(Error::Transmit);
            }
        }
        Ok(())
    }

    /// Return the raw interface to the underlying UARTE peripheral
    pub fn free(self) -> T {
        self.0
    }
}


pub struct Pins {
    pub rxd: P0_Pin<Output<PushPull>>,
    pub txd: P0_Pin<Output<PushPull>>,
    pub cts: Option<P0_Pin<Output<PushPull>>>,
    pub rts: Option<P0_Pin<Output<PushPull>>>,
}


#[derive(Debug)]
pub enum Error {

    Transmit,
    Receive,
}
