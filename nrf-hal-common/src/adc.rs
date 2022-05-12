//! API for the Analog to Digital converter.

use embedded_hal::adc::{Channel, OneShot};

use core::hint::unreachable_unchecked;

use crate::{
    gpio::{Floating, Input},
    pac::{
        adc::config::{INPSEL_A as InputSelection, REFSEL_A as Reference, RES_A as Resolution},
        ADC,
    },
};

pub struct Adc(ADC);

impl Adc {
    pub fn new(adc: ADC, config: AdcConfig) -> Self {
        while adc.busy.read().busy().is_busy() {}

        adc.config.write(|w| {
            let w1 = match config.resolution {
                Resolution::_8BIT => w.res()._8bit(),
                Resolution::_9BIT => w.res()._9bit(),
                Resolution::_10BIT => w.res()._10bit(),
            };

            let w2 = match config.input_selection {
                InputSelection::ANALOGINPUTNOPRESCALING => w1.inpsel().analog_input_no_prescaling(),
                InputSelection::ANALOGINPUTTWOTHIRDSPRESCALING => {
                    w1.inpsel().analog_input_two_thirds_prescaling()
                }
                InputSelection::ANALOGINPUTONETHIRDPRESCALING => {
                    w1.inpsel().analog_input_one_third_prescaling()
                }
                InputSelection::SUPPLYTWOTHIRDSPRESCALING => {
                    w1.inpsel().supply_two_thirds_prescaling()
                }
                InputSelection::SUPPLYONETHIRDPRESCALING => {
                    w1.inpsel().supply_one_third_prescaling()
                }
            };

            let w3 = match config.reference {
                Reference::VBG => w2.refsel().vbg(),
                Reference::EXTERNAL => w2.refsel().external(),
                Reference::SUPPLYONEHALFPRESCALING => w2.refsel().supply_one_half_prescaling(),
                Reference::SUPPLYONETHIRDPRESCALING => w2.refsel().supply_one_third_prescaling(),
            };

            w3
        });

        adc.enable.write(|w| w.enable().enabled());

        Self(adc)
    }
}

pub struct AdcConfig {
    pub resolution: Resolution,
    pub input_selection: InputSelection,
    pub reference: Reference,
}

// 0 volts reads as 0, VDD volts reads as 2^10.
impl Default for AdcConfig {
    fn default() -> Self {
        Self {
            resolution: Resolution::_10BIT,
            input_selection: InputSelection::ANALOGINPUTONETHIRDPRESCALING,
            reference: Reference::SUPPLYONETHIRDPRESCALING,
        }
    }
}

impl<PIN> OneShot<Adc, i16, PIN> for Adc
where
    PIN: Channel<Adc, ID = u8>,
{
    type Error = ();

    fn read(&mut self, _pin: &mut PIN) -> nb::Result<i16, Self::Error> {
        let original_inpsel = self.0.config.read().inpsel();
        match PIN::channel() {
            0 => self.0.config.modify(|_, w| w.psel().analog_input0()),
            1 => self.0.config.modify(|_, w| w.psel().analog_input1()),
            2 => self.0.config.modify(|_, w| w.psel().analog_input2()),
            3 => self.0.config.modify(|_, w| w.psel().analog_input3()),
            4 => self.0.config.modify(|_, w| w.psel().analog_input4()),
            5 => self.0.config.modify(|_, w| w.psel().analog_input5()),
            6 => self.0.config.modify(|_, w| w.psel().analog_input6()),
            7 => self.0.config.modify(|_, w| w.psel().analog_input7()),
            8 => self
                .0
                .config
                .modify(|_, w| w.inpsel().supply_one_third_prescaling()),
            9 => self
                .0
                .config
                .modify(|_, w| w.inpsel().supply_two_thirds_prescaling()),
            // This can never happen the only analog pins have already been defined
            // PAY CLOSE ATTENTION TO ANY CHANGES TO THIS IMPL OR THE `channel_mappings!` MACRO
            _ => unsafe { unreachable_unchecked() },
        }

        self.0.events_end.write(|w| unsafe { w.bits(0) });
        self.0.tasks_start.write(|w| unsafe { w.bits(1) });

        while self.0.events_end.read().bits() == 0 {}

        self.0.events_end.write(|w| unsafe { w.bits(0) });
        // Restore original input selection
        self.0
            .config
            .modify(|_, w| w.inpsel().variant(original_inpsel.variant().unwrap()));

        // Max resolution is 10 bits so casting is always safe
        Ok(self.0.result.read().result().bits() as i16)
    }
}

macro_rules! channel_mappings {
    ($($n:expr => $pin:path),*) => {
        $(
            impl Channel<Adc> for $pin {
                type ID = u8;

                fn channel() -> <Self as embedded_hal::adc::Channel<Adc>>::ID {
                    $n
                }
            }
        )*
    };
}

channel_mappings! {
    0 => crate::gpio::p0::P0_26<Input<Floating>>,
    1 => crate::gpio::p0::P0_27<Input<Floating>>,
    2 => crate::gpio::p0::P0_01<Input<Floating>>,
    3 => crate::gpio::p0::P0_02<Input<Floating>>,
    4 => crate::gpio::p0::P0_03<Input<Floating>>,
    5 => crate::gpio::p0::P0_04<Input<Floating>>,
    6 => crate::gpio::p0::P0_05<Input<Floating>>,
    7 => crate::gpio::p0::P0_06<Input<Floating>>,
    8 => crate::adc::InternalVddOneThird,
    9 => crate::adc::InternalVddTwoThirds
}

pub struct InternalVddOneThird;
pub struct InternalVddTwoThirds;
