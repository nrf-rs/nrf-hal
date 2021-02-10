// Required connections:
//
// - P0.28 <-> P0.29

#![no_std]
#![no_main]

use defmt_rtt as _;
use nrf52840_hal as _;
use panic_probe as _;

use nrf52840_hal::{
    pac::{TIMER0, UARTE0},
    timer::OneShot,
    uarte::Uarte,
    Timer,
};

struct State {
    uarte: Uarte<UARTE0>,
    timer: Timer<TIMER0, OneShot>,
}

#[defmt_test::tests]
mod tests {
    use defmt::unwrap;
    use nrf52840_hal::{
        gpio::{p0, Level},
        pac,
    };
    use nrf52840_hal::{
        uarte::{Baudrate, Parity, Pins, Uarte},
        Timer,
    };

    use super::State;

    #[init]
    fn init() -> State {
        let p = unwrap!(pac::Peripherals::take());
        let port0 = p0::Parts::new(p.P0);

        let timer = Timer::one_shot(p.TIMER0);

        let rxd = port0.p0_28.into_floating_input().degrade();
        let txd = port0.p0_29.into_push_pull_output(Level::High).degrade();

        let pins = Pins {
            rxd,
            txd,
            cts: None,
            rts: None,
        };

        let uarte = Uarte::new(p.UARTE0, pins, Parity::EXCLUDED, Baudrate::BAUD9600);

        State {
            uarte,
            timer,
        }
    }

    // won't work because of how the `read` API work
    /*
    #[test]
    fn loopback(state: &mut State) {
        const BYTE: u8 = 0x42;
        const TIMEOUT: u32 = 1_000_000;

        let mut buffer = [BYTE];

        // NOTE we pass a mutable reference to prevent the buffer from being allocated in Flash
        // (.rodata) as that results in an error
        state.uarte.write(&mut buffer).unwrap();

        // clear this to detect the issue of `read` not writing to the buffer
        buffer[0] = 0;
        state
            .uarte
            .read_timeout(&mut buffer, &mut state.timer, TIMEOUT)
            .unwrap();

        defmt::assert_eq!(buffer[0], BYTE)
    }
    */
}
