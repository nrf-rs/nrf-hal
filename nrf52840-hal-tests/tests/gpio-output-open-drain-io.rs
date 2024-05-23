// Required connections:
//
// - P0.28 <-> P0.29

#![deny(warnings)]
#![no_std]
#![no_main]

use defmt_rtt as _;
use nrf52840_hal as _;
use nrf52840_hal::gpio::{Input, OpenDrainIO, Output, Pin, PullUp};
use panic_probe as _;

struct State {
    input_pin: Option<Pin<Input<PullUp>>>,
    output_pin: Pin<Output<OpenDrainIO>>,
}

#[defmt_test::tests]
mod tests {
    use cortex_m::asm;
    use defmt::{assert, unwrap};
    use embedded_hal::digital::{InputPin, OutputPin};
    use nrf52840_hal::{
        gpio::{p0, Level, OpenDrainConfig},
        pac,
    };

    use super::State;

    #[init]
    fn init() -> State {
        let p = unwrap!(pac::Peripherals::take());
        let port0 = p0::Parts::new(p.P0);

        let input_pin = Some(port0.p0_28.into_pullup_input().degrade());
        let output_pin = port0
            .p0_29
            .into_open_drain_input_output(OpenDrainConfig::Standard0Disconnect1, Level::High)
            .degrade();

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
        assert!(state.input_pin.as_mut().unwrap().is_low().unwrap());
    }

    #[test]
    fn set_high_is_open(state: &mut State) {
        state.output_pin.set_high().unwrap();
        // GPIO operations are not instantaneous so a delay is needed
        asm::delay(100);
        assert!(state.input_pin.as_mut().unwrap().is_high().unwrap());

        let mut pulled_down_input_pin = state.input_pin.take().unwrap().into_pulldown_input();
        // GPIO operations are not instantaneous so a delay is needed
        asm::delay(100);
        assert!(pulled_down_input_pin.is_low().unwrap());

        // Restore original input pin state
        state.input_pin = Some(pulled_down_input_pin.into_pullup_input());
    }

    #[test]
    fn open_pullup_reads_high(state: &mut State) {
        state.output_pin.set_high().unwrap();
        // GPIO operations are not instantaneous so a delay is needed
        asm::delay(100);
        assert!(state.output_pin.is_high().unwrap());
    }

    #[test]
    fn open_pulldown_reads_low(state: &mut State) {
        state.output_pin.set_high().unwrap();

        let mut pulled_down_input_pin = state.input_pin.take().unwrap().into_pulldown_input();
        // GPIO operations are not instantaneous so a delay is needed
        asm::delay(100);
        assert!(pulled_down_input_pin.is_low().unwrap());

        // Restore original input pin state
        state.input_pin = Some(pulled_down_input_pin.into_pullup_input());
    }
}
