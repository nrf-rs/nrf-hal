// TODO - clean these up
#![allow(unused_imports)]
#![allow(non_camel_case_types)]

use core::marker::PhantomData;

/// Input mode (type state)
#[derive(Debug)]
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Floating input (type state)
#[derive(Debug)]
pub struct Floating;
/// Pulled down input (type state)
#[derive(Debug)]
pub struct PullDown;
/// Pulled up input (type state)
#[derive(Debug)]
pub struct PullUp;

/// Output mode (type state)
#[derive(Debug)]
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Push pull output (type state)
#[derive(Debug)]
pub struct PushPull;
/// Open drain output (type state)
#[derive(Debug)]
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
#[derive(Debug)]
pub struct Pin<MODE> {
    pub pin: u8,
    #[cfg(feature = "52840")]
    pub port: bool,
    _mode: PhantomData<MODE>,
}

#[cfg(feature = "9160")]
use crate::target::P0_NS as P0;

#[cfg(not(feature = "9160"))]
use crate::target::P0;

#[cfg(feature = "52840")]
use crate::target::P1;

use crate::hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};
use void::Void;

impl<MODE> Pin<MODE> {
    /// Convert the pin to be a floating input
    pub fn into_floating_input(self) -> Pin<Input<Floating>> {
        unsafe {
            &(*{
                #[cfg(not(feature = "52840"))]
                {
                    P0::ptr()
                }
                #[cfg(feature = "52840")]
                {
                    if !self.port {
                        P0::ptr()
                    } else {
                        P1::ptr()
                    }
                }
            })
            .pin_cnf[self.pin as usize]
        }
        .write(|w| {
            w.dir().input();
            w.input().connect();
            w.pull().disabled();
            w.drive().s0s1();
            w.sense().disabled();
            w
        });

        Pin {
            _mode: PhantomData,
            #[cfg(feature = "52840")]
            port: self.port,
            pin: self.pin,
        }
    }
    pub fn into_pullup_input(self) -> Pin<Input<PullUp>> {
        unsafe {
            &(*{
                #[cfg(not(feature = "52840"))]
                {
                    P0::ptr()
                }
                #[cfg(feature = "52840")]
                {
                    if !self.port {
                        P0::ptr()
                    } else {
                        P1::ptr()
                    }
                }
            })
            .pin_cnf[self.pin as usize]
        }
        .write(|w| {
            w.dir().input();
            w.input().connect();
            w.pull().pullup();
            w.drive().s0s1();
            w.sense().disabled();
            w
        });

        Pin {
            _mode: PhantomData,
            #[cfg(feature = "52840")]
            port: self.port,
            pin: self.pin,
        }
    }
    pub fn into_pulldown_input(self) -> Pin<Input<PullDown>> {
        unsafe {
            &(*{
                #[cfg(not(feature = "52840"))]
                {
                    P0::ptr()
                }
                #[cfg(feature = "52840")]
                {
                    if !self.port {
                        P0::ptr()
                    } else {
                        P1::ptr()
                    }
                }
            })
            .pin_cnf[self.pin as usize]
        }
        .write(|w| {
            w.dir().input();
            w.input().connect();
            w.pull().pulldown();
            w.drive().s0s1();
            w.sense().disabled();
            w
        });

        Pin {
            _mode: PhantomData,
            #[cfg(feature = "52840")]
            port: self.port,
            pin: self.pin,
        }
    }

    /// Convert the pin to be a push-pull output with normal drive
    pub fn into_push_pull_output(self, initial_output: Level) -> Pin<Output<PushPull>> {
        let mut pin = Pin {
            _mode: PhantomData,
            #[cfg(feature = "52840")]
            port: self.port,
            pin: self.pin,
        };

        match initial_output {
            Level::Low => pin.set_low().unwrap(),
            Level::High => pin.set_high().unwrap(),
        }

        unsafe {
            &(*{
                #[cfg(not(feature = "52840"))]
                {
                    P0::ptr()
                }
                #[cfg(feature = "52840")]
                {
                    if !self.port {
                        P0::ptr()
                    } else {
                        P1::ptr()
                    }
                }
            })
            .pin_cnf[self.pin as usize]
        }
        .write(|w| {
            w.dir().output();
            w.input().connect(); // AJM - hack for SPI
            w.pull().disabled();
            w.drive().s0s1();
            w.sense().disabled();
            w
        });

        pin
    }

