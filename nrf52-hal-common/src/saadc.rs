//use crate::gpio::{Floating, Input, Pin};
use crate::gpio::{Floating, Input};
use crate::target::{saadc, SAADC};
use core::ops::Deref;
use embedded_hal::adc::{Channel, OneShot};

// Only 1 channel is allowed right now, a discussion needs to be had as to how
// multiple channels should work (See SCAN_MODE)

pub trait SaadcExt: Deref<Target = saadc::RegisterBlock> + Sized {
    fn constrain(self) -> Saadc;
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
        saadc.ch[0].pselp.write(|w| w.pselp().analog_input0());
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
        let mut val = 0u16;
        self.0
            .result
            .ptr
            .write(|w| unsafe { w.ptr().bits(((&mut val) as *mut _) as u32) });
        self.0
            .result
            .maxcnt
            .write(|w| unsafe { w.maxcnt().bits(1) });

        self.0.tasks_start.write(|w| unsafe { w.bits(1) });
        self.0.tasks_sample.write(|w| unsafe { w.bits(1) });

        while self.0.events_end.read().bits() == 0 {}
        self.0.events_end.reset();

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
