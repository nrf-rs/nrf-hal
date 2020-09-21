//! HAL interface for the COMP peripheral.
//!
//! The comparator (COMP) compares an input voltage (Vin) against a second input voltage (Vref).
//! Vin can be derived from an analog input pin (AIN0-AIN7).
//! Vref can be derived from multiple sources depending on the operation mode of the comparator.

use {
    crate::gpio::{p0::*, Floating, Input},
    crate::pac::{
        comp::{extrefsel::EXTREFSEL_A, psel::PSEL_A, _EVENTS_CROSS, _EVENTS_DOWN, _EVENTS_UP},
        generic::Reg,
        COMP,
    },
};

/// A safe wrapper around the `COMP` peripheral.
pub struct Comp {
    comp: COMP,
}

impl Comp {
    /// Takes ownership of the `COMP` peripheral, returning a safe wrapper.
    pub fn new<P: CompInputPin>(comp: COMP, input_pin: &P) -> Self {
        comp.psel.write(|w| w.psel().variant(input_pin.ain()));
        comp.mode.write(|w| w.sp().normal());
        comp.mode.write(|w| w.main().se());
        comp.refsel.write(|w| w.refsel().int1v2());
        Self { comp }
    }

    /// Sets the speed and power mode of the comparator.
    #[inline(always)]
    pub fn power_mode(&self, mode: PowerMode) -> &Self {
        match mode {
            PowerMode::LowPower => self.comp.mode.write(|w| w.sp().low()),
            PowerMode::Normal => self.comp.mode.write(|w| w.sp().normal()),
            PowerMode::HighSpeed => self.comp.mode.write(|w| w.sp().high()),
        }
        self
    }

    /// Sets Vref of the comparator in single ended mode.
    #[inline(always)]
    pub fn vref(&self, vref: VRef) -> &Self {
        self.comp.refsel.write(|w| match vref {
            VRef::Int1V2 => w.refsel().int1v2(),
            VRef::Int1V8 => w.refsel().int1v8(),
            VRef::Int2V4 => w.refsel().int2v4(),
            VRef::Vdd => w.refsel().vdd(),
            VRef::ARef(r) => {
                self.comp.extrefsel.write(|w| w.extrefsel().variant(r));
                w.refsel().aref()
            }
        });
        self
    }

    /// Sets analog reference pin.
    #[inline(always)]
    pub fn aref_pin<P: CompRefPin>(&self, ref_pin: &P) -> &Self {
        self.comp
            .extrefsel
            .write(|w| w.extrefsel().variant(ref_pin.aref()));
        self
    }

    /// Sets comparator mode to differential with external Vref pin.
    #[inline(always)]
    pub fn differential<P: CompRefPin>(&self, ref_pin: &P) -> &Self {
        self.comp.mode.write(|w| w.main().diff());
        self.aref_pin(ref_pin);
        self
    }

    /// Upward hysteresis threshold in single ended mode `Vup = (value+1)/64*Vref`.
    #[inline(always)]
    pub fn hysteresis_threshold_up(&self, value: u8) -> &Self {
        self.comp
            .th
            .write(|w| unsafe { w.thup().bits(value.min(63)) });
        self
    }

    /// Downward hysteresis threshold in single ended mode `Vdown = (value+1)/64*Vref`.
    #[inline(always)]
    pub fn hysteresis_threshold_down(&self, value: u8) -> &Self {
        self.comp
            .th
            .write(|w| unsafe { w.thdown().bits(value.min(63)) });
        self
    }

    /// Enables/disables differential comparator hysteresis (50mV).
    #[inline(always)]
    pub fn hysteresis(&self, enabled: bool) -> &Self {
        self.comp.hyst.write(|w| match enabled {
            true => w.hyst().hyst50m_v(),
            false => w.hyst().no_hyst(),
        });
        self
    }

    /// Enables `COMP_LPCOMP` interrupt triggering on the specified event.
    #[inline(always)]
    pub fn enable_interrupt(&self, event: Transition) -> &Self {
        self.comp.intenset.modify(|_r, w| match event {
            Transition::Cross => w.cross().set_bit(),
            Transition::Down => w.down().set_bit(),
            Transition::Up => w.up().set_bit(),
        });
        self
    }

    /// Disables `COMP_LPCOMP` interrupt triggering on the specified event.
    #[inline(always)]
    pub fn disable_interrupt(&self, event: Transition) -> &Self {
        self.comp.intenclr.modify(|_r, w| match event {
            Transition::Cross => w.cross().set_bit(),
            Transition::Down => w.down().set_bit(),
            Transition::Up => w.up().set_bit(),
        });
        self
    }

    /// Enables the comparator and waits until it's ready to use.
    #[inline(always)]
    pub fn enable(&self) {
        self.comp.enable.write(|w| w.enable().enabled());
        self.comp.tasks_start.write(|w| unsafe { w.bits(1) });
        while self.comp.events_ready.read().bits() == 0 {}
    }

    /// Disables the comparator.
    #[inline(always)]
    pub fn disable(&self) {
        self.comp.tasks_stop.write(|w| unsafe { w.bits(1) });
        self.comp.enable.write(|w| w.enable().disabled());
    }

    /// Checks if the `Up` transition event has been triggered.
    #[inline(always)]
    pub fn is_up(&self) -> bool {
        self.comp.events_up.read().bits() != 0
    }

