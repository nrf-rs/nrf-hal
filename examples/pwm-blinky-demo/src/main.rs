#![no_main]
#![no_std]

use hal::{
    gpio,
    prelude::*,
    pwm::{Channel, Pwm},
    timer::Timer,
};
#[cfg(feature = "52832")]
use nrf52832_hal as hal;
#[cfg(feature = "52840")]
use nrf52840_hal as hal;
#[cfg(feature = "9160")]
use nrf9160_hal as hal;
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

    let (pwm, mut timer) = init_device(p);

    pwm.set_period(500u32.hz());

    rprintln!("PWM Blinky demo starting");

    let wait_time = 1_000_000u32 / pwm.max_duty() as u32;
    loop {
        for duty in 0..pwm.max_duty() {
            pwm.set_duty_on_common(duty);
            timer.delay(wait_time);
        }
    }
}

#[cfg(feature = "9160")]
fn init_device(p: hal::pac::Peripherals) -> (Pwm<hal::pac::PWM0_NS>, Timer<hal::pac::TIMER0_NS>) {
    let p0 = gpio::p0::Parts::new(p.P0_NS);

    let pwm = Pwm::new(p.PWM0_NS);
    pwm.set_output_pin(
        Channel::C0,
        p0.p0_02.into_push_pull_output(gpio::Level::High).degrade(),
    );

    let timer = Timer::new(p.TIMER0_NS);

    (pwm, timer)
}

#[cfg(feature = "52840")]
fn init_device(p: hal::pac::Peripherals) -> (Pwm<hal::pac::PWM0>, Timer<hal::pac::TIMER0>) {
    let p0 = gpio::p0::Parts::new(p.P0);

    let pwm = Pwm::new(p.PWM0);
    pwm.set_output_pin(
        Channel::C0,
        p0.p0_13.into_push_pull_output(gpio::Level::High).degrade(),
    );

    let timer = Timer::new(p.TIMER0);

    (pwm, timer)
}

#[cfg(feature = "52832")]
fn init_device(p: hal::pac::Peripherals) -> (Pwm<hal::pac::PWM0>, Timer<hal::pac::TIMER0>) {
    let p0 = gpio::p0::Parts::new(p.P0);

    let pwm = Pwm::new(p.PWM0);
    pwm.set_output_pin(
        Channel::C0,
        p0.p0_30.into_push_pull_output(gpio::Level::High).degrade(),
    );

    let timer = Timer::new(p.TIMER0);

    (pwm, timer)
}
