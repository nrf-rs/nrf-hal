#![no_std]
#![no_main]

use {core::panic::PanicInfo, nrf52840_hal as hal, rtt_target::rprintln};

#[rtic::app(device = crate::hal::pac, peripherals = true)]
mod app {
    use {
        hal::qdec::*,
        nrf52840_hal as hal,
        rtt_target::{rprintln, rtt_init_print},
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        qdec: Qdec,
        value: i16,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let _clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        rtt_init_print!();

        let p0 = hal::gpio::p0::Parts::new(ctx.device.P0);
        let pins = Pins {
            a: p0.p0_31.into_pullup_input().degrade(),
            b: p0.p0_30.into_pullup_input().degrade(),
            led: None,
        };

        let qdec = Qdec::new(ctx.device.QDEC, pins, SamplePeriod::_128us);
        qdec.debounce(true)
            .enable_interrupt(NumSamples::_1smpl)
            .enable();

        (Shared {}, Local { qdec, value: 0 }, init::Monotonics())
    }

    #[task(binds = QDEC, local = [qdec, value])]
    fn on_qdec(ctx: on_qdec::Context) {
        ctx.local.qdec.reset_events();
        *ctx.local.value += ctx.local.qdec.read();
        rprintln!("Value: {}", ctx.local.value);
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    rprintln!("{}", info);
    loop {}
}
