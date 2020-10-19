#![no_main]
#![no_std]

use nrf52840_hal as hal;

use hal::rtc::{Rtc, RtcCompareReg, RtcInterrupt};
use rtt_target::{rprintln, rtt_init_print};

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();

    let p = hal::pac::Peripherals::take().unwrap();

    // Enable LfClk which is required by the RTC.
    let clocks = hal::clocks::Clocks::new(p.CLOCK);
    let clocks = clocks.start_lfclk();

    // Run RTC for 1 second
    let mut rtc = Rtc::new(p.RTC0);
    rtc.set_compare(RtcCompareReg::Compare0, 32_768).unwrap();
    rtc.enable_event(RtcInterrupt::Compare0);

    rprintln!("Starting RTC");
    let rtc = rtc.enable_counter();

    rprintln!("Waiting for compare match");
    while !rtc.is_event_triggered(RtcInterrupt::Compare0) {}
    rtc.reset_event(RtcInterrupt::Compare0);

    rprintln!("Compare match, stopping RTC");
    let rtc = rtc.disable_counter();

    rprintln!("Counter stopped at {} ticks", rtc.get_counter());

    // Stop LfClk when RTC is not used anymore.
    rtc.release();
    clocks.stop_lfclk();

    loop {
        cortex_m::asm::nop();
    }
}
