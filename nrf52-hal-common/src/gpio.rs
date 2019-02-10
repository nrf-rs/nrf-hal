// TODO - clean these up
#![allow(unused_imports)]
#![allow(non_camel_case_types)]

use core::marker::PhantomData;

/// Input mode (type state)
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Floating input (type state)
pub struct Floating;
/// Pulled down input (type state)
pub struct PullDown;
/// Pulled up input (type state)
pub struct PullUp;

/// Output mode (type state)
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(
        self,
        // apb2: &mut APB2
    ) -> Self::Parts;
}

/// Push pull output (type state)
pub struct PushPull;
/// Open drain output (type state)
pub struct OpenDrain;

// /// Alternate function
// pub struct Alternate<MODE> {
//     _mode: PhantomData<MODE>,
// }


/// Represents a digital input or output level
pub enum Level {
    Low,
    High,
}


// ===============================================================
// Implement Generic Pins for this port, which allows you to use
// other peripherals without having to be completely rust-generic
// across all of the possible pins
// ===============================================================
/// Generic $PX pin
pub struct Pin<MODE> {
    pub pin: u8,
    #[cfg(feature = "52840")]
    pub port: bool,
    _mode: PhantomData<MODE>,
}

use crate::target::P0;

#[cfg(feature = "52840")]
use crate::target::{ P1 };

use crate::hal::digital::{OutputPin, StatefulOutputPin, InputPin};

impl<MODE> Pin<MODE> {
    /// Convert the pin to be a floating input
    pub fn into_floating_input(self) -> Pin<Input<Floating>> {
        unsafe {
            &(*{
                #[cfg(any(feature = "52810", feature = "52832"))]
                { P0::ptr() }
                #[cfg(feature = "52840")]
                { if !self.port { P0::ptr() } else { P1::ptr() } }
            }).pin_cnf[self.pin as usize]
        }
        .write(|w| {
            w.dir().input()
                .input().connect()
                .pull().disabled()
                .drive().s0s1()
                .sense().disabled()
        });

        Pin {
            _mode: PhantomData,
            #[cfg(feature = "52840")]
            port: self.port,
            pin: self.pin
        }
    }
    pub fn into_pullup_input(self) -> Pin<Input<PullUp>> {
        unsafe {
            &(*{
                #[cfg(any(feature = "52810", feature = "52832"))]
                { P0::ptr() }
                #[cfg(feature = "52840")]
                { if !self.port { P0::ptr() } else { P1::ptr() } }
            }).pin_cnf[self.pin as usize]
        }
        .write(|w| {
            w.dir().input()
                .input().connect()
                .pull().pullup()
                .drive().s0s1()
                .sense().disabled()
        });

        Pin {
            _mode: PhantomData,
            #[cfg(feature = "52840")]
            port: self.port,
            pin: self.pin
        }
    }
    pub fn into_pulldown_input(self) -> Pin<Input<PullDown>> {
        unsafe {
            &(*{
                #[cfg(any(feature = "52810", feature = "52832"))]
                { P0::ptr() }
                #[cfg(feature = "52840")]
                { if !self.port { P0::ptr() } else { P1::ptr() } }
            }).pin_cnf[self.pin as usize]
        }
        .write(|w| {
            w.dir().input()
                .input().connect()
                .pull().pulldown()
                .drive().s0s1()
                .sense().disabled()
        });

        Pin {
            _mode: PhantomData,
            #[cfg(feature = "52840")]
            port: self.port,
            pin: self.pin
        }
    }

    /// Convert the pin to be a push-pull output with normal drive
    pub fn into_push_pull_output(self, initial_output: Level)
        -> Pin<Output<PushPull>>
    {
        let mut pin = Pin {
            _mode: PhantomData,
            #[cfg(feature = "52840")]
            port: self.port,
            pin: self.pin
        };

        match initial_output {
            Level::Low  => pin.set_low(),
            Level::High => pin.set_high(),
        }

        unsafe {
            &(*{
                #[cfg(any(feature = "52810", feature = "52832"))]
                { P0::ptr() }
                #[cfg(feature = "52840")]
                { if !self.port { P0::ptr() } else { P1::ptr() } }
            }).pin_cnf[self.pin as usize]
        }
        .write(|w| {
            w.dir().output()
                .input().connect() // AJM - hack for SPI
                .pull().disabled()
                .drive().s0s1()
                .sense().disabled()
        });

        pin
    }

