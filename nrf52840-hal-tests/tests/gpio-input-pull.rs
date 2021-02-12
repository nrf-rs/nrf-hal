// Required connections:
//
// - P0.28 <-> P0.29

#![deny(warnings)]
#![no_std]
#![no_main]

use defmt_rtt as _;
use nrf52840_hal as _;
use panic_probe as _;

use nrf52840_hal::gpio::{Floating, Input, Pin};

struct State {
    input_pin: Pin<Input<Floating>>,
    pull_pin: Option<Pin<Input<Floating>>>,
}

#[defmt_test::tests]
mod tests {
    use defmt::{assert, unwrap};
    use nrf52840_hal::{gpio::p0, pac, prelude::*};

    use super::State;

    #[init]
    fn init() -> State {
        let p = unwrap!(pac::Peripherals::take());
        let port0 = p0::Parts::new(p.P0);

        let input_pin = port0.p0_28.into_floating_input().degrade();
        let pull_pin = Some(port0.p0_29.into_floating_input().degrade());

        State {
            input_pin,
            pull_pin,
        }
    }

    #[test]
    fn pulldown_is_low(state: &mut State) {
        let pull_pin = unwrap!(state.pull_pin.take());

        let pulldown_pin = pull_pin.into_pulldown_input();
        assert!(state.input_pin.is_low().unwrap());

        state.pull_pin = Some(pulldown_pin.into_floating_input());
    }

    #[test]
    fn pullup_is_high(state: &mut State) {
        let pull_pin = unwrap!(state.pull_pin.take());

        let pullup_pin = pull_pin.into_pullup_input();
        assert!(state.input_pin.is_high().unwrap());

        state.pull_pin = Some(pullup_pin.into_floating_input());
    }
}
