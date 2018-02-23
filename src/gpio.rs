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

/// Push pull output (type state)
pub struct PushPull;
// /// Open drain output (type state)
// pub struct OpenDrain;

// /// Alternate function
// pub struct Alternate<MODE> {
//     _mode: PhantomData<MODE>,
// }

pub mod p0 {

    use super::{
        // Alternate,
        Floating,
        // GpioExt,
        Input,
        // OpenDrain,
        Output,
        // PullDown, PullUp,
        PushPull,

        PhantomData,
    };

    use nrf52;
    use nrf52::p0::PIN_CNF;
    use hal::digital::OutputPin;


    impl P0_11_CFG {
        pub fn magic() -> Self {
            Self {
                _0: ()
            }
        }
        pub(crate) fn cfg(&mut self) -> &nrf52::p0::PIN_CNF {
            unsafe { &(*nrf52::P0::ptr()).pin_cnf[11] }
        }
    }

    pub struct P0_11_CFG {
        _0: (),
    }

    pub struct P0_11<MODE> {
        _mode: PhantomData<MODE>
    }

    impl<MODE> P0_11<MODE> {
        // TODO - type safety on PIN_CNF? just take whole `&[PIN_CNF; 32]`?
        pub fn into_floating_input(cnf: &mut P0_11_CFG) -> P0_11<Input<Floating>> {
            cnf.cfg().write(|w| {
                w.dir().input()
                 .input().connect()
                 .pull().disabled()
                 .drive().s0s1()
                 .sense().disabled()
            });

            P0_11 {
                _mode: PhantomData
            }
        }

        // TODO - type safety on PIN_CNF? just take whole `&[PIN_CNF; 32]`?
        pub fn into_push_pull_output(cnf: &mut P0_11_CFG) -> P0_11<Output<PushPull>> {
            cnf.cfg().write(|w| {
                w.dir().output()
                 .input().disconnect()
                 .pull().disabled()
                 .drive().s0s1()
                 .sense().disabled()
            });

            P0_11 {
                _mode: PhantomData
            }
        }
    }

    // TODO - impl for multiple pins
    impl<MODE> OutputPin for P0_11<Output<MODE>> {
        fn is_high(&self) -> bool {
            !self.is_low()
        }

        fn is_low(&self) -> bool {
            // NOTE(unsafe) atomic read with no side effects - TODO(AJM) verify?
            unsafe { (*nrf52::P0::ptr()).out.read().pin11().is_low() }
        }

        fn set_high(&mut self) {
            // NOTE(unsafe) atomic write to a stateless register - TODO(AJM) verify?
            unsafe { (*nrf52::P0::ptr()).outset.write(|w| w.pin11().set()); }
        }

        fn set_low(&mut self) {
            // NOTE(unsafe) atomic write to a stateless register - TODO(AJM) verify?
            unsafe { (*nrf52::P0::ptr()).outclr.write(|w| w.pin11().clear()); }
        }
    }
}





























// //! General Purpose Input / Output

// // TODO the pins here currently correspond to the LQFP-48 package. There should be Cargo features
// // that let you select different microcontroller packages

// use rcc::APB2;

// /// Extension trait to split a GPIO peripheral in independent pins and registers
// pub trait GpioExt {
//     /// The to split the GPIO into
//     type Parts;

//     /// Splits the GPIO block into independent pins and registers
//     fn split(self, apb2: &mut APB2) -> Self::Parts;
// }

// macro_rules! gpio {
//     ($GPIOX:ident, $gpiox:ident, $gpioy:ident, $iopxenr:ident, $iopxrst:ident, $PXx:ident, [
//         $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty, $CR:ident),)+
//     ]) => {
//         /// GPIO
//         pub mod $gpiox {
//             use core::marker::PhantomData;

//             use hal::digital::OutputPin;
//             use nrf52::{$gpioy, $GPIOX};

//             use rcc::APB2;
//             use super::{
//                 Alternate, Floating, GpioExt, Input,
//                 // OpenDrain,
//                 Output,
//                 // PullDown, PullUp,
//                 PushPull,
//             };

//             /// GPIO parts
//             pub struct Parts {
//                 /// Opaque CRL register
//                 pub crl: CRL,
//                 /// Opaque CRH register
//                 pub crh: CRH,
//                 $(
//                     /// Pin
//                     pub $pxi: $PXi<$MODE>,
//                 )+
//             }

//             impl GpioExt for $GPIOX {
//                 type Parts = Parts;

//                 fn split(self, apb2: &mut APB2) -> Parts {
//                     apb2.enr().modify(|_, w| w.$iopxenr().enabled());
//                     apb2.rstr().modify(|_, w| w.$iopxrst().set_bit());
//                     apb2.rstr().modify(|_, w| w.$iopxrst().clear_bit());

