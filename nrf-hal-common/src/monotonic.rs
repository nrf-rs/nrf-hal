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
        type RegBlock;
        /// Returns a pointer to the underlying register block
        ///
        /// Allows modification of the registers at a type level rather than
        /// by storing the [`Instance`] at run-time.
        fn reg<'a>() -> &'a Self::RegBlock;
        const DISABLE_INTERRUPT_ON_EMPTY_QUEUE: bool = true;
    }
    pub trait RtcInstance: Instance<RegBlock = super::RtcRegBlock> {}
    pub trait TimerInstance: Instance<RegBlock = super::TimerRegBlock0> {}
}
pub use sealed::{Instance, RtcInstance, TimerInstance};

impl<T: Instance<RegBlock = TimerRegBlock0>, const FREQ: u32> Monotonic
    for MonotonicTimer<T, FREQ>
{
    type Instant = fugit::TimerInstantU32<FREQ>;
    type Duration = fugit::TimerDurationU32<FREQ>;
    fn now(&mut self) -> Self::Instant {
        let reg: &TimerRegBlock0 = T::reg();
        reg.tasks_capture[1].write(|w| w.tasks_capture().set_bit());
        let ticks = reg.cc[1].read().bits();
        fugit::TimerInstantU32::<FREQ>::from_ticks(ticks.into())
    }

    fn set_compare(&mut self, instant: Self::Instant) {
        T::reg().cc[2].write(|w| w.cc().variant(instant.duration_since_epoch().ticks()));
    }

    fn clear_compare_flag(&mut self) {
        T::reg().events_compare[2].write(|w| w.events_compare().clear_bit());
    }

    fn zero() -> Self::Instant {
        todo!()
    }
    unsafe fn reset(&mut self) {
        let reg = T::reg();
        reg.intenset.write(|w| w.compare2().set());
        reg.tasks_clear.write(|w| w.bits(1));
        reg.tasks_start.write(|w| w.bits(1));
    }
}

pub struct MonotonicRtc<
    T: Instance<RegBlock = RtcRegBlock>,
    const FREQ: u32,
    O: core::default::Default,
> {
    instance: PhantomData<T>,
    overflow: O,
}

impl<T, const FREQ: u32, O: core::default::Default> MonotonicRtc<T, FREQ, O>
where
    T: Instance<RegBlock = RtcRegBlock>,
{
    /// Instantiates a new [`Monotonic`](rtic_monotonic)
    /// rtc for the specified [`RtcInstance`].
    ///
    /// This function permits construction of the
    pub fn new(_: T) -> Self {
        unsafe { T::reg().prescaler.write(|w| w.bits(Self::presc().unwrap())) };
        Self {
            instance: PhantomData,
            overflow: O::default(),
        }
    }
    const fn presc() -> Option<u32> {
        let intermediate = { 32_768 / FREQ };
        match 32_768 / intermediate == FREQ {
            true => Some(32_768 / (FREQ + 1)),
            _ => None,
        }
    }
}

impl<T: RtcInstance, const FREQ: u32> Monotonic for MonotonicRtc<T, FREQ, u32> {
    type Instant = fugit::TimerInstantU32<FREQ>;
    type Duration = fugit::TimerDurationU32<FREQ>;
    fn now(&mut self) -> Self::Instant {
        let rtc = T::reg();
        let cnt = rtc.counter.read().bits();

        let ovf = (if rtc.events_ovrflw.read().bits() == 1 {
            self.overflow.wrapping_add(1)
        } else {
            self.overflow
        }) as u32;

        fugit::TimerInstantU32::<FREQ>::from_ticks((ovf << 24) | cnt)
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
        unsafe { T::reg().cc[0].write(|w| w.bits(val)) };
    }

    fn clear_compare_flag(&mut self) {
        unsafe {
            T::reg().events_compare[0].write(|w| w.bits(0));
        }
    }

    unsafe fn reset(&mut self) {
        let rtc = T::reg();
        rtc.intenset.write(|w| w.compare0().set().ovrflw().set());
        rtc.evtenset.write(|w| w.compare0().set().ovrflw().set());

        rtc.tasks_clear.write(|w| w.bits(1));
        rtc.tasks_start.write(|w| w.bits(1));
    }

    fn zero() -> Self::Instant {
        Self::Instant::from_ticks(0)
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
    (type_for TimerRegBlock0 : {
        $(
            $freq:literal,$presc:literal,$overflow:literal,$sck:literal
        )+
    }) => {
        paste!(
            /// A [`Monotonic`] timer implementation
            ///
            /// This implementation allows scheduling [rtic](https://docs.rs/rtic/latest/rtic/)
            /// applications using the [`Timer`](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.1.pdf#page=459) (§6.30) peripheral.
            /// It is only possible to instantiate this abstraction for the following
            /// frequencies since they are the only ones that generate valid prescaler values.
            /// ## Timer
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
            ///
            pub struct MonotonicTimer<T: Instance<RegBlock = TimerRegBlock0>, const FREQ: u32> {
                instance:PhantomData<T>,
            }
        );

    };
    (
        new_for TimerRegBlock0 : {
            $(
                $freq:literal,$presc:literal
            )+
        }
    ) => {
        paste!(
        $(
            impl<T> MonotonicTimer<T,$freq>
            where T:Instance<RegBlock = TimerRegBlock0>{
                /// Instantiates a new [`Monotonic`] enabled
                /// timer for the specified [`TimerInstance`].
                ///
                /// This function permits construction of the
                #[doc = "[`MonotonicTimer`] for `" $freq "` Hz."]
                pub fn new(_: T) -> Self {
                    let reg = T::reg();
                    reg.prescaler
                        .write(|w| unsafe { w.prescaler().bits($presc) });
                    reg.bitmode.write(|w| w.bitmode()._32bit());
                    reg.mode.write(|w| w.mode().timer());
                    Self {
                        instance: PhantomData,
                    }
                }
            }
        )+);
    };
    ($(
        $type:literal, $reg:ident: {
                $(
                    $freq:literal,$presc:literal,$overflow:literal,$sck:literal
                )+
            }
        )+
    ) => (
            $(
                freq_gate!(
                    type_for $reg : {
                        $(
                            $freq,$presc,$overflow,$sck
                        )+
                    }
                );
                freq_gate!(
                    new_for $reg:{
                        $(
                            $freq,$presc
                        )+
                    }
                );
            )+
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
    "Timer",TimerRegBlock0:{
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
