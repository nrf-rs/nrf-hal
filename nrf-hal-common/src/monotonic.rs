/*!
Implements the [Monotonic](rtic_monotonic::Monotonic) trait for the TIMERs and the RTCs.

<Här borde vi ha användings exempel>

< Här borde vi ha docs om RTC>

### TIMER

The [`Timer`] [§6.30](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.7.pdf#%5B%7B%22num%22%3A5455%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C85.039%2C555.923%2Cnull%5D)
has 2 different clock sources that can drive it, one 16Mhz clock that is used when
the timers frequency is higher than 1 mhz where the timers frequency is given by:

`f_TIMER = 16 MHz / (2^PRESCALER)`
Where the prescaler is a 4 bit integer.

And one 1Mhz clock source which is used when the f_TIMER is at or lower than 1Mhz.
The 1MHz clock is lower power than the 16MHz clock source, so for low applications it could be beneficial to use a
frequency at or below 1MHz. For a list of all valid frequencies please see the
[`Timer`] documentation.

### Overflow

The TIMER's are configured to use a 32 bit wide counter, this means that the time until overflow is given by the following formula:
`T_overflow = 2^32/freq`. Therefore the time until overflow for the maximum frequency (16MHz) is `2^32/(16*10^6) = 268` seconds, using a
1MHz TIMER yields time till overflow `2^32/(10^6) = 4295` seconds or 1.2 hours. For more information on overflow please see the
[`Timer`] documentation.
**/

use core::marker::PhantomData;
use paste::paste;
pub use rtic_monotonic::Monotonic;

#[cfg(any(feature = "9160", feature = "5340-app", feature = "5340-net"))]
use crate::pac::{rtc0_ns::RegisterBlock as RtcRegBlock, RTC0_NS as RTC0, RTC1_NS as RTC1};

#[cfg(not(any(feature = "9160", feature = "5340-app", feature = "5340-net")))]
use crate::pac::{rtc0::RegisterBlock as RtcRegBlock, RTC0, RTC1};

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
use crate::pac::RTC2;

#[cfg(any(feature = "9160", feature = "5340-app", feature = "5340-net"))]
use crate::pac::{
    timer0_ns::RegisterBlock as TimerRegBlock0, TIMER0_NS as TIMER0, TIMER1_NS as TIMER1,
    TIMER2_NS as TIMER2,
};

#[cfg(not(any(feature = "9160", feature = "5340-app", feature = "5340-net")))]
use crate::pac::{timer0::RegisterBlock as TimerRegBlock0, TIMER0, TIMER1, TIMER2};

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
use crate::pac::{TIMER3, TIMER4};

/// Hides intermediate traits from end users.
mod sealed {
    /// A trait that ensures register access for the [`pac`](`crate::pac`)
    /// abstractions
    pub trait Instance {
        /// The type of the underlying register block
        type RegBlock: RateMonotonic;
        /// Returns a pointer to the underlying register block
        ///
        /// Allows modification of the registers at a type level rather than
        /// by storing the [`Instance`] at run-time.
        fn reg<'a>() -> &'a Self::RegBlock;
        const DISABLE_INTERRUPT_ON_EMPTY_QUEUE: bool = true;
    }
    pub trait RateMonotonic {
        fn _configure(&self, presc: u8);
        fn _now<const FREQ: u32>(&self, overflow: &mut u8) -> fugit::TimerInstantU32<FREQ>;
        fn _set_compare<const FREQ: u32>(
            &self,
            instant: fugit::TimerInstantU32<FREQ>,
            overflow: &mut u8,
        );
        fn _clear_compare_flag(&self);
        unsafe fn _reset(&self);
    }
}
use sealed::{Instance, RateMonotonic};

