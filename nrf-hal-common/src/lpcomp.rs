//! HAL interface for the LPCOMP peripheral.
//!
//! In System ON, the LPCOMP can generate separate events on rising and falling edges of a signal,
//! or sample the current state of the pin as being above or below the selected reference.
//! The block can be configured to use any of the analog inputs on the device.
//! Additionally, the low power comparator can be used as an analog wakeup source from System OFF.
//! The comparator threshold can be programmed to a range of fractions of the supply voltage
//! or to use an external analog reference input pin.

use {
    crate::gpio::{p0::*, Floating, Input},
    crate::pac::{
        lpcomp::{extrefsel::EXTREFSEL_A, psel::PSEL_A, EVENTS_CROSS, EVENTS_DOWN, EVENTS_UP},
        LPCOMP,
    },
};

/// A safe wrapper around the `LPCOMP` peripheral.
pub struct LpComp {
    lpcomp: LPCOMP,
}

impl LpComp {
    /// Takes ownership of the `LPCOMP` peripheral, returning a safe wrapper
    /// using specified input pin and a default Vref of Vdd/2.
    pub fn new<P: LpCompInputPin>(lpcomp: LPCOMP, input_pin: &P) -> Self {
        lpcomp.psel.write(|w| w.psel().variant(input_pin.ain()));
        lpcomp
            .refsel
            .write(|w| w.refsel().bits(VRef::_4_8Vdd.into()));
        Self { lpcomp }
    }

    /// Selects comparator Vref.
    #[inline(always)]
    pub fn vref(&self, vref: VRef) -> &Self {
        self.lpcomp.refsel.write(|w| w.refsel().bits(vref.into()));
        self
    }

    /// Sets analog reference pin.
    #[inline(always)]
    pub fn aref_pin<P: LpCompRefPin>(&self, ref_pin: &P) -> &Self {
        self.lpcomp
            .extrefsel
            .write(|w| w.extrefsel().variant(ref_pin.aref()));
        self
    }

    /// Enables/disables comparator hysteresis.
    #[cfg(not(feature = "51"))]
    #[inline(always)]
    pub fn hysteresis(&self, enabled: bool) -> &Self {
        self.lpcomp.hyst.write(|w| match enabled {
            true => w.hyst().set_bit(),
            false => w.hyst().clear_bit(),
        });
        self
    }

    /// `Analog detect` event configuration, used for analog signal power up from OFF.
    #[cfg(not(feature = "51"))]
    #[inline(always)]
    pub fn analog_detect(&self, event: Transition) -> &Self {
        self.lpcomp.anadetect.write(|w| match event {
            Transition::Cross => w.anadetect().cross(),
            Transition::Down => w.anadetect().down(),
            Transition::Up => w.anadetect().up(),
        });
        self
    }

    /// Enables `COMP_LPCOMP` interrupt triggering on the specified event.
    #[inline(always)]
    pub fn enable_interrupt(&self, event: Transition) -> &Self {
        self.lpcomp.intenset.modify(|_r, w| match event {
            Transition::Cross => w.cross().set_bit(),
            Transition::Down => w.down().set_bit(),
            Transition::Up => w.up().set_bit(),
        });
        self
    }

    /// Disables `COMP_LPCOMP` interrupt triggering on the specified event.
    #[inline(always)]
    pub fn disable_interrupt(&self, event: Transition) -> &Self {
        self.lpcomp.intenclr.modify(|_r, w| match event {
            Transition::Cross => w.cross().set_bit(),
            Transition::Down => w.down().set_bit(),
            Transition::Up => w.up().set_bit(),
        });
        self
    }

    /// Enables the comparator and waits until it's ready to use.
    #[inline(always)]
    pub fn enable(&self) {
        self.lpcomp.enable.write(|w| w.enable().enabled());
        self.lpcomp.tasks_start.write(|w| unsafe { w.bits(1) });
        while self.lpcomp.events_ready.read().bits() == 0 {}
    }

    /// Disables the comparator.
    #[inline(always)]
    pub fn disable(&self) {
        self.lpcomp.tasks_stop.write(|w| unsafe { w.bits(1) });
        self.lpcomp.enable.write(|w| w.enable().disabled());
    }

    /// Checks if the `Up` transition event has been triggered.
    #[inline(always)]
    pub fn is_up(&self) -> bool {
        self.lpcomp.events_up.read().bits() != 0
    }

    /// Checks if the `Down` transition event has been triggered.
    #[inline(always)]
    pub fn is_down(&self) -> bool {
        self.lpcomp.events_down.read().bits() != 0
    }

    /// Checks if the `Cross` transition event has been triggered.
    #[inline(always)]
    pub fn is_cross(&self) -> bool {
        self.lpcomp.events_cross.read().bits() != 0
    }

    /// Returns reference to `Up` transition event endpoint for PPI.
    #[inline(always)]
    pub fn event_up(&self) -> &EVENTS_UP {
        &self.lpcomp.events_up
    }

    /// Returns reference to `Down` transition event endpoint for PPI.
    #[inline(always)]
    pub fn event_down(&self) -> &EVENTS_DOWN {
        &self.lpcomp.events_down
    }

