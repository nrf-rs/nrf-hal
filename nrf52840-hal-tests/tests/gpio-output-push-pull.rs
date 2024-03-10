// Required connections:
//
// - P0.28 <-> P0.29

#![deny(warnings)]
#![no_std]
#![no_main]

use defmt_rtt as _;
use nrf52840_hal as _;
use nrf52840_hal::gpio::{Floating, Input, Output, Pin, PushPull};
use panic_probe as _;

struct State {
    input_pin: Pin<Input<Floating>>,
    output_pin: Pin<Output<PushPull>>,
}

#[defmt_test::tests]
mod tests {
    use cortex_m::asm;
    use defmt::{assert, unwrap};
    use embedded_hal::digital::{InputPin, OutputPin};
    use nrf52840_hal::{
        gpio::{p0, Level},
        pac,
    };

    use super::State;

    #[init]
    fn init() -> State {
        let p = unwrap!(pac::Peripherals::take());
        let port0 = p0::Parts::new(p.P0);

        let input_pin = port0.p0_28.into_floating_input().degrade();
        let output_pin = port0.p0_29.into_push_pull_output(Level::High).degrade();

        State {
            input_pin,
            output_pin,
        }
    }

    #[test]
    fn set_low_is_low(state: &mut State) {
        state.output_pin.set_low().unwrap();
        // GPIO operations are not instantaneous so a delay is needed
        asm::delay(100);
        assert!(state.input_pin.is_low().unwrap());
    }

    #[test]
    fn set_high_is_high(state: &mut State) {
        state.output_pin.set_high().unwrap();
        // GPIO operations are not instantaneous so a delay is needed
        asm::delay(100);
        assert!(state.input_pin.is_high().unwrap());
    }
}