// Public implementation for any peripheral that implements the
// sealed RateMonotonic trait.
impl<I: Instance, const FREQ: u32> Monotonic for MonotonicTimer<I, FREQ> {
    type Instant = fugit::TimerInstantU32<FREQ>;
    type Duration = fugit::TimerDurationU32<FREQ>;
    const DISABLE_INTERRUPT_ON_EMPTY_QUEUE: bool = I::DISABLE_INTERRUPT_ON_EMPTY_QUEUE;
    fn now(&mut self) -> Self::Instant {
        I::reg()._now(&mut self.overflow)
    }

    fn set_compare(&mut self, instant: Self::Instant) {
        I::reg()._set_compare(instant, &mut self.overflow)
    }

    fn clear_compare_flag(&mut self) {
        I::reg()._clear_compare_flag()
    }

    fn zero() -> Self::Instant {
        Self::Instant::from_ticks(0)
    }

    unsafe fn reset(&mut self) {
        I::reg()._reset()
    }
}

impl RateMonotonic for TimerRegBlock0 {
    fn _configure(&self, presc: u8) {
        let reg = self;
        reg.prescaler
            .write(|w| unsafe { w.prescaler().bits(presc) });
        reg.bitmode.write(|w| w.bitmode()._32bit());
        reg.mode.write(|w| w.mode().timer());
    }
    fn _now<const FREQ: u32>(&self, _: &mut u8) -> fugit::TimerInstantU32<FREQ> {
        let reg = self;
        reg.tasks_capture[1].write(|w| w.tasks_capture().set_bit());
        let ticks = reg.cc[1].read().bits();
        fugit::TimerInstantU32::<FREQ>::from_ticks(ticks.into())
    }

    fn _set_compare<const FREQ: u32>(&self, instant: fugit::TimerInstantU32<FREQ>, _: &mut u8) {
        self.cc[2].write(|w| w.cc().variant(instant.duration_since_epoch().ticks()));
    }

    fn _clear_compare_flag(&self) {
        self.events_compare[2].write(|w| w.events_compare().clear_bit());
    }

    unsafe fn _reset(&self) {
        let reg = self;
        reg.intenset.write(|w| w.compare2().set());
        reg.tasks_clear.write(|w| w.bits(1));
        reg.tasks_start.write(|w| w.bits(1));
    }
}

impl RateMonotonic for RtcRegBlock {
    fn _configure(&self, presc: u8) {
        unsafe { self.prescaler.write(|w| w.bits(presc as u32)) };
    }
    fn _now<const FREQ: u32>(&self, overflow: &mut u8) -> fugit::TimerInstantU32<FREQ> {
        let rtc = self;
        let cnt = rtc.counter.read().bits();

        let ovf = (if rtc.events_ovrflw.read().bits() == 1 {
            overflow.wrapping_add(1)
        } else {
            *overflow
        }) as u32;

        fugit::TimerInstantU32::<FREQ>::from_ticks((ovf << 24) | cnt)
    }

    fn _set_compare<const FREQ: u32>(
        &self,
        instant: fugit::TimerInstantU32<FREQ>,
        overflow: &mut u8,
    ) {
        // Credit @kroken89 on github [https://gist.github.com/korken89/fe94a475726414dd1bce031c76adc3dd]
        let now = self._now(overflow);

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
        let rtc = self;
        unsafe { rtc.cc[0].write(|w| w.bits(val)) };
    }

    fn _clear_compare_flag(&self) {
        unsafe {
            self.events_compare[0].write(|w| w.bits(0));
        }
    }

    unsafe fn _reset(&self) {
        let rtc = self;
        rtc.intenset.write(|w| w.compare0().set().ovrflw().set());
        rtc.evtenset.write(|w| w.compare0().set().ovrflw().set());

        rtc.tasks_clear.write(|w| w.bits(1));
        rtc.tasks_start.write(|w| w.bits(1));
    }
}