//                     Parts {
//                         crl: CRL { _0: () },
//                         crh: CRH { _0: () },
//                         $(
//                             $pxi: $PXi { _mode: PhantomData },
//                         )+
//                     }
//                 }
//             }

//             /// Opaque CRL register
//             pub struct CRL {
//                 _0: (),
//             }

//             impl CRL {
//                 // NOTE(allow) we get a warning on GPIOC because it only has 3 high pins
//                 #[allow(dead_code)]
//                 pub(crate) fn cr(&mut self) -> &$gpioy::CRL {
//                     unsafe { &(*$GPIOX::ptr()).crl }
//                 }
//             }

//             /// Opaque CRH register
//             pub struct CRH {
//                 _0: (),
//             }

//             impl CRH {
//                 pub(crate) fn cr(&mut self) -> &$gpioy::CRH {
//                     unsafe { &(*$GPIOX::ptr()).crh }
//                 }
//             }

//             /// Partially erased pin
//             pub struct $PXx<MODE> {
//                 i: u8,
//                 _mode: PhantomData<MODE>,
//             }

//             impl<MODE> OutputPin for $PXx<Output<MODE>> {
//                 fn is_high(&self) -> bool {
//                     !self.is_low()
//                 }

//                 fn is_low(&self) -> bool {
//                     // NOTE(unsafe) atomic read with no side effects
//                     unsafe { (*$GPIOX::ptr()).odr.read().bits() & (1 << self.i) == 0 }
//                 }

//                 fn set_high(&mut self) {
//                     // NOTE(unsafe) atomic write to a stateless register
//                     unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << self.i)) }
//                 }

//                 fn set_low(&mut self) {
//                     // NOTE(unsafe) atomic write to a stateless register
//                     unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << (16 + self.i))) }
//                 }
//             }

//             $(
//                 /// Pin
//                 pub struct $PXi<MODE> {
//                     _mode: PhantomData<MODE>,
//                 }

//                 impl<MODE> $PXi<MODE> {
//                     /// Configures the pin to operate as an alternate function push pull output pin
//                     pub fn into_alternate_push_pull(
//                         self,
//                         cr: &mut $CR,
//                     ) -> $PXi<Alternate<PushPull>> {
//                         let offset = (4 * $i) % 32;
//                         // Alternate function output push pull
//                         let cnf = 0b10;
//                         // Output mode, max speed 50 MHz
//                         let mode = 0b11;
//                         let bits = (cnf << 2) | mode;

//                         // input mode
//                         cr
//                             .cr()
//                             .modify(|r, w| unsafe {
//                                 w.bits((r.bits() & !(0b1111 << offset)) | (bits << offset))
//                             });

//                         $PXi { _mode: PhantomData }
//                     }

//                     /// Configures the pin to operate as a floating input pin
//                     pub fn into_floating_input(
//                         self,
//                         cr: &mut $CR,
//                     ) -> $PXi<Input<Floating>> {
//                         let offset = (4 * $i) % 32;
//                         // Floating input
//                         let cnf = 0b01;
//                         // Input mode
//                         let mode = 0b00;
//                         let bits = (cnf << 2) | mode;

//                         // input mode
//                         cr
//                             .cr()
//                             .modify(|r, w| unsafe {
//                                 w.bits((r.bits() & !(0b1111 << offset)) | (bits << offset))
//                             });

//                         $PXi { _mode: PhantomData }
//                     }

//                     // /// Configures the pin to operate as a pulled down input pin
//                     // pub fn into_pull_down_input(
//                     //     self,
//                     //     moder: &mut MODER,
//                     //     pupdr: &mut PUPDR,
//                     // ) -> $PXi<Input<PullDown>> {
//                     //     let offset = 2 * $i;

//                     //     // input mode
//                     //     moder
//                     //         .moder()
//                     //         .modify(|r, w| unsafe { w.bits(r.bits() & !(0b11 << offset)) });

//                     //     // pull-down
//                     //     pupdr.pupdr().modify(|r, w| unsafe {
//                     //         w.bits((r.bits() & !(0b11 << offset)) | (0b10 << offset))
//                     //     });

//                     //     $PXi { _mode: PhantomData }
//                     // }

//                     // /// Configures the pin to operate as a pulled up input pin
//                     // pub fn into_pull_up_input(
//                     //     self,
//                     //     moder: &mut MODER,
//                     //     pupdr: &mut PUPDR,
//                     // ) -> $PXi<Input<PullUp>> {
//                     //     let offset = 2 * $i;

