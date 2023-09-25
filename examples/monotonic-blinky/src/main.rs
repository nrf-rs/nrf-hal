#![no_main]
#![no_std]

use panic_halt as _;
use nrf52840_hal as hal;
use hal::pac;
#[rtic::app(device = pac, dispatchers = [UARTE1])]
mod app {
    use super::*;
    use cortex_m::asm;
    use pac::TIMER0;
    use hal::{
        gpio::{p0::Parts, Level, Output, Pin, PushPull},
        prelude::*, monotonic::MonotonicTimer,
    };
    use rtt_target::{rprintln, rtt_init_print};

    #[monotonic(binds = TIMER0, default = true)]
    type MyMono = MonotonicTimer<TIMER0,62_500>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Pin<Output<PushPull>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();
        rprintln!("init");

        let mono = MyMono::new(cx.device.TIMER0);
        let p0 = Parts::new(cx.device.P0);
        let led = p0.p0_13.into_push_pull_output(Level::High).degrade();

        blink::spawn().ok();
        (Shared {}, Local { led }, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            rprintln!("idle");
            // Put core to sleep until next interrupt
            asm::wfi();
        }
    }

    #[task(local = [led])]
    fn blink(ctx: blink::Context) {
        rprintln!("Blink!");
        let led = ctx.local.led;
        // Note this unwrap is safe since is_set_low is allways Ok
        if led.is_set_low().unwrap() {
            led.set_high().ok();
        } else {
            led.set_low().ok();
        }
        // spawn after current time + 1 second
        blink::spawn_after(fugit::ExtU32::millis(1000)).ok();
    }
}
