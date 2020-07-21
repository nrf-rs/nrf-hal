use crate::{
    pac::{Interrupt, NVIC},
    ppi::{Ppi, ConfigurablePpi},
    timer::Instance as TimerInstance,
    uarte:: {
        Baudrate, Parity, Pins, uarte_start_read, uarte_setup, uarte_start_write, uarte_cancel_read, Instance as UarteInstance,
    },
    target_constants::EASY_DMA_SIZE,
};
use bbqueue::{ArrayLength, Consumer, GrantR, GrantW, Producer};
use core::sync::atomic::{compiler_fence, AtomicBool, Ordering::SeqCst};

pub struct UarteTimer<Timer>
where
    Timer: TimerInstance,
{
    pub(crate) timer: Timer,
    pub(crate) timeout_flag: &'static AtomicBool,
    pub(crate) interrupt: Interrupt,
}

impl<Timer> UarteTimer<Timer>
where
    Timer: TimerInstance,
{
    pub fn init(&mut self, microsecs: u32) {
        self.timer.disable_interrupt();
        self.timer.timer_cancel();
        self.timer.set_periodic();
        self.timer.set_shorts_periodic();
        self.timer.enable_interrupt();

        self.timer.timer_start(microsecs);
    }

    pub fn interrupt(&self) {
        // pend uarte interrupt
        self.timer.timer_reset_event();
        self.timeout_flag.store(true, SeqCst);
        NVIC::pend(self.interrupt);
    }
}

pub struct UarteIrq<OutgoingLen, IncomingLen, Channel, Uarte>
where
    OutgoingLen: ArrayLength<u8>,
    IncomingLen: ArrayLength<u8>,
    Channel: Ppi + ConfigurablePpi,
    Uarte: UarteInstance
{
    pub(crate) outgoing_cons: Consumer<'static, OutgoingLen>,
    pub(crate) incoming_prod: Producer<'static, IncomingLen>,
    pub(crate) timeout_flag: &'static AtomicBool,
    pub(crate) rx_grant: Option<GrantW<'static, IncomingLen>>,
    pub(crate) tx_grant: Option<GrantR<'static, OutgoingLen>>,
    pub(crate) uarte: Uarte,
    pub(crate) block_size: usize,
    pub(crate) ppi_ch: Channel,
}

impl<OutgoingLen, IncomingLen, Channel, Uarte> UarteIrq<OutgoingLen, IncomingLen, Channel, Uarte>
where
    OutgoingLen: ArrayLength<u8>,
    IncomingLen: ArrayLength<u8>,
    Channel: Ppi + ConfigurablePpi,
    Uarte: UarteInstance
{
    pub fn init(&mut self, pins: Pins, parity: Parity, baudrate: Baudrate) {
        uarte_setup(&self.uarte, pins, parity, baudrate);


        // Clear all interrupts
        self.uarte.intenclr.write(|w| unsafe { w.bits(0xFFFFFFFF) });

        // Enable relevant interrupts
        self.uarte.intenset.write(|w| {
            w.endrx().set_bit();
            w.endtx().set_bit();
            w.error().set_bit();
            w
        });

        self.ppi_ch.enable();

        if let Ok(mut gr) = self.incoming_prod.grant_exact(self.block_size) {
            uarte_start_read(&self.uarte, &mut gr).unwrap();
            self.rx_grant = Some(gr);
        }
    }

    pub fn interrupt(&mut self) {
        let endrx = self.uarte.events_endrx.read().bits() != 0;
        let endtx = self.uarte.events_endtx.read().bits() != 0;
        let rxdrdy = self.uarte.events_rxdrdy.read().bits() != 0;
        let error = self.uarte.events_error.read().bits() != 0;
        let txstopped = self.uarte.events_txstopped.read().bits() != 0;

        let timeout = self.timeout_flag.swap(false, SeqCst);
        let errsrc = self.uarte.errorsrc.read().bits();

        // RX section
        if endrx || timeout || self.rx_grant.is_none() {
            // We only flush the connection if:
            //
            // * We didn't get a "natural" end of reception (full buffer), AND
            // * The timer expired, AND
            // * We have received one or more bytes to the receive buffer
            if !endrx && timeout && rxdrdy {
                uarte_cancel_read(&self.uarte);
            }

            compiler_fence(SeqCst);

            // Get the bytes received. If the rxdrdy flag wasn't set, then we haven't
            // actually received any bytes, and we can't trust the `amount` field
            // (it may have a stale value from the last reception)
            let amt = if rxdrdy {
                self.uarte.rxd.amount.read().bits() as usize
            } else {
                0
            };

            // If we received data, cycle the grant and get a new one
            if amt != 0 || self.rx_grant.is_none() {
                let gr = self.rx_grant.take();

                // If the buffer was full last time, we may not actually have a grant right now
                if let Some(gr) = gr {
                    gr.commit(amt);
                }

                // Attempt to get the next grant. If we don't get one now, no worries,
                // we'll try again on the next timeout
                if let Ok(mut gr) = self.incoming_prod.grant_exact(self.block_size) {
                    uarte_start_read(&self.uarte, &mut gr).unwrap();
                    self.rx_grant = Some(gr);
                }
            }
        }

        // TX Section
        if endtx || self.tx_grant.is_none() {
            if endtx {
                if let Some(gr) = self.tx_grant.take() {
                    let len = gr.len();
                    gr.release(len.min(EASY_DMA_SIZE));
                }
            }

            if let Ok(gr) = self.outgoing_cons.read() {
                let len = gr.len();
                uarte_start_write(&self.uarte, &gr[..len.min(EASY_DMA_SIZE)]).unwrap();
                self.tx_grant = Some(gr);
            }
        }


        // Clear events we processed
        if endrx {
            self.uarte.events_endrx.write(|w| w);
        }
        if endtx {
            self.uarte.events_endtx.write(|w| w);
        }
        if error {
            self.uarte.events_error.write(|w| w);
        }
        if rxdrdy {
            self.uarte.events_rxdrdy.write(|w| w);
        }
        if txstopped {
            self.uarte.events_txstopped.write(|w| w);
        }

        // Clear any errors
        if errsrc != 0 {
            self.uarte.errorsrc.write(|w| unsafe { w.bits(errsrc) });
        }
    }
}
