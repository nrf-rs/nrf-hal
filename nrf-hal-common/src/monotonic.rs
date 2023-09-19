#[cfg(any(feature = "9160", feature = "5340-app", feature = "5340-net"))]
use crate::pac::{
    timer0_ns::{
        RegisterBlock as RegBlock0, EVENTS_COMPARE, TASKS_CAPTURE, TASKS_CLEAR, TASKS_COUNT,
        TASKS_START, TASKS_STOP,
    },
    Interrupt, TIMER0_NS as TIMER0, TIMER1_NS as TIMER1, TIMER2_NS as TIMER2,
};

#[cfg(not(any(feature = "9160", feature = "5340-app", feature = "5340-net")))]
use crate::pac::{
    timer0::{
        RegisterBlock as RegBlock0, EVENTS_COMPARE, TASKS_CAPTURE, TASKS_CLEAR, TASKS_COUNT,
        TASKS_START, TASKS_STOP,
    },
    Interrupt, TIMER0, TIMER1, TIMER2,
};
use cast::u32;
use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    prelude::*,
    timer,
};
use nb::{self, block};
use void::{unreachable, Void};

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
use crate::pac::{TIMER3, TIMER4};

// The 832 and 840 expose TIMER3 and TIMER for as timer3::RegisterBlock...
#[cfg(any(feature = "52832", feature = "52840"))]
use crate::pac::timer3::{
    RegisterBlock as RegBlock3, EVENTS_COMPARE as EventsCompare3, TASKS_CAPTURE as TasksCapture3,
};

// ...but the 833 exposes them as timer0::RegisterBlock. This might be a bug
// in the PAC, and could be fixed later. For now, it is equivalent anyway.
#[cfg(feature = "52833")]
use crate::pac::timer0::{
    RegisterBlock as RegBlock3, EVENTS_COMPARE as EventsCompare3, TASKS_CAPTURE as TasksCapture3,
};

use core::marker::PhantomData;
use rtic_monotonic::Monotonic;

pub trait RegisterAccess<RegBlock> {
    fn reg<'a>() -> &'a RegBlock;
}

trait TimerRegister: RegisterAccess<RegBlock0> {}

pub trait Instantiatable<RegBlock> {
    fn new<Instance: RegisterAccess<RegBlock>>(instance: Instance) -> Self;
}

struct MonotonicTimer<Timer: TimerRegister> {
    instance: PhantomData<Timer>,
}

impl<Timer: TimerRegister> Instantiatable<RegBlock0> for MonotonicTimer<Timer> {
    fn new<Instance: RegisterAccess<RegBlock0>>(_: Instance) -> Self {
        let reg = Timer::reg();
        reg.prescaler.write(|w| unsafe { w.prescaler().bits(4) });
        reg.bitmode.write(|w| w.bitmode()._32bit());
        Self {
            instance: PhantomData,
        }
    }
}

impl<Timer: TimerRegister> Monotonic for MonotonicTimer<Timer> {
    type Instant = fugit::TimerInstantU32<1_000_000>;
    type Duration = fugit::TimerDurationU32<1_000_000>;

    fn now(&mut self) -> Self::Instant {
        let reg = Timer::reg();
        reg.tasks_capture[0].write(|w| w.tasks_capture().set_bit());
        let ticks = reg.cc[0].read().cc().bits();
        let now = fugit::TimerInstantU32::from_ticks(ticks);
        return now;
    }

    fn set_compare(&mut self, instant: Self::Instant) {
        Timer::reg().cc[0].write(|w| w.cc().variant(instant.duration_since_epoch().ticks()));
    }

    fn clear_compare_flag(&mut self) {
        Timer::reg().events_compare[0].write(|w| w.events_compare().clear_bit());
    }

    fn zero() -> Self::Instant {
        Self::Instant::from_ticks(0)
    }

    unsafe fn reset(&mut self) {
        let reg = Timer::reg();
        reg.intenset.write(|w| w.compare0().set_bit());
        reg.tasks_clear.write(|w| w.tasks_clear().set_bit());
        reg.tasks_start.write(|w| w.tasks_start().set_bit());
    }
}
