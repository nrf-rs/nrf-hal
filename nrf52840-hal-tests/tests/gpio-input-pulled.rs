// Required connections:
//
// - P0.28 <-> P0.29

#![deny(warnings)]
#![no_std]
#![no_main]

use defmt_rtt as _;
use nrf52840_hal as _;
use nrf52840_hal::gpio::{Floating, Input, Pin};
use panic_probe as _;

struct State {
    input_pin: Pin<Input<Floating>>,
    puller_pin: Option<Pin<Input<Floating>>>,
}

#[defmt_test::tests]
mod tests {
    use cortex_m::asm;
    use defmt::{assert, unwrap};
    use embedded_hal::digital::InputPin;
    use nrf52840_hal::{gpio::p0, pac};

    use super::State;

    #[init]
    fn init() -> State {
        let p = unwrap!(pac::Peripherals::take());
        let port0 = p0::Parts::new(p.P0);

        let input_pin = port0.p0_28.into_floating_input().degrade();
        let puller_pin = Some(port0.p0_29.into_floating_input().degrade());

        State {
            input_pin,
            puller_pin,
        }
    }

    #[test]
    fn pulldown_is_low(state: &mut State) {
        let puller_pin = unwrap!(state.puller_pin.take());

        let mut pulldown_pin = puller_pin.into_pulldown_input();
        // GPIO re-configuration is not instantaneous so a delay is needed
        asm::delay(100);
        assert!(pulldown_pin.is_low().unwrap());

        state.puller_pin = Some(pulldown_pin.into_floating_input());
    }

    #[test]
    fn pulldown_drives_low(state: &mut State) {
        let puller_pin = unwrap!(state.puller_pin.take());

        let pulldown_pin = puller_pin.into_pulldown_input();
        assert!(state.input_pin.is_low().unwrap());

        state.puller_pin = Some(pulldown_pin.into_floating_input());
    }

    #[test]
    fn pullup_is_high(state: &mut State) {
        let puller_pin = unwrap!(state.puller_pin.take());

        let mut pullup_pin = puller_pin.into_pullup_input();
        // GPIO re-configuration is not instantaneous so a delay is needed
        asm::delay(100);
        assert!(pullup_pin.is_high().unwrap());

        state.puller_pin = Some(pullup_pin.into_floating_input());
    }

    #[test]
    fn pullup_drives_high(state: &mut State) {
        let puller_pin = unwrap!(state.puller_pin.take());

        let pullup_pin = puller_pin.into_pullup_input();
        // GPIO re-configuration is not instantaneous so a delay is needed
        asm::delay(100);
        assert!(state.input_pin.is_high().unwrap());

        state.puller_pin = Some(pullup_pin.into_floating_input());
    }
}
