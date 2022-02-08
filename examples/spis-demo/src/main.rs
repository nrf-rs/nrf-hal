#![no_std]
#![no_main]

// Demo for the SPIS module, transmitting the current buffer contents while receiving new data.
// Press button to zero the buffer.

use {core::panic::PanicInfo, nrf52840_hal as hal, rtt_target::rprintln};

#[rtic::app(device = crate::hal::pac, peripherals = true)]
mod app {
    use {
        hal::{gpiote::Gpiote, pac::SPIS0, spis::*},
        nrf52840_hal as hal,
        rtt_target::{rprintln, rtt_init_print},
    };

    #[shared]
    struct Shared {
        #[lock_free]
        transfer: Option<Transfer<SPIS0, &'static mut [u8; 8]>>,
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
        rprintln!("Send me [u8; 8] over SPI");
        rprintln!("Press button to reset buffer");

        let p0 = hal::gpio::p0::Parts::new(ctx.device.P0);
        let cs_pin = p0.p0_25.into_floating_input().degrade();
        let sck_pin = p0.p0_24.into_floating_input().degrade();
        let copi_pin = p0.p0_16.into_floating_input().degrade();
        let cipo_pin = p0.p0_14.into_floating_input().degrade();

        let spis = Spis::new(
            ctx.device.SPIS0,
            Pins {
                sck: sck_pin,
                cs: cs_pin,
                copi: Some(copi_pin),
                cipo: Some(cipo_pin),
            },
        );
        spis.enable_interrupt(SpisEvent::End);

        let btn = p0.p0_29.into_pullup_input().degrade();
        let gpiote = Gpiote::new(ctx.device.GPIOTE);
        gpiote.port().input_pin(&btn).low();
        gpiote.port().enable_interrupt();

        (
            Shared {
                transfer: spis.transfer(BUF).ok(),
            },
            Local { gpiote },
            init::Monotonics(),
        )
    }

    #[task(binds = GPIOTE, local = [gpiote], shared = [transfer])]
    fn on_gpiote(ctx: on_gpiote::Context) {
        ctx.local.gpiote.reset_events();
        rprintln!("Reset buffer");
        let (buf, spis) = ctx.shared.transfer.take().unwrap().wait();
        buf.copy_from_slice(&[0; 8][..]);
        rprintln!("{:?}", buf);
        *ctx.shared.transfer = spis.transfer(buf).ok();
    }

    #[task(binds = SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0, shared = [transfer])]
    fn on_spis(ctx: on_spis::Context) {
        let (buf, spis) = ctx.shared.transfer.take().unwrap().wait();
        spis.reset_event(SpisEvent::End);
        rprintln!("Received: {:?}", buf);
        *ctx.shared.transfer = spis.transfer(buf).ok();
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    rprintln!("{}", info);
    loop {}
}