    /// Checks if the `Down` transition event has been triggered.
    #[inline(always)]
    pub fn is_down(&self) -> bool {
        self.comp.events_down.read().bits() != 0
    }

    /// Checks if the `Cross` transition event has been triggered.
    #[inline(always)]
    pub fn is_cross(&self) -> bool {
        self.comp.events_cross.read().bits() != 0
    }

    /// Returns reference to `Up` transition event endpoint for PPI.
    #[inline(always)]
    pub fn event_up(&self) -> &Reg<u32, _EVENTS_UP> {
        &self.comp.events_up
    }

    /// Returns reference to `Down` transition event endpoint for PPI.
    #[inline(always)]
    pub fn event_down(&self) -> &Reg<u32, _EVENTS_DOWN> {
        &self.comp.events_down
    }

    /// Returns reference to `Cross` transition event endpoint for PPI.
    #[inline(always)]
    pub fn event_cross(&self) -> &Reg<u32, _EVENTS_CROSS> {
        &self.comp.events_cross
    }

    /// Marks event as handled.
    #[inline(always)]
    pub fn reset_event(&self, event: Transition) {
        match event {
            Transition::Cross => self.comp.events_cross.write(|w| w),
            Transition::Down => self.comp.events_down.write(|w| w),
            Transition::Up => self.comp.events_up.write(|w| w),
        }
    }

    /// Marks all events as handled.
    #[inline(always)]
    pub fn reset_events(&self) {
        self.comp.events_cross.write(|w| w);
        self.comp.events_down.write(|w| w);
        self.comp.events_up.write(|w| w);
    }

    /// Returns the output state of the comparator.
    #[inline(always)]
    pub fn read(&self) -> CompResult {
        self.comp.tasks_sample.write(|w| unsafe { w.bits(1) });
        match self.comp.result.read().result().is_above() {
            true => CompResult::Above,
            false => CompResult::Below,
        }
    }

    /// Consumes `self` and returns back the raw `COMP` peripheral.
    #[inline(always)]
    pub fn free(self) -> COMP {
        self.comp
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum OperationMode {
    Differential,
    SingleEnded,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PowerMode {
    LowPower,
    Normal,
    HighSpeed,
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
    Int1V2,
    Int1V8,
    Int2V4,
    Vdd,
    ARef(EXTREFSEL_A),
}

impl VRef {
    pub fn from_pin<P: CompRefPin>(pin: &P) -> Self {
        VRef::ARef(pin.aref())
    }
}

/// Trait to represent analog input pins.
pub trait CompInputPin {
    fn ain(&self) -> PSEL_A;
}
/// Trait to represent analog ref pins.
pub trait CompRefPin {
    fn aref(&self) -> EXTREFSEL_A;
}

macro_rules! comp_input_pins {
    ($($pin:path => $ain:expr,)+) => {
        $(
            impl CompInputPin for $pin {
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
            impl CompRefPin for $pin {
                fn aref(&self) -> EXTREFSEL_A {
                    $aref
                }
            }
        )*
    };
}

comp_ref_pins! {
    P0_02<Input<Floating>> => EXTREFSEL_A::ANALOGREFERENCE0,
    P0_03<Input<Floating>> => EXTREFSEL_A::ANALOGREFERENCE1,
    P0_04<Input<Floating>> => EXTREFSEL_A::ANALOGREFERENCE2,
    P0_05<Input<Floating>> => EXTREFSEL_A::ANALOGREFERENCE3,
    P0_28<Input<Floating>> => EXTREFSEL_A::ANALOGREFERENCE4,
    P0_29<Input<Floating>> => EXTREFSEL_A::ANALOGREFERENCE5,
    P0_30<Input<Floating>> => EXTREFSEL_A::ANALOGREFERENCE6,
    P0_31<Input<Floating>> => EXTREFSEL_A::ANALOGREFERENCE7,
}

#[cfg(not(any(feature = "52811", feature = "52810")))]
comp_input_pins! {
    P0_02<Input<Floating>> => PSEL_A::ANALOGINPUT0,
    P0_03<Input<Floating>> => PSEL_A::ANALOGINPUT1,
    P0_04<Input<Floating>> => PSEL_A::ANALOGINPUT2,
    P0_05<Input<Floating>> => PSEL_A::ANALOGINPUT3,
    P0_28<Input<Floating>> => PSEL_A::ANALOGINPUT4,
    P0_29<Input<Floating>> => PSEL_A::ANALOGINPUT5,
    P0_30<Input<Floating>> => PSEL_A::ANALOGINPUT6,
    P0_31<Input<Floating>> => PSEL_A::ANALOGINPUT6,
}

#[cfg(any(feature = "52811", feature = "52810"))]
comp_input_pins! {
    P0_02<Input<Floating>> => PSEL_A::ANALOGINPUT0,
    P0_03<Input<Floating>> => PSEL_A::ANALOGINPUT1,
    P0_04<Input<Floating>> => PSEL_A::ANALOGINPUT2,
    P0_05<Input<Floating>> => PSEL_A::ANALOGINPUT3,
    P0_28<Input<Floating>> => PSEL_A::ANALOGINPUT4,
    P0_29<Input<Floating>> => PSEL_A::ANALOGINPUT5,
    P0_30<Input<Floating>> => PSEL_A::ANALOGINPUT6,
}