    /// Convert the pin to be an open-drain output
    ///
    /// This method currently does not support configuring an
    /// internal pull-up or pull-down resistor.
    pub fn into_open_drain_output(
        self,
        config: OpenDrainConfig,
        initial_output: Level,
    ) -> Pin<Output<OpenDrain>> {
        let mut pin = Pin {
            _mode: PhantomData,
            #[cfg(feature = "52840")]
            port: self.port,
            pin: self.pin,
        };

        match initial_output {
            Level::Low => pin.set_low().unwrap(),
            Level::High => pin.set_high().unwrap(),
        }

        // This is safe, as we restrict our access to the dedicated
        // register for this pin.
        let pin_cnf = unsafe {
            &(*{
                #[cfg(not(feature = "52840"))]
                {
                    P0::ptr()
                }
                #[cfg(feature = "52840")]
                {
                    if !self.port {
                        P0::ptr()
                    } else {
                        P1::ptr()
                    }
                }
            })
            .pin_cnf[self.pin as usize]
        };
        pin_cnf.write(|w| {
            w.dir().output();
            w.input().disconnect();
            w.pull().disabled();
            w.drive().variant(config.variant());
            w.sense().disabled();
            w
        });

        pin
    }
}

impl<MODE> InputPin for Pin<Input<MODE>> {
    type Error = Void;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe {
            ((*{
                #[cfg(not(feature = "52840"))]
                {
                    P0::ptr()
                }
                #[cfg(feature = "52840")]
                {
                    if !self.port {
                        P0::ptr()
                    } else {
                        P1::ptr()
                    }
                }
            })
            .in_
            .read()
            .bits()
                & (1 << self.pin))
                == 0
        })
    }
}

impl<MODE> OutputPin for Pin<Output<MODE>> {
    type Error = Void;

    /// Set the output as high
    fn set_high(&mut self) -> Result<(), Self::Error> {
        // NOTE(unsafe) atomic write to a stateless register - TODO(AJM) verify?
        // TODO - I wish I could do something like `.pins$i()`...
        unsafe {
            (*{
                #[cfg(not(feature = "52840"))]
                {
                    P0::ptr()
                }
                #[cfg(feature = "52840")]
                {
                    if !self.port {
                        P0::ptr()
                    } else {
                        P1::ptr()
                    }
                }
            })
            .outset
            .write(|w| w.bits(1u32 << self.pin));
        }
        Ok(())
    }

    /// Set the output as low
    fn set_low(&mut self) -> Result<(), Self::Error> {
        // NOTE(unsafe) atomic write to a stateless register - TODO(AJM) verify?
        // TODO - I wish I could do something like `.pins$i()`...
        unsafe {
            (*{
                #[cfg(not(feature = "52840"))]
                {
                    P0::ptr()
                }
                #[cfg(feature = "52840")]
                {
                    if !self.port {
                        P0::ptr()
                    } else {
                        P1::ptr()
                    }
                }
            })
            .outclr
            .write(|w| w.bits(1u32 << self.pin));
        }
        Ok(())
    }
}

impl<MODE> StatefulOutputPin for Pin<Output<MODE>> {
    /// Is the output pin set as high?
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        self.is_set_low().map(|v| !v)
    }

    /// Is the output pin set as low?
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        // NOTE(unsafe) atomic read with no side effects - TODO(AJM) verify?
        // TODO - I wish I could do something like `.pins$i()`...
        Ok(unsafe {
            ((*{
                #[cfg(not(feature = "52840"))]
                {
                    P0::ptr()
                }
                #[cfg(feature = "52840")]
                {
                    if !self.port {
                        P0::ptr()
                    } else {
                        P1::ptr()
                    }
                }
            })
            .out
            .read()
            .bits()
                & (1 << self.pin))
                == 0
        })
    }
}

/// Pin configuration for open-drain mode
pub enum OpenDrainConfig {
    Disconnect0Standard1,
    Disconnect0HighDrive1,
    Standard0Disconnect1,
    HighDrive0Disconnect1,
}

#[cfg(feature = "9160")]
use crate::target::p0_ns::{pin_cnf, PIN_CNF};

#[cfg(not(feature = "9160"))]
use crate::target::p0::{pin_cnf, PIN_CNF};

