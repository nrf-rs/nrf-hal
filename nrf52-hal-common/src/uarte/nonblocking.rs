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
    interrupt,
    Interrupt,
    NVIC,
};

use crate::prelude::*;

use bbqueue::{
    Producer,
    Consumer,
    GrantR,
    Error as BBQError
};

use crate::uarte::{
    Error,
    Pins,
    Parity,
    Baudrate,
};

pub trait UarteAsyncExt: Deref<Target = uarte0::RegisterBlock> + Sized {
    fn constrain_async(
        self,
        nvic: &mut NVIC,
        pins: Pins,
        parity: Parity,
        baudrate: Baudrate,
        prod: Producer,
        cons: Consumer,
    ) -> UarteAsync<Self>;
}

impl UarteAsyncExt for UARTE0 {
    fn constrain_async(
        self,
        nvic: &mut NVIC,
        pins: Pins,
        parity: Parity,
        baudrate: Baudrate,
        prod: Producer,
        cons: Consumer,
    ) -> UarteAsync<Self> {
        UarteAsync::new(
            self,
            nvic,
            pins,
            parity,
            baudrate,
            prod,
            cons,
        )
    }
}

enum DmaState {
    Idle,
    ActiveGrant((GrantR, usize)),
}

static mut MAYBE_CONSUMER: Option<Consumer> = None;
static mut DMA_STATUS: DmaState = DmaState::Idle;

/// Interface to a UARTE instance
///
/// This is a very basic interface that comes with the following limitations:
/// - The UARTE instances share the same address space with instances of UART.
///   You need to make sure that conflicting instances
///   are disabled before using `Uarte`. See product specification:
///     - nrf52832: Section 15.2
///     - nrf52840: Section 6.1.2
pub struct UarteAsync<T>{
    periph: T,
    prod: Producer,
}

impl<T> UarteAsync<T> where T: UarteAsyncExt {
    pub fn new(
        uarte: T,
        nvic: &mut NVIC,
        mut pins: Pins,
        parity: Parity,
        baudrate: Baudrate,
        prod: Producer,
        cons: Consumer,
    ) -> Self {
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

        unsafe {
            MAYBE_CONSUMER = Some(cons)
        };

        nvic.enable(Interrupt::UARTE0_UART0);

        uarte.intenset.write(|w| {
            w.endtx().set_bit()
        });

        UarteAsync {
            periph: uarte,
            prod,
        }
    }

    pub fn write_async(
        &mut self,
        tx_buffer: &[u8]
    ) -> Result<(), Error> {
        let w_grant = self.prod.grant(tx_buffer.len())
            .map_err(|_| Error::TxBufferTooLong)?;

        w_grant.buf.copy_from_slice(tx_buffer);

        self.prod.commit(tx_buffer.len(), w_grant);

        NVIC::pend(Interrupt::UARTE0_UART0);

        Ok(())
    }

    /// Return the raw interface to the underlying UARTE peripheral
    pub fn free(self) -> T {
        self.periph
    }
}

#[interrupt]
unsafe fn UARTE0_UART0() {
    // First, check if end_tx is pending, and clear it
    let uart = &*UARTE0::ptr();

    let end_tx = uart.events_endtx.read().bits() != 0;

    if end_tx {
        uart.events_endtx.write(|w| w.bits(0));
    }
    let cons: &mut Consumer = MAYBE_CONSUMER.as_mut().unwrap();

    use crate::uarte::nonblocking::DmaState::*;
    let mut stat = Idle;

    ::core::mem::swap(&mut stat, &mut DMA_STATUS);

    match (end_tx, stat) {
        (false, Idle) => {
            // check for pending data, send if possible
        }
        (true, Idle) => {
            // what?
            panic!("what?");
            // return;
        }
        (false, ActiveGrant((gr, sz))) => {
            // Starting condition but already busy, let it go
            // Place grant back!
            DMA_STATUS = ActiveGrant((gr, sz));
            return;
        }
        (true, ActiveGrant((gr, sz))) => {
            // Complete, check for retrigger
            let sent = uart.txd.amount.read().bits() as usize;
            // assert!(sent < sz);
            cons.release(min(sz, sent), gr);
        }
    }

    let rgrant = match cons.read() {
        Ok(gr) => gr,
        Err(BBQError::InsufficientSize) => {
            // No more data to send
            return;
        }
        Err(_) => panic!("queue error"),
    };

    let sz = min(255, rgrant.buf.len());

    // Conservative compiler fence to prevent optimizations that do not
    // take in to account actions by DMA. The fence has been placed here,
    // before any DMA action has started
    compiler_fence(SeqCst);

    // Set up the DMA write
    uart.txd.ptr.write(|w|
        // We're giving the register a pointer to the stack. Since we're
        // waiting for the UARTE transaction to end before this stack pointer
        // becomes invalid, there's nothing wrong here.
        //
        // The PTR field is a full 32 bits wide and accepts the full range
        // of values.
        w.ptr().bits(rgrant.buf.as_ptr() as u32)
    );
    uart.txd.maxcnt.write(|w|
        // We're giving it the length of the buffer, so no danger of
        // accessing invalid memory. We have verified that the length of the
        // buffer fits in an `u8`, so the cast to `u8` is also fine.
        //
        // The MAXCNT field is 8 bits wide and accepts the full range of
        // values.
        w.maxcnt().bits(sz as _));

    // Start UARTE Transmit transaction
    uart.tasks_starttx.write(|w|
        // `1` is a valid value to write to task registers.
        w.bits(1));

    DMA_STATUS = ActiveGrant((rgrant, sz));
}
