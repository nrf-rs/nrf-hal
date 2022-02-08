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
    core::panic::PanicInfo,
    hal::{
        gpio::Level,
        timer::Timer,
        wdt::{count, Parts, Watchdog},
    },
    nrf52840_hal as hal,
    rtt_target::{rprintln, rtt_init_print},
};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::pac::Peripherals::take().unwrap();
    let mut core = cortex_m::Peripherals::take().unwrap();
    let _clocks = hal::clocks::Clocks::new(p.CLOCK).enable_ext_hfosc();
    rtt_init_print!();
    let p0 = hal::gpio::p0::Parts::new(p.P0);

    let btn1 = p0.p0_11.into_pullup_input().degrade();
    let btn2 = p0.p0_12.into_pullup_input().degrade();
    let btn3 = p0.p0_24.into_pullup_input().degrade();
    let btn4 = p0.p0_25.into_pullup_input().degrade();

    let mut led1 = p0.p0_13.into_push_pull_output(Level::High).degrade();
    let mut led2 = p0.p0_14.into_push_pull_output(Level::High).degrade();
    let mut led3 = p0.p0_15.into_push_pull_output(Level::High).degrade();
    let mut led4 = p0.p0_16.into_push_pull_output(Level::High).degrade();

    let mut timer = Timer::new(p.TIMER0);

    // Create a new watchdog instance
    //
    // In case the watchdog is already running, just spin and let it expire, since
    // we can't configure it anyway. This usually happens when we first program
    // the device and the watchdog was previously active
    let (hdl0, hdl1, hdl2, hdl3) = match Watchdog::try_new(p.WDT) {
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
                loop {}
            }
        },
    };

    // Enable the monotonic timer (CYCCNT)
    core.DCB.enable_trace();
    core.DWT.enable_cycle_counter();

    rprintln!("Starting!");

    if p.POWER.resetreas.read().dog().is_detected() {
        p.POWER.resetreas.modify(|_r, w| {
            // Clear the watchdog reset reason bit
            w.dog().set_bit()
        });
        rprintln!("Restarted by the dog!");
    } else {
        rprintln!("Not restarted by the dog!");
    }

    let buttons = [&btn1, &btn2, &btn3, &btn4];

    let leds = [&mut led1, &mut led2, &mut led3, &mut led4];

    let handles = [
        &mut hdl0.degrade(),
        &mut hdl1.degrade(),
        &mut hdl2.degrade(),
        &mut hdl3.degrade(),
    ];

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

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    rprintln!("{}", info);
    loop {}
}
