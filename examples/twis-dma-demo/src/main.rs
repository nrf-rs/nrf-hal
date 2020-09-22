#![no_std]
#![no_main]

// Demo of using non-blocking DMA transactions with the
// TWIS (Two Wire Interface/I2C in peripheral mode) module.

use {
    core::{
        panic::PanicInfo,
        sync::atomic::{compiler_fence, Ordering},
    },
    hal::{gpio::p0::Parts, pac::TWIS0, twis::*},
    nrf52840_hal as hal,
    rtt_target::{rprintln, rtt_init_print},
};

pub enum TwisTransfer {
    Running(Transfer<TWIS0, &'static mut [u8; 8]>),
    Idle((&'static mut [u8; 8], Twis<TWIS0>)),
}

#[rtic::app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        transfer: Option<TwisTransfer>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        static mut BUF: [u8; 8] = [0; 8];
        let _clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        rtt_init_print!();
        rprintln!("Waiting for commands from controller...");

        let p0 = Parts::new(ctx.device.P0);
        let scl = p0.p0_14.into_floating_input().degrade();
        let sda = p0.p0_16.into_floating_input().degrade();

        let twis = Twis::new(ctx.device.TWIS0, Pins { scl, sda }, 0x1A);
        twis.enable_interrupt(TwiEvent::Write)
            .enable_interrupt(TwiEvent::Read)
            .enable_interrupt(TwiEvent::Stopped)
            .enable();

        init::LateResources {
            transfer: Some(TwisTransfer::Idle((BUF, twis))),
        }
    }

    #[task(binds = SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0, resources = [transfer])]
    fn on_twis(ctx: on_twis::Context) {
        let (buf, twis) = match ctx.resources.transfer.take().unwrap() {
            TwisTransfer::Running(t) => t.wait(),
            TwisTransfer::Idle(t) => t,
        };
        if twis.is_event_triggered(TwiEvent::Read) {
            twis.reset_event(TwiEvent::Read);
            rprintln!("READ command received");
            *ctx.resources.transfer = Some(TwisTransfer::Running(twis.tx(buf).unwrap()));
        } else if twis.is_event_triggered(TwiEvent::Write) {
            twis.reset_event(TwiEvent::Write);
            rprintln!("WRITE command received");
            *ctx.resources.transfer = Some(TwisTransfer::Running(twis.rx(buf).unwrap()));
        } else {
            twis.reset_event(TwiEvent::Stopped);
            rprintln!("{:?}", buf);
            *ctx.resources.transfer = Some(TwisTransfer::Idle((buf, twis)));
        }
    }
};

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    rprintln!("{}", info);
    loop {
        compiler_fence(Ordering::SeqCst);
    }
}