    /// Returns reference to `Cross` transition event endpoint for PPI.
    #[inline(always)]
    pub fn event_cross(&self) -> &EVENTS_CROSS {
        &self.lpcomp.events_cross
    }

    /// Marks event as handled.
    #[inline(always)]
    pub fn reset_event(&self, event: Transition) {
        match event {
            Transition::Cross => self.lpcomp.events_cross.reset(),
            Transition::Down => self.lpcomp.events_down.reset(),
            Transition::Up => self.lpcomp.events_up.reset(),
        }
    }

    /// Marks all events as handled.
    #[inline(always)]
    pub fn reset_events(&self) {
        self.lpcomp.events_cross.reset();
        self.lpcomp.events_down.reset();
        self.lpcomp.events_up.reset();
    }

    /// Returns the output state of the comparator.
    #[inline(always)]
    pub fn read(&self) -> CompResult {
        self.lpcomp.tasks_sample.write(|w| unsafe { w.bits(1) });
        match self.lpcomp.result.read().result().is_above() {
            true => CompResult::Above,
            false => CompResult::Below,
        }
    }

    /// Consumes `self` and returns back the raw `LPCOMP` peripheral.
    #[inline(always)]
    pub fn free(self) -> LPCOMP {
        self.lpcomp
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CompResult {
    Above,
    Below,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Transition {
    Up,
    Down,
    Cross,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum VRef {
    _1_8Vdd = 0,
    _2_8Vdd = 1,
    _3_8Vdd = 2,
    _4_8Vdd = 3,
    _5_8Vdd = 4,
    _6_8Vdd = 5,
    _7_8Vdd = 6,
    ARef = 7,
    #[cfg(not(feature = "51"))]
    _1_16Vdd = 8,
    #[cfg(not(feature = "51"))]
    _3_16Vdd = 9,
    #[cfg(not(feature = "51"))]
    _5_16Vdd = 10,
    #[cfg(not(feature = "51"))]
    _7_16Vdd = 11,
    #[cfg(not(feature = "51"))]
    _9_16Vdd = 12,
    #[cfg(not(feature = "51"))]
    _11_16Vdd = 13,
    #[cfg(not(feature = "51"))]
    _13_16Vdd = 14,
    #[cfg(not(feature = "51"))]
    _15_16Vdd = 15,
}

impl From<VRef> for u8 {
    #[inline(always)]
    fn from(variant: VRef) -> Self {
        variant as _
    }
}
/// Trait to represent analog input pins.
pub trait LpCompInputPin {
    fn ain(&self) -> PSEL_A;
}
/// Trait to represent analog ref pins.
pub trait LpCompRefPin {
    fn aref(&self) -> EXTREFSEL_A;
}

macro_rules! comp_input_pins {
    ($($pin:path => $ain:expr,)+) => {
        $(
            impl LpCompInputPin for $pin {
                fn ain(&self) -> PSEL_A {
                    $ain
                }
            }
        )*
    };
}

macro_rules! comp_ref_pins {
    ($($pin:path => $aref:expr,)+) => {
        $(
            impl LpCompRefPin for $pin {
                fn aref(&self) -> EXTREFSEL_A {
                    $aref
                }
            }
        )*
    };
}

#[cfg(not(feature = "51"))]
comp_ref_pins! {
    P0_02<Input<Floating>> => EXTREFSEL_A::ANALOG_REFERENCE0,
    P0_03<Input<Floating>> => EXTREFSEL_A::ANALOG_REFERENCE1,
}

#[cfg(not(feature = "51"))]
comp_input_pins! {
    P0_02<Input<Floating>> => PSEL_A::ANALOG_INPUT0,
    P0_03<Input<Floating>> => PSEL_A::ANALOG_INPUT1,
    P0_04<Input<Floating>> => PSEL_A::ANALOG_INPUT2,
    P0_05<Input<Floating>> => PSEL_A::ANALOG_INPUT3,
    P0_28<Input<Floating>> => PSEL_A::ANALOG_INPUT4,
    P0_29<Input<Floating>> => PSEL_A::ANALOG_INPUT5,
    P0_30<Input<Floating>> => PSEL_A::ANALOG_INPUT6,
    P0_31<Input<Floating>> => PSEL_A::ANALOG_INPUT7,
}

#[cfg(feature = "51")]
comp_ref_pins! {
    P0_00<Input<Floating>> => EXTREFSEL_A::ANALOG_REFERENCE0,
    P0_06<Input<Floating>> => EXTREFSEL_A::ANALOG_REFERENCE1,
}

#[cfg(feature = "51")]
comp_input_pins! {
    P0_26<Input<Floating>> => PSEL_A::ANALOG_INPUT0,
    P0_27<Input<Floating>> => PSEL_A::ANALOG_INPUT1,
    P0_01<Input<Floating>> => PSEL_A::ANALOG_INPUT2,
    P0_02<Input<Floating>> => PSEL_A::ANALOG_INPUT3,
    P0_03<Input<Floating>> => PSEL_A::ANALOG_INPUT4,
    P0_04<Input<Floating>> => PSEL_A::ANALOG_INPUT5,
    P0_05<Input<Floating>> => PSEL_A::ANALOG_INPUT6,
    P0_06<Input<Floating>> => PSEL_A::ANALOG_INPUT7,
}
