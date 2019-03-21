use crate::{
    gpio::{Floating, Input},
    target::{saadc, SAADC},
};
use core::{
    ops::Deref,
    sync::atomic::{compiler_fence, Ordering::SeqCst},
};
use embedded_hal::adc::{Channel, OneShot};

// Only 1 channel is allowed right now, a discussion needs to be had as to how
// multiple channels should work (See "scan mode" in the datasheet)
// Issue: https://github.com/nrf-rs/nrf52-hal/issues/82

pub trait SaadcExt: Deref<Target = saadc::RegisterBlock> + Sized {
    fn constrain(self) -> Saadc;
}

impl SaadcExt for SAADC {
    fn constrain(self) -> Saadc {
        Saadc::new(self)
    }
}

pub struct Saadc(SAADC);

impl Saadc {
    pub fn new(saadc: SAADC) -> Self {
        saadc.enable.write(|w| w.enable().enabled());
        saadc.resolution.write(|w| w.val()._12bit());
        saadc.oversample.write(|w| w.oversample().bypass());
        saadc.samplerate.write(|w| w.mode().task());

        saadc.ch[0].config.write(|w| {
            w.refsel()
                .internal()
                .gain()
                .gain1_6()
                .tacq()
                ._20us()
                .mode()
                .se()
                .resp()
                .bypass()
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
            // This can never happen as if another pin was used, there would be a compile time error
            _ => {
                return Err(nb::Error::Other(()));
            }
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
