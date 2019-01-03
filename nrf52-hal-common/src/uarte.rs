//! HAL interface to the UARTE peripheral
//!
//! See product specification:
//!
//! - nrf52832: Section 35
//! - nrf52840: Section 6.34
use core::ops::Deref;
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};
use core::ptr::NonNull;
use lazy_static::lazy_static;
use core::cmp::min;

use crate::target::{
    uarte0,
    UARTE0,
    interrupt,
    Interrupt,
    NVIC,
};

use crate::prelude::*;
use crate::gpio::{
    p0::P0_Pin,
    Output,
    PushPull,
};

use bbqueue::{BBQueue, Producer, Consumer, typenum::*, GrantR};

// Re-export SVD variants to allow user to directly set values
pub use crate::target::uarte0::{
    baudrate::BAUDRATEW as Baudrate,
    config::PARITYW as Parity,
};

pub trait UarteExt: Deref<Target = uarte0::RegisterBlock> + Sized {
    fn constrain(self, nvic: NVIC, pins: Pins, parity: Parity, baudrate: Baudrate) -> Uarte<Self>;
}

impl UarteExt for UARTE0 {
    fn constrain(self, nvic: NVIC, pins: Pins, parity: Parity, baudrate: Baudrate) -> Uarte<Self> {
        Uarte::new(self, nvic, pins, parity, baudrate)
    }
}

lazy_static! {
    static ref MUH_BUFFAH: BBQueue<U2048> = {
        BBQueue::new()
    };
}

enum DmaState {
    Idle,
    ActiveGrant((GrantR, usize)),
}

static mut MAYBE_CONSUMER: Option<Consumer<'static, U2048>> = None;
static mut DMA_STATUS: DmaState = DmaState::Idle;

/// Interface to a UARTE instance
///
/// This is a very basic interface that comes with the following limitations:
/// - The UARTE instances share the same address space with instances of UART.
///   You need to make sure that conflicting instances
///   are disabled before using `Uarte`. See product specification:
///     - nrf52832: Section 15.2
///     - nrf52840: Section 6.1.2
pub struct Uarte<T>{
    periph: T,
    prod: Producer<'static, U2048>,
    nvic: NVIC,
}

impl<T> Uarte<T> where T: UarteExt {
    pub fn new(uarte: T, mut nvic: NVIC, mut pins: Pins, parity: Parity, baudrate: Baudrate) -> Self {
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

        let (prod, cons) = MUH_BUFFAH.split();

        unsafe {
            MAYBE_CONSUMER = Some(cons)
        };

        nvic.enable(Interrupt::UARTE0_UART0);

        uarte.intenset.write(|w| {
            w.endtx().set_bit()
        });

        Uarte {
            periph: uarte,
            prod,
            nvic,
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


pub struct Pins {
    pub rxd: P0_Pin<Output<PushPull>>,
    pub txd: P0_Pin<Output<PushPull>>,
    pub cts: Option<P0_Pin<Output<PushPull>>>,
    pub rts: Option<P0_Pin<Output<PushPull>>>,
}


#[derive(Debug)]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    Transmit,
    Receive,
}

#[interrupt]
unsafe fn UARTE0_UART0() {
    // First, check if end_tx is pending, and clear it
    let uart = &*UARTE0::ptr();

    let end_tx = uart.events_endtx.read().bits() != 0;

    if end_tx {
        uart.events_endtx.write(|w| w.bits(0));
    }
    let cons: &mut Consumer<_> = MAYBE_CONSUMER.as_mut().unwrap();

    use crate::uarte::DmaState::*;
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

    let rgrant = cons.read();
    let len = rgrant.buf.len();

    if len == 0 {
        // No data pending, no more sending
        return;
    }

    let sz = ::core::cmp::min(255, len);

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
