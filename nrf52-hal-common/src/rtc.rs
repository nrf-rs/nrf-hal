use crate::target::{
    rtc0,
    RTC0,
    RTC1,
};
#[cfg(not(feature = "52810"))]
use crate::target::{
    RTC2,
};
use core::ops::Deref;

pub struct Stopped;
pub struct Started;

pub struct Rtc<T, M> {
    periph: T,
    _mode: M,
}

pub trait RtcExt : Deref<Target=rtc0::RegisterBlock> + Sized {
    fn constrain(self) -> Rtc<Self, Stopped>;
}

macro_rules! impl_rtc_ext {
    ($($rtc:ty,)*) => {
        $(
            impl RtcExt for $rtc {
                fn constrain(self) -> Rtc<$rtc, Stopped> {
                    Rtc {
                        periph: self,
                        _mode: Stopped,
                    }
                }
            }
        )*
    }
}

impl_rtc_ext!(
    RTC0,
    RTC1,
);

#[cfg(not(feature = "52810"))]
impl_rtc_ext!(
    RTC2,
);

pub enum RtcInterrupt {
    Tick,
    Overflow,
    Compare0,
    Compare1,
    Compare2,
    Compare3,
}

pub enum RtcCompareReg {
    Compare0,
    Compare1,
    Compare2,
    Compare3,
}

impl<T, M> Rtc<T, M> where T: RtcExt {
    pub fn enable_counter(self) -> Rtc<T, Started> {
        unsafe {
            self.periph.tasks_start.write(|w| w.bits(1));
        }
        Rtc {
            periph: self.periph,
            _mode: Started,
        }
    }

    pub fn disable_counter(self) -> Rtc<T, Stopped> {
        unsafe {
            self.periph.tasks_stop.write(|w| w.bits(1));
        }
        Rtc {
            periph: self.periph,
            _mode: Stopped,
        }
    }

    pub fn enable_interrupt(&mut self, int: RtcInterrupt) {
        match int {
            RtcInterrupt::Tick => self.periph.intenset.write(|w| w.tick().set()),
            RtcInterrupt::Overflow => self.periph.intenset.write(|w| w.ovrflw().set()),
            RtcInterrupt::Compare0 => self.periph.intenset.write(|w| w.compare0().set()),
            RtcInterrupt::Compare1 => self.periph.intenset.write(|w| w.compare1().set()),
            RtcInterrupt::Compare2 => self.periph.intenset.write(|w| w.compare2().set()),
            RtcInterrupt::Compare3 => self.periph.intenset.write(|w| w.compare3().set()),
        }
    }

    pub fn disable_interrupt(&mut self, int: RtcInterrupt) {
        match int {
            RtcInterrupt::Tick => self.periph.intenclr.write(|w| w.tick().clear()),
            RtcInterrupt::Overflow => self.periph.intenclr.write(|w| w.ovrflw().clear()),
            RtcInterrupt::Compare0 => self.periph.intenclr.write(|w| w.compare0().clear()),
            RtcInterrupt::Compare1 => self.periph.intenclr.write(|w| w.compare1().clear()),
            RtcInterrupt::Compare2 => self.periph.intenclr.write(|w| w.compare2().clear()),
            RtcInterrupt::Compare3 => self.periph.intenclr.write(|w| w.compare3().clear()),
        }
    }

    pub fn enable_event(&mut self, evt: RtcInterrupt) {
        match evt {
            RtcInterrupt::Tick => self.periph.evtenset.write(|w| w.tick().set()),
            RtcInterrupt::Overflow => self.periph.evtenset.write(|w| w.ovrflw().set()),
            RtcInterrupt::Compare0 => self.periph.evtenset.write(|w| w.compare0().set()),
            RtcInterrupt::Compare1 => self.periph.evtenset.write(|w| w.compare1().set()),
            RtcInterrupt::Compare2 => self.periph.evtenset.write(|w| w.compare2().set()),
            RtcInterrupt::Compare3 => self.periph.evtenset.write(|w| w.compare3().set()),
        }
    }

    pub fn disable_event(&mut self, evt: RtcInterrupt) {
        match evt {
            RtcInterrupt::Tick => self.periph.evtenclr.write(|w| w.tick().clear()),
            RtcInterrupt::Overflow => self.periph.evtenclr.write(|w| w.ovrflw().clear()),
            RtcInterrupt::Compare0 => self.periph.evtenclr.write(|w| w.compare0().clear()),
            RtcInterrupt::Compare1 => self.periph.evtenclr.write(|w| w.compare1().clear()),
            RtcInterrupt::Compare2 => self.periph.evtenclr.write(|w| w.compare2().clear()),
            RtcInterrupt::Compare3 => self.periph.evtenclr.write(|w| w.compare3().clear()),
        }
    }

    pub fn get_event_triggered(&mut self, evt: RtcInterrupt, clear_on_read: bool) -> bool {
        let mut orig = 0;
        let set_val = if clear_on_read { 0 } else { 1 };
        match evt {
            RtcInterrupt::Tick => {
                self.periph.events_tick.modify(|r, w| {
                    orig = r.bits();
                    unsafe { w.bits(set_val) }
                })
            }
            RtcInterrupt::Overflow => {
                self.periph.events_ovrflw.modify(|r, w| {
                    orig = r.bits();
                    unsafe { w.bits(set_val) }
                })
            }
            RtcInterrupt::Compare0 => {
                self.periph.events_compare[0].modify(|r, w| {
                    orig = r.bits();
                    unsafe { w.bits(set_val) }
                })
            }
            RtcInterrupt::Compare1 => {
                self.periph.events_compare[1].modify(|r, w| {
                    orig = r.bits();
                    unsafe { w.bits(set_val) }
                })
            }
            RtcInterrupt::Compare2 => {
                self.periph.events_compare[2].modify(|r, w| {
                    orig = r.bits();
                    unsafe { w.bits(set_val) }
                })
            }
            RtcInterrupt::Compare3 => {
                self.periph.events_compare[3].modify(|r, w| {
                    orig = r.bits();
                    unsafe { w.bits(set_val) }
                })
            }
        };

        orig == 1
    }

    pub fn set_compare(&mut self, reg: RtcCompareReg, val: u32) -> Result<(), Error> {
        if val >= (1 << 24) {
            return Err(Error::CompareOutOfRange);
        }

        let reg = match reg {
            RtcCompareReg::Compare0 => 0,
            RtcCompareReg::Compare1 => 1,
            RtcCompareReg::Compare2 => 2,
            RtcCompareReg::Compare3 => 3,
        };

        unsafe { self.periph.cc[reg].write(|w| w.bits(val)); }

        Ok(())
    }

    pub fn get_counter(&self) -> u32 {
        self.periph.counter.read().bits()
    }

    pub fn release(self) -> T {
        self.periph
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    PrescalerOutOfRange,
    CompareOutOfRange,
}

impl<T> Rtc<T, Stopped> where T: RtcExt {
    pub fn set_prescaler(&mut self, prescaler: u32) -> Result<(), Error> {
        if prescaler >= (1 << 12) {
            return Err(Error::PrescalerOutOfRange);
        }

        unsafe { self.periph.prescaler.write(|w| w.bits(prescaler)) };

        Ok(())
    }
}
