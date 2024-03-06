// Required connections:
//
// - P0.03 <-> GND
// - P0.04 <-> VDD

#![deny(warnings)]
#![no_std]
#![no_main]

use defmt_rtt as _;
use nrf52840_hal as _;
use panic_probe as _;

use nrf52840_hal::gpio::{Floating, Input, Pin};

struct State {
    input_ground: Pin<Input<Floating>>,
    input_vdd: Pin<Input<Floating>>,
}

#[defmt_test::tests]
mod tests {
    use defmt::{assert, unwrap};
    use embedded_hal::digital::InputPin;
    use nrf52840_hal::{gpio::p0, pac};

    use super::State;

    #[init]
    fn init() -> State {
        let p = unwrap!(pac::Peripherals::take());
        let port0 = p0::Parts::new(p.P0);

        let input_ground = port0.p0_03.into_floating_input().degrade();
        let input_vdd = port0.p0_04.into_floating_input().degrade();

        State {
            input_ground,
            input_vdd,
        }
    }

    #[test]
    fn ground_is_low(state: &mut State) {
        assert!(state.input_ground.is_low().unwrap());
    }

    #[test]
    fn vdd_is_high(state: &mut State) {
        assert!(state.input_vdd.is_high().unwrap());
    }
}
