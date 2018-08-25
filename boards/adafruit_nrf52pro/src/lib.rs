#![no_std]
pub extern crate nrf52_hal as hal;
use hal::gpio::{p0, Floating, Input};
pub use hal::nrf52;

/// Maps the pins to the names printed on the device
pub struct Pins {
    pub a0: p0::P0_2<Input<Floating>>,
    pub a1: p0::P0_3<Input<Floating>>,
    pub a2: p0::P0_4<Input<Floating>>,
    pub a3: p0::P0_5<Input<Floating>>,
    pub a4: p0::P0_28<Input<Floating>>,
    pub a5: p0::P0_29<Input<Floating>>,
    pub sck: p0::P0_12<Input<Floating>>,
    pub mosi: p0::P0_13<Input<Floating>>,
    pub miso: p0::P0_14<Input<Floating>>,
    pub txd: p0::P0_8<Input<Floating>>,
    pub rxd: p0::P0_6<Input<Floating>>,
    pub dfu: p0::P0_20<Input<Floating>>,
    pub frst: p0::P0_22<Input<Floating>>,
    pub d16: p0::P0_16<Input<Floating>>,
    pub d15: p0::P0_15<Input<Floating>>,
    pub d7: p0::P0_7<Input<Floating>>,
    pub d11: p0::P0_11<Input<Floating>>,
    pub a7: p0::P0_31<Input<Floating>>,
    pub a6: p0::P0_30<Input<Floating>>,
    pub d27: p0::P0_27<Input<Floating>>,
    pub scl: p0::P0_26<Input<Floating>>,
    pub sda: p0::P0_25<Input<Floating>>,
    pub led1: p0::P0_17<Input<Floating>>,
    pub led2: p0::P0_19<Input<Floating>>,
}

impl Pins {
    pub fn new(pins: p0::Parts) -> Self {
        Self {
            a0: pins.p0_2,
            a1: pins.p0_3,
            a2: pins.p0_4,
            a3: pins.p0_5,
            a4: pins.p0_28,
            a5: pins.p0_29,
            sck: pins.p0_12,
            mosi: pins.p0_13,
            miso: pins.p0_14,
            txd: pins.p0_8,
            rxd: pins.p0_6,
            dfu: pins.p0_20,
            frst: pins.p0_22,
            d16: pins.p0_16,
            d15: pins.p0_15,
            d7: pins.p0_7,
            d11: pins.p0_11,
            a7: pins.p0_31,
            a6: pins.p0_30,
            d27: pins.p0_27,
            scl: pins.p0_26,
            sda: pins.p0_25,
            led1: pins.p0_17,
            led2: pins.p0_19,
        }
    }
}
