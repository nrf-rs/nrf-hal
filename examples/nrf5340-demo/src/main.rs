#![no_std]
#![no_main]

extern crate panic_semihosting;

#[cfg(feature = "app")]
use nrf5340_app_hal as hal;
#[cfg(feature = "net")]
use nrf5340_net_hal as hal;

use cortex_m::asm;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use cortex_m_semihosting::hprintln;

use hal::delay::Delay;
use hal::gpio::{p0, Level};
use hal::prelude::*;

#[entry]
fn main() -> ! {
    let p = hal::target::Peripherals::take().unwrap();
    let core = cortex_m::Peripherals::take().unwrap();

    #[cfg(feature = "app")]
    {
        use hal::reset::ResetController;
        use hal::spu::{SecAttr, Spu};
        use hal::target::oscillators_ns::xosc32ki::intcap::INTCAP_A;

        // Weird workaround from Zephyr
        // https://github.com/zephyrproject-rtos/zephyr/blob/4e135d76a3c222f208a8e0a3dce93dde55671a4e/soc/arm/nordic_nrf/nrf53/soc.c#L57-L60
        p.OSCILLATORS_S
            .xosc32ki
            .intcap
            .write(|w| w.intcap().variant(INTCAP_A::C6PF));

        let mut delay = Delay::new(core.SYST);

        let mut spu = Spu::new(p.SPU_S);
        spu.set_network_domain_security(SecAttr::Secure);
        spu.set_gpio_security(29, SecAttr::NonSecure);

        let parts = p0::Parts::new(p.P0_S);
        parts.p0_29.into_network();
        let mut led = parts.p0_31.into_push_pull_output(Level::Low);

        let mut reset = ResetController::new(p.RESET_S);
        reset.set_network_power(true);

        loop {
            delay.delay_ms(500_u32);
            led.set_high().ok();
            delay.delay_ms(500_u32);
            led.set_low().ok();

            hprintln!("{:?}", spu.event_status()).ok();
        }
    }

    #[cfg(feature = "net")]
    {
        let mut led = p0::Parts::new(p.P0_NS)
            .p0_29
            .into_push_pull_output(Level::Low);
        let mut delay = Delay::new(core.SYST);

        loop {
            delay.delay_ms(500_u32);
            led.set_high().ok();
            delay.delay_ms(500_u32);
            led.set_low().ok();
        }
    }
}

#[exception]
fn HardFault(_eh: &ExceptionFrame) -> ! {
    //hprintln!("{:?}", eh).ok();
    loop {
        asm::nop();
    }
}
