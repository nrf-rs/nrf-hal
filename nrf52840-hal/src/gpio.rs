//! General Purpose Input / Output

use core::marker::PhantomData;

/// Extension trait to split a P0 peripheral in independent pins and registers
pub trait GpioExt {
    /// The to split the P0 into
    type Parts;

    /// Splits the P0 block into independent pins and registers
    fn split(self) -> Self::Parts;
}

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

/// Open drain input or output (type state)
pub struct OpenDrain;

/// Output mode (type state)
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Push pull output (type state)
pub struct PushPull;

macro_rules! gpio {
    ($GPIOX:ident, $gpiox:ident, $PXx:ident, [
        $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty),)+
    ]) => {
        /// P0
        pub mod $gpiox {
            use core::marker::PhantomData;

            use hal::digital::{InputPin, OutputPin, StatefulOutputPin};
            use nrf52840::P0;

            use super::{
                Floating, GpioExt, Input, OpenDrain, Output,
                PullDown, PullUp, PushPull,
            };

            /// P0 parts
            pub struct Parts {
                $(
                    /// Pin
                    pub $pxi: $PXi<$MODE>,
                )+
            }

            impl GpioExt for $GPIOX {
                type Parts = Parts;

                fn split(self) -> Parts {
                    Parts {
                        $(
                            $pxi: $PXi { _mode: PhantomData },
                        )+
                    }
                }
            }

            /// Partially erased pin
            pub struct $PXx<MODE> {
                i: u8,
                _mode: PhantomData<MODE>,
            }

            impl<MODE> StatefulOutputPin for $PXx<Output<MODE>> {
                fn is_set_high(&self) -> bool {
                    !self.is_set_low()
                }

                fn is_set_low(&self) -> bool {
                    // NOTE(unsafe) atomic read with no side effects
                    unsafe { (*P0::ptr()).out.read().bits() & (1 << self.i) == 0 }
                }
            }

            impl<MODE> OutputPin for $PXx<Output<MODE>> {
                fn set_high(&mut self) {
                    // NOTE(unsafe) atomic write to a stateless register
                    unsafe { (*P0::ptr()).outset.write(|w| w.bits(1 << self.i)) }
                }

                fn set_low(&mut self) {
                    // NOTE(unsafe) atomic write to a stateless register
                    unsafe { (*P0::ptr()).outclr.write(|w| w.bits(1 << self.i)) }
                }
            }

            impl<MODE> InputPin for $PXx<Input<MODE>> {
                fn is_high(&self) -> bool {
                    !self.is_low()
                }

                fn is_low(&self) -> bool {
                    // NOTE(unsafe) atomic read with no side effects
                    unsafe { (*P0::ptr()).in_.read().bits() & (1 << self.i) == 0 }
                }
            }

            $(
                /// Pin
                pub struct $PXi<MODE> {
                    _mode: PhantomData<MODE>,
                }

                impl<MODE> $PXi<MODE> {
                    /// Configures the pin to operate as a floating input pin
                    pub fn into_floating_input(
                        self,
                    ) -> $PXi<Input<Floating>> {
                        let pincnf = unsafe { &(*P0::ptr()).pin_cnf[$i] };
                        pincnf.write(|w| {
                            w.dir()
                                .input()
                                .drive()
                                .s0s1()
                                .pull()
                                .disabled()
                                .sense()
                                .disabled()
                                .input()
                                .connect()
                        });
                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as a open drain input pin
                    pub fn into_open_drain_input(
                        self,
                    ) -> $PXi<Input<OpenDrain>> {
                        let pincnf = unsafe { &(*P0::ptr()).pin_cnf[$i] };
                        pincnf.write(|w| {
                            w.dir()
                                .input()
                                .drive()
                                .s0d1()
                                .pull()
                                .disabled()
                                .sense()
                                .disabled()
                                .input()
                                .connect()
                        });
                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as a pulled down input pin
                    pub fn into_pull_down_input(
                        self,
                        ) -> $PXi<Input<PullDown>> {
                        let pincnf = unsafe { &(*P0::ptr()).pin_cnf[$i] };
                        pincnf.write(|w| {
                            w.dir()
                                .input()
                                .drive()
                                .s0s1()
                                .pull()
                                .pulldown()
                                .sense()
                                .disabled()
                                .input()
                                .connect()
                        });

                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as a pulled up input pin
                    pub fn into_pull_up_input(
                        self,
                    ) -> $PXi<Input<PullUp>> {
                        let pincnf = unsafe { &(*P0::ptr()).pin_cnf[$i] };
                        pincnf.write(|w| {
                            w.dir()
                                .input()
                                .drive()
                                .s0s1()
                                .pull()
                                .pullup()
                                .sense()
                                .disabled()
                                .input()
                                .connect()
                        });

                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as an open drain output pin
                    pub fn into_open_drain_output(
                        self,
                    ) -> $PXi<Output<OpenDrain>> {
                        let pincnf = unsafe { &(*P0::ptr()).pin_cnf[$i] };
                        pincnf.write(|w| {
                            w.dir()
                                .output()
                                .drive()
                                .s0d1()
                                .pull()
                                .disabled()
                                .sense()
                                .disabled()
                                .input()
                                .disconnect()
                        });

                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as an push pull output pin
                    pub fn into_push_pull_output(
                        self,
                    ) -> $PXi<Output<PushPull>> {

                        let pincnf = unsafe { &(*P0::ptr()).pin_cnf[$i] };
                        pincnf.write(|w| {
                            w.dir()
                                .output()
                                .drive()
                                .s0s1()
                                .pull()
                                .disabled()
                                .sense()
                                .disabled()
                                .input()
                                .disconnect()
                        });

                        $PXi { _mode: PhantomData }
                    }
                }

                impl $PXi<Output<OpenDrain>> {
                    /// Enables / disables the internal pull up
                    pub fn internal_pull_up(&mut self, on: bool) {
                        let pincnf = unsafe { &(*P0::ptr()).pin_cnf[$i] };
                        if on {
                            pincnf.modify(|_, w| w.pull().pullup());
                        } else {
                            pincnf.modify(|_, w| w.pull().disabled());
                        }
                    }
                }

                impl<MODE> $PXi<Output<MODE>> {
                    /// Erases the pin number from the type
                    ///
                    /// This is useful when you want to collect the pins into an array where you
                    /// need all the elements to have the same type
                    pub fn downgrade(self) -> $PXx<Output<MODE>> {
                        $PXx {
                            i: $i,
                            _mode: self._mode,
                        }
                    }
                }

                impl<MODE> StatefulOutputPin for $PXi<Output<MODE>> {
                    fn is_set_high(&self) -> bool {
                        !self.is_set_low()
                    }

                    fn is_set_low(&self) -> bool {
                        // NOTE(unsafe) atomic read with no side effects
                        unsafe { (*P0::ptr()).out.read().bits() & (1 << $i) == 0 }
                    }
                }

                impl<MODE> OutputPin for $PXi<Output<MODE>> {
                    fn set_high(&mut self) {
                        // NOTE(unsafe) atomic write to a stateless register
                        //unsafe { (*P0::ptr()).outset.write(|w| w.bits(1 << $i)) }
                        unsafe { (*P0::ptr()).outset.write(|w| w.bits(1 << $i)) }
                    }

                    fn set_low(&mut self) {
                        // NOTE(unsafe) atomic write to a stateless register
                        unsafe { (*P0::ptr()).outclr.write(|w| w.bits(1 << $i)) }
                    }
                }

                impl<MODE> $PXi<Input<MODE>> {
                    /// Erases the pin number from the type
                    ///
                    /// This is useful when you want to collect the pins into an array where you
                    /// need all the elements to have the same type
                    pub fn downgrade(self) -> $PXx<Input<MODE>> {
                        $PXx {
                            i: $i,
                            _mode: self._mode,
                        }
                    }
                }

                impl<MODE> InputPin for $PXi<Input<MODE>> {
                    fn is_high(&self) -> bool {
                        !self.is_low()
                    }

                    fn is_low(&self) -> bool {
                        // NOTE(unsafe) atomic read with no side effects
                        unsafe { (*P0::ptr()).in_.read().bits() & (1 << $i) == 0 }
                    }
                }
            )+

                impl<TYPE> $PXx<TYPE> {
                    pub fn get_id (&self) -> u8
                    {
                        self.i
                    }
                }
        }
    }
}

gpio!(P0, gpio, PIN, [
    PIN0: (pin0, 0, Input<Floating>),
    PIN1: (pin1, 1, Input<Floating>),
    PIN2: (pin2, 2, Input<Floating>),
    PIN3: (pin3, 3, Input<Floating>),
    PIN4: (pin4, 4, Input<Floating>),
    PIN5: (pin5, 5, Input<Floating>),
    PIN6: (pin6, 6, Input<Floating>),
    PIN7: (pin7, 7, Input<Floating>),
    PIN8: (pin8, 8, Input<Floating>),
    PIN9: (pin9, 9, Input<Floating>),
    PIN10: (pin10, 10, Input<Floating>),
    PIN11: (pin11, 11, Input<Floating>),
    PIN12: (pin12, 12, Input<Floating>),
    PIN13: (pin13, 13, Input<Floating>),
    PIN14: (pin14, 14, Input<Floating>),
    PIN15: (pin15, 15, Input<Floating>),
    PIN16: (pin16, 16, Input<Floating>),
    PIN17: (pin17, 17, Input<Floating>),
    PIN18: (pin18, 18, Input<Floating>),
    PIN19: (pin19, 19, Input<Floating>),
    PIN20: (pin20, 20, Input<Floating>),
    PIN21: (pin21, 21, Input<Floating>),
    PIN22: (pin22, 22, Input<Floating>),
    PIN23: (pin23, 23, Input<Floating>),
    PIN24: (pin24, 24, Input<Floating>),
    PIN25: (pin25, 25, Input<Floating>),
    PIN26: (pin26, 26, Input<Floating>),
    PIN27: (pin27, 27, Input<Floating>),
    PIN28: (pin28, 28, Input<Floating>),
    PIN29: (pin29, 29, Input<Floating>),
    PIN30: (pin30, 30, Input<Floating>),
    PIN31: (pin31, 31, Input<Floating>),
]);
