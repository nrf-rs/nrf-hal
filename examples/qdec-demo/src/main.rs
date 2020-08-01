#![no_std]
#![no_main]

use {
    core::{
        panic::PanicInfo,
        sync::atomic::{compiler_fence, Ordering},
    },
    hal::qdec::*,
    nrf52840_hal as hal,
    rtt_target::{rprintln, rtt_init_print},
};

#[rtic::app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        qdec: Qdec,
        #[init(0)]
        value: i16,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        let _clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        rtt_init_print!();

        let p0 = hal::gpio::p0::Parts::new(ctx.device.P0);
        let pin_a = p0.p0_31.into_pullup_input().degrade();
        let pin_b = p0.p0_30.into_pullup_input().degrade();

        let qdec = Qdec::new(ctx.device.QDEC, pin_a, pin_b, None, SamplePeriod::_128us);
        qdec.debounce(true)
            .enable_interrupt(NumSamples::_1smpl)
            .enable();

        init::LateResources { qdec }
    }

    #[task(binds = QDEC, resources = [qdec, value])]
    fn on_qdec(ctx: on_qdec::Context) {
        ctx.resources.qdec.reset_events();
        *ctx.resources.value += ctx.resources.qdec.read();
        rprintln!("Value: {}", ctx.resources.value);
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
