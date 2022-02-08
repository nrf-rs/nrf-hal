#![no_std]
#![no_main]

use {core::panic::PanicInfo, nrf52840_hal as hal, rtt_target::rprintln};

#[rtic::app(device = crate::hal::pac, peripherals = true)]
mod app {
    use embedded_hal::digital::v2::OutputPin;
    use {
        hal::{
            gpio::{Level, Output, Pin, PushPull},
            gpiote::Gpiote,
            lpcomp::*,
            pac::POWER,
        },
        nrf52840_hal as hal,
        rtt_target::{rprintln, rtt_init_print},
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        gpiote: Gpiote,
        led1: Pin<Output<PushPull>>,
        lpcomp: LpComp,
        power: POWER,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let _clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        rtt_init_print!();

        let p0 = hal::gpio::p0::Parts::new(ctx.device.P0);
        let btn1 = p0.p0_11.into_pullup_input().degrade();
        let mut led1 = p0.p0_13.into_push_pull_output(Level::High).degrade();
        let in_pin = p0.p0_04.into_floating_input();
        let ref_pin = p0.p0_03.into_floating_input();

        let lpcomp = LpComp::new(ctx.device.LPCOMP, &in_pin);
        lpcomp
            .vref(VRef::ARef) // Set Vref to external analog reference
            .aref_pin(&ref_pin) // External analog reference pin
            .hysteresis(true)
            .analog_detect(Transition::Up) // Power up the device on upward transition
            .enable_interrupt(Transition::Cross) // Trigger `COMP_LPCOMP` interrupt on any transition
            .enable();

        // Read initial comparator state and set led on/off
        match lpcomp.read() {
            CompResult::Above => led1.set_low().ok(),
            CompResult::Below => led1.set_high().ok(),
        };

        let gpiote = Gpiote::new(ctx.device.GPIOTE);
        gpiote
            .channel0()
            .input_pin(&btn1)
            .hi_to_lo()
            .enable_interrupt();

        rprintln!("Power ON");

        // Check if the device was powered up by the comparator
        if ctx.device.POWER.resetreas.read().lpcomp().is_detected() {
            // Clear the lpcomp reset reason bit
            ctx.device
                .POWER
                .resetreas
                .modify(|_r, w| w.lpcomp().set_bit());
            rprintln!("Powered up by the comparator!");
        }

        rprintln!("Press button 1 to shut down");

        (
            Shared {},
            Local {
                gpiote,
                led1,
                lpcomp,
                power: ctx.device.POWER,
            },
            init::Monotonics(),
        )
    }

    #[task(binds = GPIOTE, local = [gpiote, power])]
    fn on_gpiote(ctx: on_gpiote::Context) {
        ctx.local.gpiote.reset_events();
        rprintln!("Power OFF");
        ctx.local.power.systemoff.write(|w| w.systemoff().enter());
    }

    #[task(binds = COMP_LPCOMP, local = [lpcomp, led1])]
    fn on_comp(ctx: on_comp::Context) {
        ctx.local.lpcomp.reset_events();
        match ctx.local.lpcomp.read() {
            CompResult::Above => ctx.local.led1.set_low().ok(),
            CompResult::Below => ctx.local.led1.set_high().ok(),
        };
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    rprintln!("{}", info);
    loop {}
}