macro_rules! impl_instance {
    (
        $(
            $reg:ident : {
                $(
                    $(#[$feature_gate:meta])?
                    $peripheral:ident
                )+
            }
        )+
    ) => {
        $(
            $(
                $( #[$feature_gate] )?
                impl Instance for $peripheral {
                    type RegBlock = $reg;
                    fn reg<'a>() -> &'a Self::RegBlock {
                        // SAFETY: TIMER0 and TIMER3 register layouts are identical, except
                        // that TIMER3 has 6 CC registers, while TIMER0 has 4. There is
                        // appropriate padding to allow other operations to work correctly
                        unsafe { & *Self::ptr().cast() }
                    }
                }
            )+
        )+
    };
}

macro_rules! freq_gate {
    (
        $(
            $type:literal, $reg:ident : {
                $(
                    $freq:literal,$presc:literal,$overflow:literal,$sck:literal
                )+
            }
        )+
    ) => (
        paste!(
            /// A [`Monotonic`] timer implementation
            ///
            /// This implementation allows scheduling [rtic](https://docs.rs/rtic/latest/rtic/)
            /// applications using either the [`Timer`](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.1.pdf#page=459) (§6.30) or the
            /// [`rtc`](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.1.pdf#page=363) (§6.22) peripherals.
            /// It is only possible to instantiate this abstraction for the following
            /// frequencies since they are the only ones that generate valid prescaler values.
            $(
                    #[doc = "\n## " $type "\n"]
                    ///
                    ///<center>
                    ///
                    ///| frequency  | source clock frequency | time until overflow |
                    ///|------------|------------------|---------------------|
                    $(
                        #[doc = "| <center> " $freq "Hz </center> | <center> " $sck " </center> | <center> " $overflow " </center> |"]
                    )+
                    ///
                    ///</center>
                    /// 
            )+
            pub struct MonotonicTimer<T: Instance, const FREQ: u32> {
                instance: PhantomData<T>,
                /// Unwrapping the overflow for rtc will is allways safe.
                overflow: u8,
            }
            $(
                mod [<sealed_ $reg:lower>] {
                    use super::*;
                    $(

                        impl<T>  MonotonicTimer<T,$freq>
                        where T:Instance<RegBlock = $reg>{
                            /// Instantiates a new [`Monotonic`](rtic_monotonic)
                            /// rtc for the specified [`RtcInstance`].
                            ///
                            /// This function permits construction of the
                            #[doc = "" $type:lower " for `" $freq "` Hz derived from a " $sck " clock c a prescaler of " $presc "."]
                            /// This rtc will overflow after
                            #[doc = $overflow "."]
                            pub fn new(_: T) -> Self {
                                T::reg()._configure(($presc as u8));
                                Self {
                                    instance: super::PhantomData,
                                    overflow: 0,
                                }
                            }
                        }
                    )+
                }
                pub use [<sealed_ $reg:lower>]::*;
            )+
        );
    )
}

impl_instance!(
    TimerRegBlock0 : {
        TIMER0 TIMER1 TIMER2
        #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
        TIMER3
        #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
        TIMER4
    }
    RtcRegBlock : {
        RTC0 RTC1
        #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
        RTC2
    }
);

freq_gate! {
    "Timer",TimerRegBlock0:  {
        16_000_000,0,"4 min 28 seconds","16MHz"
        8_000_000,1,"8 min 56 seconds","16MHz"
        4_000_000,2,"17 min 53 seconds","16MHz"
        2_000_000,3,"35 min 47 seconds","16MHz"
        1_000_000,4,"1 hour 11 min 34 seconds","1MHz"
        500_000,5,"2 hours 23 min 9 seconds","1MHz"
        250_000,6,"4 hours 46 min 19 seconds","1MHz"
        125_000,7,"9 hours 32 min 39 seconds","1MHz"
        62_500,8,"19 hours 5 min 19 seconds","1MHz"
    }

    "Rtc",RtcRegBlock: {
        123,0,"4 min 28 seconds","16MHz" // temp data from timer
    }
    // TODO Add frequencies
}
