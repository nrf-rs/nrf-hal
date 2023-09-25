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
    pub fn new(instance: PhantomData<T>) -> Self {
        let rtc = Self::reg();
        unsafe { rtc.prescaler.write(|w| w.bits(0)) };

        MonotonicRtc {
            instance: instance,
            overflow: 0,
        }
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
        // Credit @kroken89 on github [https://gist.github.com/korken89/fe94a475726414dd1bce031c76adc3dd]
        let now = self.now();

        const MIN_TICKS_FOR_COMPARE: u32 = 3;

        // Since the timer may or may not overflow based on the requested compare val, we check
        // how many ticks are left.
        let val = match instant.checked_duration_since(now) {
            Some(x) if x.ticks() <= 0xffffff && x.ticks() > MIN_TICKS_FOR_COMPARE => {
                instant.duration_since_epoch().ticks() & 0xffffff
            }
            Some(x) => {
                (instant.duration_since_epoch().ticks() + (MIN_TICKS_FOR_COMPARE - x.ticks()))
                    & 0xffffff
            }
            _ => 0, // Will overflow or in the past, set the same value as after overflow to not get extra interrupts
        } as u32;
        let rtc = Self::reg();
        unsafe { rtc.cc[0].write(|w| w.bits(val)) };
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

// impl with regblocks
// impl InstanceRtc for RTC0 {}
// impl InstanceRtc for RTC1 {}
// impl InstanceRtc for RTC2 {}
