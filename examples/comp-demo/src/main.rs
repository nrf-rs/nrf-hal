#![no_std]
#![no_main]

use embedded_hal::digital::v2::OutputPin;
use {
    core::{
        panic::PanicInfo,
        sync::atomic::{compiler_fence, Ordering},
    },
    hal::{
        comp::*,
        gpio::{Level, Output, Pin, PushPull},
    },
    nrf52840_hal as hal,
    rtt_target::{rprintln, rtt_init_print},
};

#[rtic::app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        comp: Comp,
        led1: Pin<Output<PushPull>>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        let _clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        rtt_init_print!();

        let p0 = hal::gpio::p0::Parts::new(ctx.device.P0);
        let led1 = p0.p0_13.into_push_pull_output(Level::High).degrade();
        let in_pin = p0.p0_30.into_floating_input();
        let ref_pin = p0.p0_31.into_floating_input();

        let comp = Comp::new(ctx.device.COMP, &in_pin);
        comp.differential(&ref_pin)
            .hysteresis(true)
            .enable_interrupt(Transition::Cross)
            .enable();

        init::LateResources { comp, led1 }
    }

    #[task(binds = COMP_LPCOMP, resources = [comp, led1])]
    fn on_comp(ctx: on_comp::Context) {
        ctx.resources.comp.reset_event(Transition::Cross);
        match ctx.resources.comp.read() {
            CompResult::Above => {
                rprintln!("Vin > Vref");
                ctx.resources.led1.set_low().ok();
            }
            CompResult::Below => {
                rprintln!("Vin < Vref");
                ctx.resources.led1.set_high().ok();
            }
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