    /// Convert the pin to be an open-drain output
    ///
    /// This method currently does not support configuring an
    /// internal pull-up or pull-down resistor.
    pub fn into_open_drain_output(self,
        config:         OpenDrainConfig,
        initial_output: Level,
    )
        -> Pin<Output<OpenDrain>>
    {
        let mut pin = Pin {
            _mode: PhantomData,
            #[cfg(feature = "52840")]
            port: self.port,
            pin: self.pin
        };

        match initial_output {
            Level::Low  => pin.set_low(),
            Level::High => pin.set_high(),
        }

        // This is safe, as we restrict our access to the dedicated
        // register for this pin.
        let pin_cnf = unsafe {
            &(*{
                #[cfg(any(feature = "52810", feature = "52832"))]
                { P0::ptr() }
                #[cfg(feature = "52840")]
                { if !self.port { P0::ptr() } else { P1::ptr() } }
            }).pin_cnf[self.pin as usize]
        };
        pin_cnf.write(|w| {
            w
                .dir().output()
                .input().disconnect()
                .pull().disabled()
                .drive().variant(config.variant())
                .sense().disabled()
        });

        pin
    }
}

impl<MODE> InputPin for Pin<Input<MODE>> {
    fn is_high(&self) -> bool {
        !self.is_low()
    }

    fn is_low(&self) -> bool {
        unsafe { (
            (*{
                #[cfg(any(feature = "52810", feature = "52832"))]
                { P0::ptr() }
                #[cfg(feature = "52840")]
                { if !self.port { P0::ptr() } else { P1::ptr() } }
            }).in_.read().bits() & (1 << self.pin)
        ) == 0 }
    }
}

impl<MODE> OutputPin for Pin<Output<MODE>> {
    /// Set the output as high
    fn set_high(&mut self) {
        // NOTE(unsafe) atomic write to a stateless register - TODO(AJM) verify?
        // TODO - I wish I could do something like `.pins$i()`...
        unsafe {
            (*{
                #[cfg(any(feature = "52810", feature = "52832"))]
                { P0::ptr() }
                #[cfg(feature = "52840")]
                { if !self.port { P0::ptr() } else { P1::ptr() } }
            }).outset.write(|w| w.bits(1u32 << self.pin));
        }
    }

    /// Set the output as low
    fn set_low(&mut self) {
        // NOTE(unsafe) atomic write to a stateless register - TODO(AJM) verify?
        // TODO - I wish I could do something like `.pins$i()`...
        unsafe {
            (*{
                #[cfg(any(feature = "52810", feature = "52832"))]
                { P0::ptr() }
                #[cfg(feature = "52840")]
                { if !self.port { P0::ptr() } else { P1::ptr() } }
            }).outclr.write(|w| w.bits(1u32 << self.pin));
        }
    }
}

impl<MODE> StatefulOutputPin for Pin<Output<MODE>> {
    /// Is the output pin set as high?
    fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    /// Is the output pin set as low?
    fn is_set_low(&self) -> bool {
        // NOTE(unsafe) atomic read with no side effects - TODO(AJM) verify?
        // TODO - I wish I could do something like `.pins$i()`...
        unsafe { (
            (*{
                #[cfg(any(feature = "52810", feature = "52832"))]
                { P0::ptr() }
                #[cfg(feature = "52840")]
                { if !self.port { P0::ptr() } else { P1::ptr() } }
            }).out.read().bits() & (1 << self.pin)
        ) == 0 }
    }
}

/// Pin configuration for open-drain mode
pub enum OpenDrainConfig {
    Disconnect0Standard1,
    Disconnect0HighDrive1,
    Standard0Disconnect1,
    HighDrive0Disconnect1,
}

use crate::target::p0::{
    pin_cnf,
    PIN_CNF,
};

impl OpenDrainConfig {
    fn variant(self) -> pin_cnf::DRIVEW {
        use self::OpenDrainConfig::*;

        match self {
            Disconnect0Standard1  => pin_cnf::DRIVEW::D0S1,
            Disconnect0HighDrive1 => pin_cnf::DRIVEW::D0H1,
            Standard0Disconnect1  => pin_cnf::DRIVEW::S0D1,
            HighDrive0Disconnect1 => pin_cnf::DRIVEW::H0D1,
        }
    }
}

