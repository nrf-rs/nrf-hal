//! Watchdog example for nRF52840-DK
//!
//! The watchdog will be configured to time out after five seconds.
//! There will be four handles, one tied to each button. To prevent a
//! reset from occuring, the user must press all four buttons within
//! a five second window. The LEDs represent the "is_pet" state of
//! each watchdog handle.
//!
//! This is basically playing whack-a-mole (pet-a-dog?) with the buttons
//! to prevent the watchdog from restarting the system. If you use the
//! Reset button to reset the device before the watchdog kicks, you'll
//! see the reset reason was not due to the watchdog.

#![no_std]
#![no_main]

use embedded_hal::{
    digital::v2::{InputPin, OutputPin},
    timer::CountDown,
};
use {
    core::{
        panic::PanicInfo,
        sync::atomic::{compiler_fence, Ordering},
    },
    hal::pac::TIMER0,
    hal::{
        gpio::{Input, Level, Output, Pin, PullUp, PushPull},
        timer::Timer,
        wdt::{count, handles::HdlN, Parts, Watchdog, WatchdogHandle},
    },
    nrf52840_hal as hal,
    rtt_target::{rprintln, rtt_init_print},
};

#[rtic::app(device = crate::hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        btn1: Pin<Input<PullUp>>,
        btn2: Pin<Input<PullUp>>,
        btn3: Pin<Input<PullUp>>,
        btn4: Pin<Input<PullUp>>,
        led1: Pin<Output<PushPull>>,
        led2: Pin<Output<PushPull>>,
        led3: Pin<Output<PushPull>>,
        led4: Pin<Output<PushPull>>,
        hdl0: WatchdogHandle<HdlN>,
        hdl1: WatchdogHandle<HdlN>,
        hdl2: WatchdogHandle<HdlN>,
        hdl3: WatchdogHandle<HdlN>,
        timer: Timer<TIMER0>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> init::LateResources {
        let _clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        rtt_init_print!();
        let p0 = hal::gpio::p0::Parts::new(ctx.device.P0);

        let btn1 = p0.p0_11.into_pullup_input().degrade();
        let btn2 = p0.p0_12.into_pullup_input().degrade();
        let btn3 = p0.p0_24.into_pullup_input().degrade();
        let btn4 = p0.p0_25.into_pullup_input().degrade();

        let led1 = p0.p0_13.into_push_pull_output(Level::High).degrade();
        let led2 = p0.p0_14.into_push_pull_output(Level::High).degrade();
        let led3 = p0.p0_15.into_push_pull_output(Level::High).degrade();
        let led4 = p0.p0_16.into_push_pull_output(Level::High).degrade();

        let timer = Timer::new(ctx.device.TIMER0);

        // Create a new watchdog instance
        //
        // In case the watchdog is already running, just spin and let it expire, since
        // we can't configure it anyway. This usually happens when we first program
        // the device and the watchdog was previously active
        let (hdl0, hdl1, hdl2, hdl3) = match Watchdog::try_new(ctx.device.WDT) {
            Ok(mut watchdog) => {
                // Set the watchdog to timeout after 5 seconds (in 32.768kHz ticks)
                watchdog.set_lfosc_ticks(5 * 32768);

                // Activate the watchdog with four handles
                let Parts {
                    watchdog: _watchdog,
                    handles,
                } = watchdog.activate::<count::Four>();

                handles
            }
            Err(wdt) => match Watchdog::try_recover::<count::Four>(wdt) {
                Ok(Parts { mut handles, .. }) => {
                    rprintln!("Oops, watchdog already active, but recovering!");

                    // Pet all the dogs quickly to reset to default timeout
                    handles.0.pet();
                    handles.1.pet();
                    handles.2.pet();
                    handles.3.pet();

                    handles
                }
                Err(_wdt) => {
                    rprintln!("Oops, watchdog already active, resetting!");
                    loop {
                        continue;
                    }
                }
            },
        };

        // Enable the monotonic timer (CYCCNT)
        ctx.core.DCB.enable_trace();
        ctx.core.DWT.enable_cycle_counter();

        rprintln!("Starting!");

        if ctx.device.POWER.resetreas.read().dog().is_detected() {
            ctx.device.POWER.resetreas.modify(|_r, w| {
                // Clear the watchdog reset reason bit
                w.dog().set_bit()
            });
            rprintln!("Restarted by the dog!");
        } else {
            rprintln!("Not restarted by the dog!");
        }

        init::LateResources {
            btn1,
            btn2,
            btn3,
            btn4,
            led1,
            led2,
            led3,
            led4,
            hdl0: hdl0.degrade(),
            hdl1: hdl1.degrade(),
            hdl2: hdl2.degrade(),
            hdl3: hdl3.degrade(),
            timer,
        }
    }

    #[idle(resources = [btn1, btn2, btn3, btn4, led1, led2, led3, led4, hdl0, hdl1, hdl2, hdl3, timer])]
    fn idle(mut ctx: idle::Context) -> ! {
        let buttons = [
            &ctx.resources.btn1,
            &ctx.resources.btn2,
            &ctx.resources.btn3,
            &ctx.resources.btn4,
        ];

        let leds = [
            &mut ctx.resources.led1,
            &mut ctx.resources.led2,
            &mut ctx.resources.led3,
            &mut ctx.resources.led4,
        ];

        let handles = [
            &mut ctx.resources.hdl0,
            &mut ctx.resources.hdl1,
            &mut ctx.resources.hdl2,
            &mut ctx.resources.hdl3,
        ];

        let timer = &mut ctx.resources.timer;

        let mut cumulative_ticks = 0;
        let mut any_pet = false;

        timer.start(1_000_000u32 * 6);

        loop {
            let mut petted = 0;

            // Loop through all handles/leds/buttons
            for i in 0..4 {
                if !handles[i].is_pet() && buttons[i].is_low().unwrap() {
                    rprintln!("Petting {}", i);
                    any_pet = true;
                    handles[i].pet();
                    while buttons[i].is_low().unwrap() {}
                }

                if handles[i].is_pet() {
                    petted += 1;
                    leds[i].set_low().ok();
                } else {
                    leds[i].set_high().ok();
                }
            }

            // We must have pet all handles, reset the timer
            if any_pet && petted == 0 {
                cumulative_ticks = 0;
                any_pet = false;
                timer.start(1_000_000u32 * 6);
            }

            // Check whether to update the counter time
            let rd = timer.read();
            if (cumulative_ticks + 250_000) <= rd {
                cumulative_ticks = rd;
                rprintln!("Time left: {}ms", (5_000_000 - rd) / 1_000);
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
