#![no_std]
#![no_main]

use {core::panic::PanicInfo, rtt_target::rprintln};

#[rtic::app(device = nrf52840_hal::pac, peripherals = true)]
mod app {
    use embedded_hal::digital::v2::OutputPin;
    use nrf52840_hal::clocks::Clocks;
    use nrf52840_hal::comp::*;
    use nrf52840_hal::gpio::{self, Level, Output, Pin, PushPull};
    use rtt_target::{rprintln, rtt_init_print};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        comp: Comp,
        led1: Pin<Output<PushPull>>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let _clocks = Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        rtt_init_print!();

        let p0 = gpio::p0::Parts::new(ctx.device.P0);
        let led1 = p0.p0_13.into_push_pull_output(Level::High).degrade();
        let in_pin = p0.p0_30.into_floating_input();
        let ref_pin = p0.p0_31.into_floating_input();

        let comp = Comp::new(ctx.device.COMP, &in_pin);
        comp.differential(&ref_pin)
            .hysteresis(true)
            .enable_interrupt(Transition::Cross)
            .enable();

        (Shared {}, Local { comp, led1 }, init::Monotonics())
    }

    #[task(binds = COMP_LPCOMP, local = [comp, led1])]
    fn on_comp(ctx: on_comp::Context) {
        ctx.local.comp.reset_event(Transition::Cross);
        match ctx.local.comp.read() {
            CompResult::Above => {
                rprintln!("Vin > Vref");
                ctx.local.led1.set_low().ok();
            }
            CompResult::Below => {
                rprintln!("Vin < Vref");
                ctx.local.led1.set_high().ok();
            }
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
