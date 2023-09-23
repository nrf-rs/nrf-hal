#[cfg(any(feature = "9160", feature = "5340-app", feature = "5340-net"))]
use crate::pac::{rtc0_ns::RegisterBlock as rtcRegBlock, RTC0_NS as RTC0, RTC1_NS as RTC1};

#[cfg(not(any(feature = "9160", feature = "5340-app", feature = "5340-net")))]
use crate::pac::{rtc0::RegisterBlock as rtcRegBlock, RTC0, RTC1};

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
use crate::pac::RTC2;

use core::marker::PhantomData;
use rtic_monotonic::Monotonic;

pub trait InstanceRtc {
    type RegBlock;
    fn reg<'a>() -> &'a Self::RegBlock;
}

pub struct MonotonicRtc<T: InstanceRtc, const FREQ: u32> {
    instance: PhantomData<T>,
    overflow: u8,
}

impl<T: InstanceRtc, const FREQ: u32> MonotonicRtc<T, FREQ> {
    fn new() {
        todo!()
    }

    fn reg<'a>() -> &'a rtcRegBlock {
        unsafe { &*RTC0::ptr() }
    }
}

impl<T: InstanceRtc, const FREQ: u32> Monotonic for MonotonicRtc<T, FREQ> {
    type Instant = fugit::TimerInstantU32<FREQ>;
    type Duration = fugit::TimerDurationU32<FREQ>;

    fn now(&mut self) -> Self::Instant {
        let rtc = Self::reg();
        let cnt = rtc.counter.read().bits();

        let ovf = (if rtc.events_ovrflw.read().bits() == 1 {
            self.overflow.wrapping_add(1)
        } else {
            self.overflow
        }) as u32;

        Self::Instant::from_ticks((ovf << 24) | cnt)
    }

    fn set_compare(&mut self, instant: Self::Instant) {
        todo!()
    }

    fn clear_compare_flag(&mut self) {
        let rtc = Self::reg();
        unsafe {
            rtc.events_compare[0].write(|w| w.bits(0));
        }
    }

    fn zero() -> Self::Instant {
        Self::Instant::from_ticks(0)
    }

    unsafe fn reset(&mut self) {
        let rtc = Self::reg();
        rtc.intenset.write(|w| w.compare0().set().ovrflw().set());
        rtc.evtenset.write(|w| w.compare0().set().ovrflw().set());

        rtc.tasks_clear.write(|w| w.bits(1));
        rtc.tasks_start.write(|w| w.bits(1));
    }
}