macro_rules! gpio {
    (
        $PX:ident, $pxsvd:ident, $px:ident, $port_value:expr, [
            $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty),)+
        ]
    ) => {
        /// GPIO
        pub mod $px {
            use super::{
                Pin,

                // Alternate,
                Floating,
                GpioExt,
                Input,
                Level,
                OpenDrain,
                OpenDrainConfig,
                Output,
                PullDown,
                PullUp,
                PushPull,

                PhantomData,
            };

            use crate::target;
            use crate::target::$PX;
            use crate::hal::digital::{OutputPin, StatefulOutputPin, InputPin};

            

            // ===============================================================
            // This chunk allows you to obtain an nrf52-hal gpio from the
            // upstream nrf52 gpio definitions by defining a trait
            // ===============================================================
            /// GPIO parts
            pub struct Parts {
                $(
                    /// Pin
                    pub $pxi: $PXi<$MODE>,
                )+
            }

            impl GpioExt for $PX {
                type Parts = Parts;

                fn split(self) -> Parts {
                    Parts {
                        $(
                            $pxi: $PXi {
                                _mode: PhantomData,
                            },
                        )+
                    }
                }
            }

            // ===============================================================
            // Implement each of the typed pins usable through the nrf52-hal
            // defined interface
            // ===============================================================
            $(
                pub struct $PXi<MODE> {
                    _mode: PhantomData<MODE>,
                }


                impl<MODE> $PXi<MODE> {
                    /// Convert the pin to be a floating input
                    pub fn into_floating_input(self) -> $PXi<Input<Floating>> {
                        unsafe { &(*$PX::ptr()).pin_cnf[$i] }.write(|w| {
                            w.dir().input()
                             .input().connect()
                             .pull().disabled()
                             .drive().s0s1()
                             .sense().disabled()
                        });

                        $PXi {
                            _mode: PhantomData,
                        }
                    }
                    pub fn into_pulldown_input(self) -> $PXi<Input<PullDown>> {
                        unsafe { &(*$PX::ptr()).pin_cnf[$i] }.write(|w| {
                            w.dir().input()
                             .input().connect()
                             .pull().pulldown()
                             .drive().s0s1()
                             .sense().disabled()
                        });

                        $PXi {
                            _mode: PhantomData,
                        }
                    }
                    pub fn into_pullup_input(self) -> $PXi<Input<PullUp>> {
                        unsafe { &(*$PX::ptr()).pin_cnf[$i] }.write(|w| {
                            w.dir().input()
                             .input().connect()
                             .pull().pullup()
                             .drive().s0s1()
                             .sense().disabled()
                        });

                        $PXi {
                            _mode: PhantomData,
                        }
                    }

                    /// Convert the pin to bepin a push-pull output with normal drive
                    pub fn into_push_pull_output(self, initial_output: Level)
                        -> $PXi<Output<PushPull>>
                    {
                        let mut pin = $PXi {
                            _mode: PhantomData,
                        };

                        match initial_output {
                            Level::Low  => pin.set_low(),
                            Level::High => pin.set_high(),
                        }

                        unsafe { &(*$PX::ptr()).pin_cnf[$i] }.write(|w| {
                            w.dir().output()
                             .input().disconnect()
                             .pull().disabled()
                             .drive().s0s1()
                             .sense().disabled()
                        });

                        pin
                    }

                    /// Convert the pin to be an open-drain output
                    ///
                    /// This method currently does not support configuring an
                    /// internal pull-up or pull-down resistor.
                    pub fn into_open_drain_output(self,
                        config:         OpenDrainConfig,
                        initial_output: Level,
                    )
                        -> $PXi<Output<OpenDrain>>
                    {
                        let mut pin = $PXi {
                            _mode: PhantomData,
                        };

                        match initial_output {
                            Level::Low  => pin.set_low(),
                            Level::High => pin.set_high(),
                        }

                        // This is safe, as we restrict our access to the
                        // dedicated register for this pin.
                        let pin_cnf = unsafe {
                            &(*$PX::ptr()).pin_cnf[$i]
                        };
                        pin_cnf.write(|w| {
                            w
                                .dir().output()
                                .input().disconnect()
                                .pull().disabled()
                                .drive().variant(config.variant())
                                .sense().disabled()
                        });

                        pin
                    }

                    /// Degrade to a generic pin struct, which can be used with peripherals
                    pub fn degrade(self) -> Pin<MODE> {
                        Pin {
                            _mode: PhantomData,
                            #[cfg(feature = "52840")]
                            port: $port_value,
                            pin: $i
                        }
                    }
                }

                impl<MODE> InputPin for $PXi<Input<MODE>> {
                    fn is_high(&self) -> bool {
                        !self.is_low()
                    }

                    fn is_low(&self) -> bool {
                        unsafe { ((*$PX::ptr()).in_.read().bits() & (1 << $i)) == 0 }
                    }
                }

                impl<MODE> OutputPin for $PXi<Output<MODE>> {
                    /// Set the output as high
                    fn set_high(&mut self) {
                        // NOTE(unsafe) atomic write to a stateless register - TODO(AJM) verify?
                        // TODO - I wish I could do something like `.pins$i()`...
                        unsafe { (*$PX::ptr()).outset.write(|w| w.bits(1u32 << $i)); }
                    }

                    /// Set the output as low
                    fn set_low(&mut self) {
                        // NOTE(unsafe) atomic write to a stateless register - TODO(AJM) verify?
                        // TODO - I wish I could do something like `.pins$i()`...
                        unsafe { (*$PX::ptr()).outclr.write(|w| w.bits(1u32 << $i)); }
                    }
                }

                impl<MODE> StatefulOutputPin for $PXi<Output<MODE>> {
                    /// Is the output pin set as high?
                    fn is_set_high(&self) -> bool {
                        !self.is_set_low()
                    }

                    /// Is the output pin set as low?
                    fn is_set_low(&self) -> bool {
                        // NOTE(unsafe) atomic read with no side effects - TODO(AJM) verify?
                        // TODO - I wish I could do something like `.pins$i()`...
                        unsafe { ((*$PX::ptr()).out.read().bits() & (1 << $i)) == 0 }
                    }
                }
            )+
        }
    }
}