//                     //     // input mode
//                     //     moder
//                     //         .moder()
//                     //         .modify(|r, w| unsafe { w.bits(r.bits() & !(0b11 << offset)) });

//                     //     // pull-up
//                     //     pupdr.pupdr().modify(|r, w| unsafe {
//                     //         w.bits((r.bits() & !(0b11 << offset)) | (0b01 << offset))
//                     //     });

//                     //     $PXi { _mode: PhantomData }
//                     // }

//                     // /// Configures the pin to operate as an open drain output pin
//                     // pub fn into_open_drain_output(
//                     //     self,
//                     //     moder: &mut MODER,
//                     //     otyper: &mut OTYPER,
//                     // ) -> $PXi<Output<OpenDrain>> {
//                     //     let offset = 2 * $i;

//                     //     // general purpose output mode
//                     //     let mode = 0b01;
//                     //     moder.moder().modify(|r, w| unsafe {
//                     //         w.bits((r.bits() & !(0b11 << offset)) | (mode << offset))
//                     //     });

//                     //     // open drain output
//                     //     otyper
//                     //         .otyper()
//                     //         .modify(|r, w| unsafe { w.bits(r.bits() | (0b1 << $i)) });

//                     //     $PXi { _mode: PhantomData }
//                     // }

//                     /// Configures the pin to operate as an push pull output pin
//                     pub fn into_push_pull_output(
//                         self,
//                         cr: &mut $CR,
//                     ) -> $PXi<Output<PushPull>> {
//                         let offset = (4 * $i) % 32;
//                         // General purpose output push-pull
//                         let cnf = 0b00;
//                         // Output mode, max speed 50 MHz
//                         let mode = 0b11;
//                         let bits = (cnf << 2) | mode;

//                         cr
//                             .cr()
//                             .modify(|r, w| unsafe {
//                                 w.bits((r.bits() & !(0b1111 << offset)) | (bits << offset))
//                             });

//                         $PXi { _mode: PhantomData }
//                     }
//                 }

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
//         }
//     }
// }

// gpio!(GPIOA, gpioa, gpioa, iopaen, ioparst, PAx, [
//     PA0: (pa0, 0, Input<Floating>, CRL),
//     PA1: (pa1, 1, Input<Floating>, CRL),
//     PA2: (pa2, 2, Input<Floating>, CRL),
//     PA3: (pa3, 3, Input<Floating>, CRL),
//     PA4: (pa4, 4, Input<Floating>, CRL),
//     PA5: (pa5, 5, Input<Floating>, CRL),
//     PA6: (pa6, 6, Input<Floating>, CRL),
//     PA7: (pa7, 7, Input<Floating>, CRL),
//     PA8: (pa8, 8, Input<Floating>, CRH),
//     PA9: (pa9, 9, Input<Floating>, CRH),
//     PA10: (pa10, 10, Input<Floating>, CRH),
//     PA11: (pa11, 11, Input<Floating>, CRH),
//     PA12: (pa12, 12, Input<Floating>, CRH),
//     PA13: (pa13, 13, Input<Floating>, CRH),
//     PA14: (pa14, 14, Input<Floating>, CRH),
//     PA15: (pa15, 15, Input<Floating>, CRH),
// ]);

// gpio!(GPIOB, gpiob, gpioa, iopben, iopbrst, PBx, [
//     PB0: (pb0, 0, Input<Floating>, CRL),
//     PB1: (pb1, 1, Input<Floating>, CRL),
//     PB2: (pb2, 2, Input<Floating>, CRL),
//     PB3: (pb3, 3, Input<Floating>, CRL),
//     PB4: (pb4, 4, Input<Floating>, CRL),
//     PB5: (pb5, 5, Input<Floating>, CRL),
//     PB6: (pb6, 6, Input<Floating>, CRL),
//     PB7: (pb7, 7, Input<Floating>, CRL),
//     PB8: (pb8, 8, Input<Floating>, CRH),
//     PB9: (pb9, 9, Input<Floating>, CRH),
//     PB10: (pb10, 10, Input<Floating>, CRH),
//     PB11: (pb11, 11, Input<Floating>, CRH),
//     PB12: (pb12, 12, Input<Floating>, CRH),
//     PB13: (pb13, 13, Input<Floating>, CRH),
//     PB14: (pb14, 14, Input<Floating>, CRH),
//     PB15: (pb15, 15, Input<Floating>, CRH),
// ]);

// gpio!(GPIOC, gpioc, gpioa, iopcen, iopcrst, PCx, [
//     PC13: (pc13, 13, Input<Floating>, CRH),
//     PC14: (pc14, 14, Input<Floating>, CRH),
//     PC15: (pc15, 15, Input<Floating>, CRH),
// ]);

