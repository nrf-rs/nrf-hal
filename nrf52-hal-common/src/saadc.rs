use crate::{
    gpio::{Floating, Input},
    target::{saadc, SAADC},
};
use core::{
    hint::unreachable_unchecked,
    ops::Deref,
    sync::atomic::{compiler_fence, Ordering::SeqCst},
};
use embedded_hal::adc::{Channel, OneShot};

pub use crate::target::saadc::{
    ch::config::{GAINW as Gain, REFSELW as Reference, RESPW as Resistor, TACQW as Time},
    oversample::OVERSAMPLEW as Oversample,
    resolution::VALW as Resolution,
};

// Only 1 channel is allowed right now, a discussion needs to be had as to how
// multiple channels should work (See "scan mode" in the datasheet)
// Issue: https://github.com/nrf-rs/nrf52-hal/issues/82

pub trait SaadcExt: Deref<Target = saadc::RegisterBlock> + Sized {
    fn constrain(self) -> Saadc;
}

impl SaadcExt for SAADC {
    fn constrain(self) -> Saadc {
        Saadc::new(self, SaadcConfig::default())
    }
}

pub struct Saadc(SAADC);

impl Saadc {
    pub fn new(saadc: SAADC, config: SaadcConfig) -> Self {
        // The write enums do not implement clone/copy/debug, only the
        // read ones, hence the need to pull out and move the values
        let SaadcConfig {
            resolution,
            oversample,
            reference,
            gain,
            resistor,
            time
        } = config;

        saadc.enable.write(|w| w.enable().enabled());
        saadc
            .resolution
            .write(|w| w.val().variant(resolution));
        saadc
            .oversample
            .write(|w| w.oversample().variant(oversample));
        saadc.samplerate.write(|w| w.mode().task());

        saadc.ch[0].config.write(|w| {
            w.refsel()
                .variant(reference)
                .gain()
                .variant(gain)
                .tacq()
                .variant(time)
                .mode()
                .se()
                .resp()
                .variant(resistor)
                .resn()
                .bypass()
                .burst()
                .enabled()
        });
        saadc.ch[0].pseln.write(|w| w.pseln().nc());

        // Calibrate
        saadc.tasks_calibrateoffset.write(|w| unsafe { w.bits(1) });
        while saadc.events_calibratedone.read().bits() == 0 {}

        Saadc(saadc)
    }
}

pub struct SaadcConfig {
    resolution: Resolution,
    oversample: Oversample,
    reference: Reference,
    gain: Gain,
    resistor: Resistor,
    time: Time,
}

// 0 volts reads as 0, VDD volts reads as u16::MAX
impl Default for SaadcConfig {
    fn default() -> Self {
        SaadcConfig {
            resolution: Resolution::_14BIT,
            oversample: Oversample::OVER8X,
            reference: Reference::VDD1_4,
            gain: Gain::GAIN1_4,
            resistor: Resistor::BYPASS,
            time: Time::_20US,
        }
    }
}

impl<PIN> OneShot<Saadc, u16, PIN> for Saadc
where
    PIN: Channel<Saadc, ID = u8>,
{
    type Error = ();
    fn read(&mut self, _pin: &mut PIN) -> nb::Result<u16, Self::Error> {
        match PIN::channel() {
            0 => self.0.ch[0].pselp.write(|w| w.pselp().analog_input0()),
            1 => self.0.ch[0].pselp.write(|w| w.pselp().analog_input1()),
            2 => self.0.ch[0].pselp.write(|w| w.pselp().analog_input2()),
            3 => self.0.ch[0].pselp.write(|w| w.pselp().analog_input3()),
            4 => self.0.ch[0].pselp.write(|w| w.pselp().analog_input4()),
            5 => self.0.ch[0].pselp.write(|w| w.pselp().analog_input5()),
            6 => self.0.ch[0].pselp.write(|w| w.pselp().analog_input6()),
            7 => self.0.ch[0].pselp.write(|w| w.pselp().analog_input7()),
            // This can never happen the only analog pins have already been defined
            // PAY CLOSE ATTENTION TO ANY CHANGES TO THIS IMPL OR THE `channel_mappings!` MACRO
            _ => unsafe { unreachable_unchecked() },
        }

        let mut val: u16 = 0;
        self.0
            .result
            .ptr
            .write(|w| unsafe { w.ptr().bits(((&mut val) as *mut _) as u32) });
        self.0
            .result
            .maxcnt
            .write(|w| unsafe { w.maxcnt().bits(1) });

        // Conservative compiler fence to prevent starting the ADC before the
        // pointer and maxcount have been set
        compiler_fence(SeqCst);

        self.0.tasks_start.write(|w| unsafe { w.bits(1) });
        self.0.tasks_sample.write(|w| unsafe { w.bits(1) });

        while self.0.events_end.read().bits() == 0 {}
        self.0.events_end.reset();

        // Will only occur if more than one channel has been enabled
        if self.0.result.amount.read().bits() != 1 {
            return Err(nb::Error::Other(()));
        }

        // Second fence to prevent optimizations creating issues with the EasyDMA-modified `val`
        compiler_fence(SeqCst);

        Ok(val)
    }
}

macro_rules! channel_mappings {
    ($($n:expr => $pin:path),*) => {
        $(
            impl Channel<Saadc> for $pin {
                type ID = u8;

                fn channel() -> <Self as embedded_hal::adc::Channel<Saadc>>::ID {
                    $n
                }
            }
        )*
    };
}

channel_mappings! {
    0 => crate::gpio::p0::P0_02<Input<Floating>>,
    1 => crate::gpio::p0::P0_03<Input<Floating>>,
    2 => crate::gpio::p0::P0_04<Input<Floating>>,
    3 => crate::gpio::p0::P0_05<Input<Floating>>,
    4 => crate::gpio::p0::P0_28<Input<Floating>>,
    5 => crate::gpio::p0::P0_29<Input<Floating>>,
    6 => crate::gpio::p0::P0_30<Input<Floating>>,
    7 => crate::gpio::p0::P0_31<Input<Floating>>
}
