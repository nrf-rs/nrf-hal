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
// /// Pulled down input (type state)
// pub struct PullDown;
// /// Pulled up input (type state)
// pub struct PullUp;

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
// /// Open drain output (type state)
// pub struct OpenDrain;

// /// Alternate function
// pub struct Alternate<MODE> {
//     _mode: PhantomData<MODE>,
// }



macro_rules! gpio {
    (
        $PX:ident, $px:ident, $py:ident, [
            $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty),)+
        ]
    ) => {
        /// GPIO
        pub mod $px {
            use super::{
                // Alternate,
                Floating,
                GpioExt,
                Input,
                // OpenDrain,
                Output,
                // PullDown, PullUp,
                PushPull,

                PhantomData,
            };

            use nrf52;
            use nrf52::$PX;
            use nrf52::$px::PIN_CNF;
            use hal::digital::{OutputPin, InputPin};

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

                    /// Convert the pin to be a push-pull output
                    pub fn into_push_pull_output(self) -> $PXi<Output<PushPull>> {
                        unsafe { &(*$PX::ptr()).pin_cnf[$i] }.write(|w| {
                            w.dir().output()
                             .input().disconnect()
                             .pull().disabled()
                             .drive().s0s1()
                             .sense().disabled()
                        });

                        $PXi {
                            _mode: PhantomData,
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
                    /// Is the output pin set as high?
                    fn is_high(&self) -> bool {
                        !self.is_low()
                    }

                    /// Is the output pin set as low?
                    fn is_low(&self) -> bool {
                        // NOTE(unsafe) atomic read with no side effects - TODO(AJM) verify?
                        // TODO - I wish I could do something like `.pins$i()`...
                        unsafe { ((*$PX::ptr()).out.read().bits() & (1 << $i)) == 0 }
                    }

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
            )+

// TODO(AJM) - type erasure impl

//                 impl<MODE> $PXi<Output<MODE>> {
//                     /// Erases the pin number from the type
//                     ///
//                     /// This is useful when you want to collect the pins into an array where you
//                     /// need all the elements to have the same type
//                     pub fn downgrade(self) -> $PXx<Output<MODE>> {
//                         $PXx {
//                             i: $i,
//                             _mode: self._mode,
//                         }
//                     }
//                 }

//                 impl<MODE> OutputPin for $PXi<Output<MODE>> {
//                     fn is_high(&self) -> bool {
//                         !self.is_low()
//                     }

//                     fn is_low(&self) -> bool {
//                         // NOTE(unsafe) atomic read with no side effects
//                         unsafe { (*$GPIOX::ptr()).odr.read().bits() & (1 << $i) == 0 }
//                     }

//                     fn set_high(&mut self) {
//                         // NOTE(unsafe) atomic write to a stateless register
//                         unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << $i)) }
//                     }

//                     fn set_low(&mut self) {
//                         // NOTE(unsafe) atomic write to a stateless register
//                         unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << (16 + $i))) }
//                     }
//                 }
//             )+
        }
    }
}

gpio!(P0, p0, p0, [
    P0_0:  (p0_0,  0,  Input<Floating>),
    P0_1:  (p0_1,  1,  Input<Floating>),
    P0_2:  (p0_2,  2,  Input<Floating>),
    P0_3:  (p0_3,  3,  Input<Floating>),
    P0_4:  (p0_4,  4,  Input<Floating>),
    P0_5:  (p0_5,  5,  Input<Floating>),
    P0_6:  (p0_6,  6,  Input<Floating>),
    P0_7:  (p0_7,  7,  Input<Floating>),
    P0_8:  (p0_8,  8,  Input<Floating>),
    P0_9:  (p0_9,  9,  Input<Floating>),
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
