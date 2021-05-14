#![no_main]
#![no_std]

use hal::{gpio, prelude::*, pwm, pwm::Pwm, timer, timer::Timer};
use nb::block;
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
    loop {
        pwm.set_duty_on_common(pwm.get_max_duty());
        delay(&mut timer, 250_000); // 250ms
        pwm.set_duty_on_common(0);
        delay(&mut timer, 1_000_000); // 1s
    }
}

#[cfg(feature = "9160")]
fn init_device(p: hal::pac::Peripherals) -> (Pwm<hal::pac::PWM0_NS>, Timer<hal::pac::TIMER0_NS>) {
    let p0 = gpio::p0::Parts::new(p.P0_NS);

    let pwm = Pwm::new(p.PWM0_NS);
    pwm.set_output_pin(
        pwm::Channel::C0,
        &p0.p0_02.into_push_pull_output(gpio::Level::High).degrade(),
    );

    let timer = Timer::new(p.TIMER0_NS);

    (pwm, timer)
}

#[cfg(feature = "52840")]
fn init_device(p: hal::pac::Peripherals) -> (Pwm<hal::pac::PWM0>, Timer<hal::pac::TIMER0>) {
    let p0 = gpio::p0::Parts::new(p.P0);

    let pwm = Pwm::new(p.PWM0);
    pwm.set_output_pin(
        pwm::Channel::C0,
        &p0.p0_13.into_push_pull_output(gpio::Level::High).degrade(),
    );

    let timer = Timer::new(p.TIMER0);

    (pwm, timer)
}

fn delay<T>(timer: &mut Timer<T>, cycles: u32)
where
    T: timer::Instance,
{
    timer.start(cycles);
    let _ = block!(timer.wait());
}
