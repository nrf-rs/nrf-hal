//! Defines a minimal blinky example.
//!
//! This example is ment to showcase how to work with the [`MonotonicTimer`] abstraction
#![no_main]
#![no_std]

use hal::pac;
use nrf52840_hal as hal;
use panic_halt as _;
#[rtic::app(device = pac, dispatchers = [UARTE1])]
mod app {
    use super::*;
    use cortex_m::asm;
    use hal::{
        gpio::{p0::Parts, Level, Output, Pin, PushPull},
        monotonic::MonotonicRtc,
        prelude::*,
    };
    use pac::RTC0;
    use rtt_target::{rprintln, rtt_init_print};

    #[monotonic(binds = RTC0, default = true)]
    type MyMono = MonotonicRtc<RTC0, 32_768>;

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

        let p0 = Parts::new(cx.device.P0);
        let led = p0.p0_13.into_push_pull_output(Level::High).degrade();

        let clocks = hal::clocks::Clocks::new(cx.device.CLOCK);
        let clocks = clocks.start_lfclk();
        // Will throw error if freq is invalid
        let mono = MyMono::new(cx.device.RTC0, &clocks).unwrap();

        blink::spawn().ok();
        (Shared {}, Local { led }, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            rprintln!("idle");
            // Put core to sleep until next interrupt
            asm::wfe();
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
