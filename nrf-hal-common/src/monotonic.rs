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

#[cfg(any(feature = "9160", feature = "5340-app", feature = "5340-net"))]
use crate::pac::{
    timer0_ns::RegisterBlock as TimerRegBlock0, TIMER0_NS as TIMER0, TIMER1_NS as TIMER1,
    TIMER2_NS as TIMER2,
};

#[cfg(not(any(feature = "9160", feature = "5340-app", feature = "5340-net")))]
use crate::pac::{timer0::RegisterBlock as TimerRegBlock0, TIMER0, TIMER1, TIMER2};

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
use crate::pac::{TIMER3, TIMER4};

use core::marker::PhantomData;
use paste::paste;
pub use rtic_monotonic::Monotonic;

/// Hides intermediate traits from end users.
mod sealed {
    /// A trait that ensures register access for the [`pac`](`crate::pac`)
    /// abstractions
    pub trait Instance {
        /// The type of the underlying register block
        type RegBlock;
        /// Returns a pointer to the underlying register block
        ///
        /// Allows modification of the registers at a type level rather than
        /// by storing the [`Instance`] at run-time.
        fn reg<'a>() -> &'a Self::RegBlock;
    }

    pub trait RateMonotonic<Instant> {
        fn _new(presc: u8) -> Self;
        fn _now(&mut self) -> Instant;
        fn _set_compare(&mut self, instant: Instant);
        fn _clear_compare_flag(&mut self);
        unsafe fn _reset(&mut self);
    }
    /// A marker trait denoting
    /// that the specified [`pac`](crate::pac)
    /// peripheral is a valid timer.
    pub trait TimerInstance: Instance<RegBlock = super::TimerRegBlock0> {}
}
use sealed::{Instance, RateMonotonic, TimerInstance};

// Public implementation for any peripheral that implements the
// sealed RateMonotonic trait.
impl<T: Instance, const FREQ: u32> Monotonic for MonotonicTimer<T, FREQ>
where
    MonotonicTimer<T, FREQ>: sealed::RateMonotonic<fugit::TimerInstantU32<FREQ>>,
{
    type Instant = fugit::TimerInstantU32<FREQ>;
    type Duration = fugit::TimerDurationU32<FREQ>;
    fn now(&mut self) -> Self::Instant {
        self._now()
    }

    fn set_compare(&mut self, instant: Self::Instant) {
        self._set_compare(instant);
    }

    fn clear_compare_flag(&mut self) {
        self._clear_compare_flag();
    }

    fn zero() -> Self::Instant {
        Self::Instant::from_ticks(0)
    }

    unsafe fn reset(&mut self) {
        self._reset();
    }
}

// Private implementation of monotonic for a generic timer
impl<T: TimerInstance, const FREQ: u32> RateMonotonic<fugit::TimerInstantU32<FREQ>>
    for MonotonicTimer<T, FREQ>
{
    fn _new(presc: u8) -> Self {
        let reg = T::reg();
        reg.prescaler
            .write(|w| unsafe { w.prescaler().bits(presc) });
        reg.bitmode.write(|w| w.bitmode()._32bit());
        reg.mode.write(|w| w.mode().timer());
        Self {
            instance: PhantomData,
        }
    }
    fn _now(&mut self) -> fugit::TimerInstantU32<FREQ> {
        let reg = T::reg();
        reg.tasks_capture[1].write(|w| w.tasks_capture().set_bit());
        let ticks = reg.cc[1].read().bits();
        fugit::TimerInstantU32::<FREQ>::from_ticks(ticks.into())
    }

    fn _set_compare(&mut self, instant: fugit::TimerInstantU32<FREQ>) {
        T::reg().cc[2].write(|w| w.cc().variant(instant.duration_since_epoch().ticks()));
    }

    fn _clear_compare_flag(&mut self) {
        T::reg().events_compare[2].write(|w| w.events_compare().clear_bit());
    }

    unsafe fn _reset(&mut self) {
        let reg: &TimerRegBlock0 = T::reg();
        reg.intenset.write(|w| w.compare2().set());
        reg.tasks_clear.write(|w| w.bits(1));
        reg.tasks_start.write(|w| w.bits(1));
    }
}

macro_rules! impl_instance {
    (
        $(
            $instance:ident with $reg:ident : {
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
                        unsafe { &*Self::ptr().cast() }
                    }
                }
                $( #[$feature_gate] )?
                impl $instance for $peripheral{}
            )+
        )+
    };
}

macro_rules! freq_gate {
    (
        $(
            $type:literal,$instant_type:ident : {
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
            )+
            pub struct MonotonicTimer<T: Instance, const FREQ: u32> {
                instance: PhantomData<T>,
            }
            $(
                $(
                    impl<T: $instant_type>  MonotonicTimer<T,$freq> {
                        /// Instantiates a new [`Monotonic`](rtic_monotonic)
                        /// timer for the specified [`TimerInstance`].
                        ///
                        /// This function permits construction of the
                        #[doc = "timer for `" $freq "` Hz derived from a " $sck " clock c a prescaler of " $presc "."]
                        /// This timer will overflow after
                        #[doc = $overflow "."]
                        pub fn new(_: T) -> Self {
                            Self::_new(($presc as u8))
                        }
                    }
                )+
            )+
        );
    )
}

impl_instance!(
    TimerInstance with TimerRegBlock0 : {
        TIMER0 TIMER1 TIMER2
        #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
        TIMER3
        #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
        TIMER4
    }
);

freq_gate! {
    "Timer",TimerInstance : {
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
}