impl OpenDrainConfig {
    fn variant(self) -> pin_cnf::DRIVE_A {
        use self::OpenDrainConfig::*;

        match self {
            Disconnect0Standard1 => pin_cnf::DRIVE_A::D0S1,
            Disconnect0HighDrive1 => pin_cnf::DRIVE_A::D0H1,
            Standard0Disconnect1 => pin_cnf::DRIVE_A::S0D1,
            HighDrive0Disconnect1 => pin_cnf::DRIVE_A::H0D1,
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
                Input,
                Level,
                OpenDrain,
                OpenDrainConfig,
                Output,
                PullDown,
                PullUp,
                PushPull,

                PhantomData,
                $PX
            };

            use crate::target;
            use crate::hal::digital::v2::{OutputPin, StatefulOutputPin, InputPin};
            use void::Void;



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

            impl Parts {
                pub fn new(_gpio: $PX) -> Self {
                    Self {
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
                            w.dir().input();
                            w.input().connect();
                            w.pull().disabled();
                            w.drive().s0s1();
                            w.sense().disabled();
                            w
                        });

                        $PXi {
                            _mode: PhantomData,
                        }
                    }
                    pub fn into_pulldown_input(self) -> $PXi<Input<PullDown>> {
                        unsafe { &(*$PX::ptr()).pin_cnf[$i] }.write(|w| {
                            w.dir().input();
                            w.input().connect();
                            w.pull().pulldown();
                            w.drive().s0s1();
                            w.sense().disabled();
                            w
                        });

                        $PXi {
                            _mode: PhantomData,
                        }
                    }
                    pub fn into_pullup_input(self) -> $PXi<Input<PullUp>> {
                        unsafe { &(*$PX::ptr()).pin_cnf[$i] }.write(|w| {
                            w.dir().input();
                            w.input().connect();
                            w.pull().pullup();
                            w.drive().s0s1();
                            w.sense().disabled();
                            w
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
                            Level::Low  => pin.set_low().unwrap(),
                            Level::High => pin.set_high().unwrap(),
                        }

                        unsafe { &(*$PX::ptr()).pin_cnf[$i] }.write(|w| {
                            w.dir().output();
                            w.input().disconnect();
                            w.pull().disabled();
                            w.drive().s0s1();
                            w.sense().disabled();
                            w
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
                            Level::Low  => pin.set_low().unwrap(),
                            Level::High => pin.set_high().unwrap(),
                        }

                        // This is safe, as we restrict our access to the
                        // dedicated register for this pin.
                        let pin_cnf = unsafe {
                            &(*$PX::ptr()).pin_cnf[$i]
                        };
                        pin_cnf.write(|w| {
                            w.dir().output();
                            w.input().disconnect();
                            w.pull().disabled();
                            w.drive().variant(config.variant());
                            w.sense().disabled();
                            w
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
                    type Error = Void;

                    fn is_high(&self) -> Result<bool, Self::Error> {
                        self.is_low().map(|v| !v)
                    }

                    fn is_low(&self) -> Result<bool, Self::Error> {
                        Ok(unsafe { ((*$PX::ptr()).in_.read().bits() & (1 << $i)) == 0 })
                    }
                }

                impl<MODE> OutputPin for $PXi<Output<MODE>> {
                    type Error = Void;

                    /// Set the output as high
                    fn set_high(&mut self) -> Result<(), Self::Error> {
                        // NOTE(unsafe) atomic write to a stateless register - TODO(AJM) verify?
                        // TODO - I wish I could do something like `.pins$i()`...
                        unsafe { (*$PX::ptr()).outset.write(|w| w.bits(1u32 << $i)); }
                        Ok(())
                    }

                    /// Set the output as low
                    fn set_low(&mut self) -> Result<(), Self::Error> {
                        // NOTE(unsafe) atomic write to a stateless register - TODO(AJM) verify?
                        // TODO - I wish I could do something like `.pins$i()`...
                        unsafe { (*$PX::ptr()).outclr.write(|w| w.bits(1u32 << $i)); }
                        Ok(())
                    }
                }

                impl<MODE> StatefulOutputPin for $PXi<Output<MODE>> {
                    /// Is the output pin set as high?
                    fn is_set_high(&self) -> Result<bool, Self::Error> {
                        self.is_set_low().map(|v| !v)
                    }

                    /// Is the output pin set as low?
                    fn is_set_low(&self) -> Result<bool, Self::Error> {
                        // NOTE(unsafe) atomic read with no side effects - TODO(AJM) verify?
                        // TODO - I wish I could do something like `.pins$i()`...
                        Ok(unsafe { ((*$PX::ptr()).out.read().bits() & (1 << $i)) == 0 })
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
