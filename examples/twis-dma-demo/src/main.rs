#![no_std]
#![no_main]

// Demo of using non-blocking DMA transactions with the
// TWIS (Two Wire Interface/I2C in peripheral mode) module.

use {core::panic::PanicInfo, nrf52840_hal as hal, rtt_target::rprintln};

#[rtic::app(device = crate::hal::pac, peripherals = true)]
mod app {
    use {
        hal::{gpio::p0::Parts, gpiote::Gpiote, pac::TWIS0, twis::*},
        nrf52840_hal as hal,
        rtt_target::{rprintln, rtt_init_print},
    };

    type DmaBuffer = &'static mut [u8; 8];

    pub enum TwisTransfer {
        Running(Transfer<TWIS0, DmaBuffer>),
        Idle((DmaBuffer, Twis<TWIS0>)),
    }

    #[shared]
    struct Shared {
        #[lock_free]
        transfer: Option<TwisTransfer>,
    }

    #[local]
    struct Local {
        gpiote: Gpiote,
    }

    #[init(local = [
        BUF: [u8; 8] = [0; 8],
    ])]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let BUF = ctx.local.BUF;

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

        let btn = p0.p0_29.into_pullup_input().degrade();
        let gpiote = Gpiote::new(ctx.device.GPIOTE);
        gpiote.port().input_pin(&btn).low();
        gpiote.port().enable_interrupt();

        (
            Shared {
                transfer: Some(TwisTransfer::Idle((BUF, twis))),
            },
            Local { gpiote },
            init::Monotonics(),
        )
    }

    #[task(binds = GPIOTE, local = [gpiote], shared = [transfer])]
    fn on_gpiote(ctx: on_gpiote::Context) {
        ctx.local.gpiote.reset_events();
        rprintln!("Reset buffer");
        let transfer = ctx.shared.transfer;
        let (buf, twis) = match transfer.take().unwrap() {
            TwisTransfer::Running(t) => t.wait(),
            TwisTransfer::Idle(t) => t,
        };
        buf.copy_from_slice(&[0; 8][..]);
        rprintln!("{:?}", buf);
        transfer.replace(TwisTransfer::Idle((buf, twis)));
    }

    #[task(binds = SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0, shared = [transfer])]
    fn on_twis(ctx: on_twis::Context) {
        let transfer = ctx.shared.transfer;
        let (buf, twis) = match transfer.take().unwrap() {
            TwisTransfer::Running(t) => t.wait(),
            TwisTransfer::Idle(t) => t,
        };
        if twis.is_event_triggered(TwiEvent::Read) {
            twis.reset_event(TwiEvent::Read);
            rprintln!("READ command received");
            let tx = twis.tx(buf).unwrap();
            transfer.replace(TwisTransfer::Running(tx));
        } else if twis.is_event_triggered(TwiEvent::Write) {
            twis.reset_event(TwiEvent::Write);
            rprintln!("WRITE command received");
            let rx = twis.rx(buf).unwrap();
            transfer.replace(TwisTransfer::Running(rx));
        } else {
            twis.reset_event(TwiEvent::Stopped);
            rprintln!("{:?}", buf);
            transfer.replace(TwisTransfer::Idle((buf, twis)));
        }
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    rprintln!("{}", info);
    loop {}
}
