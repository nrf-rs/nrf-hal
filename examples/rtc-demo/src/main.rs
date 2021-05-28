#![no_main]
#![no_std]

use nrf52840_hal as hal;

use core::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};
use cortex_m::interrupt::Mutex;
use hal::pac::interrupt;
use hal::rtc::{Rtc, RtcCompareReg, RtcInterrupt};
use rtt_target::{rprintln, rtt_init_print};

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

// We need to share the RTC between the main execution thread and an interrupt, hence the mutex.
// They'll never be any contention though as interrupts cannot fire while there's a critical
// section. Also note that the Mutex here is from cortex_m and is designed to work
// only with single core processors.
static RTC: Mutex<RefCell<Option<Rtc<hal::pac::RTC0>>>> = Mutex::new(RefCell::new(None));

// Keep a flag to indicate that our timer has expired.
static TIMER_EXPIRED: AtomicBool = AtomicBool::new(false);

#[interrupt]
fn RTC0() {
    cortex_m::interrupt::free(|cs| {
        let rtc = RTC.borrow(cs).borrow();
        if let Some(rtc) = rtc.as_ref() {
            rtc.reset_event(RtcInterrupt::Compare0);
            rtc.clear_counter();
        }
    });

    TIMER_EXPIRED.store(true, Ordering::Relaxed);
}

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();

    let mut cp = hal::pac::CorePeripherals::take().unwrap();
    let p = hal::pac::Peripherals::take().unwrap();

    // Enable the low-power/low-frequency clock which is required by the RTC.
    let clocks = hal::clocks::Clocks::new(p.CLOCK);
    let clocks = clocks.start_lfclk();

    // Run RTC for 1 second (1hz == LFCLK_FREQ)
    let mut rtc = Rtc::new(p.RTC0, 0).unwrap();
    rtc.set_compare(RtcCompareReg::Compare0, hal::clocks::LFCLK_FREQ)
        .unwrap();
    rtc.enable_event(RtcInterrupt::Compare0);
    rtc.enable_interrupt(RtcInterrupt::Compare0, Some(&mut cp.NVIC));

    rprintln!("Starting RTC");
    rtc.enable_counter();

    // Permit the interrupt to gain access to the RTC for the purpsoes of resetting etc
    cortex_m::interrupt::free(|cs| {
        RTC.borrow(cs).replace(Some(rtc));
    });

    rprintln!("Waiting for compare match");

    while TIMER_EXPIRED.compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
        != Ok(true)
    {
        // Go to sleep until we get an event (typically our RTC interrupt)
        cortex_m::asm::wfe();
    }

    rprintln!("Compare match, stopping RTC");

    if let Some(rtc) = cortex_m::interrupt::free(|cs| RTC.borrow(cs).replace(None)) {
        rtc.disable_counter();

        rprintln!("Counter stopped at {} ticks", rtc.get_counter());

        rtc.release();
    }

    // Stop LfClk when RTC is not used anymore.
    clocks.stop_lfclk();

    loop {
        cortex_m::asm::nop();
    }
}