// ===========================================================================
// Definition of all the items used by the macros above.
//
// For now, it is a little repetitive, especially as the nrf52 only has one
// 32-bit GPIO port (P0)
// ===========================================================================
gpio!(P0, p0, p0, false, [
    P0_00: (p0_00,  0, Input<Floating>),
    P0_01: (p0_01,  1, Input<Floating>),
    P0_02: (p0_02,  2, Input<Floating>),
    P0_03: (p0_03,  3, Input<Floating>),
    P0_04: (p0_04,  4, Input<Floating>),
    P0_05: (p0_05,  5, Input<Floating>),
    P0_06: (p0_06,  6, Input<Floating>),
    P0_07: (p0_07,  7, Input<Floating>),
    P0_08: (p0_08,  8, Input<Floating>),
    P0_09: (p0_09,  9, Input<Floating>),
    P0_10: (p0_10, 10, Input<Floating>),
    P0_11: (p0_11, 11, Input<Floating>),
    P0_12: (p0_12, 12, Input<Floating>),
    P0_13: (p0_13, 13, Input<Floating>),
    P0_14: (p0_14, 14, Input<Floating>),
    P0_15: (p0_15, 15, Input<Floating>),
    P0_16: (p0_16, 16, Input<Floating>),
    P0_17: (p0_17, 17, Input<Floating>),
    P0_18: (p0_18, 18, Input<Floating>),
    P0_19: (p0_19, 19, Input<Floating>),
    P0_20: (p0_20, 20, Input<Floating>),
    P0_21: (p0_21, 21, Input<Floating>),
    P0_22: (p0_22, 22, Input<Floating>),
    P0_23: (p0_23, 23, Input<Floating>),
    P0_24: (p0_24, 24, Input<Floating>),
    P0_25: (p0_25, 25, Input<Floating>),
    P0_26: (p0_26, 26, Input<Floating>),
    P0_27: (p0_27, 27, Input<Floating>),
    P0_28: (p0_28, 28, Input<Floating>),
    P0_29: (p0_29, 29, Input<Floating>),
    P0_30: (p0_30, 30, Input<Floating>),
    P0_31: (p0_31, 31, Input<Floating>),
]);

// The p1 types are present in the p0 module generated from the
// svd, but we want to export them in a p1 module from this crate.
#[cfg(feature = "52840")]
gpio!(P1, p0, p1, true, [
    P1_00: (p1_00,  0, Input<Floating>),
    P1_01: (p1_01,  1, Input<Floating>),
    P1_02: (p1_02,  2, Input<Floating>),
    P1_03: (p1_03,  3, Input<Floating>),
    P1_04: (p1_04,  4, Input<Floating>),
    P1_05: (p1_05,  5, Input<Floating>),
    P1_06: (p1_06,  6, Input<Floating>),
    P1_07: (p1_07,  7, Input<Floating>),
    P1_08: (p1_08,  8, Input<Floating>),
    P1_09: (p1_09,  9, Input<Floating>),
    P1_10: (p1_10, 10, Input<Floating>),
    P1_11: (p1_11, 11, Input<Floating>),
    P1_12: (p1_12, 12, Input<Floating>),
    P1_13: (p1_13, 13, Input<Floating>),
    P1_14: (p1_14, 14, Input<Floating>),
    P1_15: (p1_15, 15, Input<Floating>),
]);
