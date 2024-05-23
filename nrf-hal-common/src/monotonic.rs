/*!
Implements the [Monotonic] trait for the TIMERs and the RTCs.

## Preface

The links to the datasheets in the documentation are specific for the nrf52840, however the register
interfaces should be the same for all the nRF51, nRF52 and nRF91 families of microcontrollers.

A simple example using the timer/rtc can be found under the nrf-hal
[examples](https://github.com/nrf-rs/nrf-hal/tree/master/examples/monotonic-blinky).

## RTC - Real-time counter

The [`Rtc`](crate::rtc::Rtc) [§6.22](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.7.pdf)
has a 12-bit wide prescaler. This allows for prescalers ranging from 0 to 4095. With the prescaler,
one can calculate the frequency by:

`f_RTC [KHz] = 32.768 / (PRESCALER + 1)`

Since the RTC will only accept frequencies that have a valid prescaler. It is not always possible to
get the exact desired frequency, however, it is possible to calculate a prescaler which results in a
frequency close to the desired one. This prescaler can be calculated by:

`f_RTC = 32.768 / (round((32.768/f_desired) - 1)+1)`

When using the RTC, make sure that the low-frequency clock source (lfclk) is started. Otherwise, the
RTC will not work.

<Strong> Example (RTC): </Strong>
```ignore
// RTC0 with a frequency of 32 768 Hz
type MyMono = MonotonicRtc<RTC0, 32_768>;

// Make sure lfclk is started
let clocks = hal::clocks::Clocks::new(cx.device.CLOCK);
let clocks = clocks.start_lfclk();

let mono = MyMono::new(cx.device.RTC0, &clocks).unwrap();
```

## TIMER

The [`Timer`](crate::timer::Timer) [§6.30](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.7.pdf)
has 2 different clock sources that can drive it, one 16MHz clock that is used when the timer
frequency is higher than 1MHz and a 1MHz clock is used otherwise. The 1MHz clock consumes less power
than the 16MHz clock source, so for low applications, it could be beneficial to use a frequency at
or below 1MHz. For a list of all valid frequencies please see the [`MonotonicTimer`] documentation.

The timer frequency is given by the formula:

`f_TIMER = 16 MHz / (2^PRESCALER)`

Where the prescaler is a 4-bit integer.

### Example (Timer):

```ignore
// TIMER0 with a frequency of 16 000 000 Hz
type MyMono = MonotonicTimer<TIMER0, 16_000_000>;
let mono = MyMono::new(cx.device.TIMER0);
```

## Overflow

The TIMERs are configured to use a 32-bit wide counter, this means that the time until overflow is
given by the following formula: `T_overflow = 2^32/freq`. Therefore the time until overflow for the
maximum frequency (16MHz) is `2^32/(16*10^6) = 268` seconds, using a 1MHz TIMER yields time till
overflow `2^32/(10^6) = 4295` seconds or 1.2 hours. For more information on overflow please see the
[`Timer`](crate::timer::Timer) documentation.

The RTC uses a 24-bit wide counter. The time to overflow can be calculated using:
`T_overflow = 2^(24+overflow_bits)/freq` Therefore, with the frequency 32.768 KHz and the overflow
counter being u8, the RTC would overflow after about 36.5 hours.
**/
use crate::clocks::{Clocks, LfOscStarted};
use core::marker::PhantomData;
pub use rtic_monotonic::Monotonic;

#[cfg(any(feature = "9160", feature = "5340-app", feature = "5340-net"))]
use crate::pac::{rtc0_ns::RegisterBlock as RtcRegBlock, RTC0_NS as RTC0, RTC1_NS as RTC1};

#[cfg(not(any(feature = "9160", feature = "5340-app", feature = "5340-net")))]
use crate::pac::{rtc0::RegisterBlock as RtcRegBlock, RTC0, RTC1};

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
use crate::pac::RTC2;

#[cfg(any(feature = "9160", feature = "5340-app", feature = "5340-net"))]
use crate::pac::{
    timer0_ns::RegisterBlock as TimerRegBlock, TIMER0_NS as TIMER0, TIMER1_NS as TIMER1,
    TIMER2_NS as TIMER2,
};

#[cfg(not(any(feature = "9160", feature = "5340-app", feature = "5340-net")))]
use crate::pac::{timer0::RegisterBlock as TimerRegBlock, TIMER0, TIMER1, TIMER2};

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
    }

    pub trait RtcInstance: Instance<RegBlock = super::RtcRegBlock> {}

    pub trait TimerInstance: Instance<RegBlock = super::TimerRegBlock> {
        /// Sets the compare value for the [`Instance`].
        fn set_compare<const IDX: usize>(compare: u32);

        /// Clears the compare event.
        fn clear_compare_flag<const IDX: usize>();

        /// Enables the comparator for this [`Instance`].
        fn enable_compare<const IDX: usize>();
    }
}

pub use sealed::{Instance, RtcInstance, TimerInstance};

/// All of the error cases for the [`MonotonicRtc`] and
/// [`MonotonicTimer`].
#[derive(Debug)]
pub enum Error {
    /// Thrown when an invalid frequency is requested from the [`MonotonicRtc`].
    ///
    /// To compute a valid frequency use the formula `f = 32_768/(prescaler + 1)`, where _prescaler_
    /// is an integer less than 4095.
    InvalidFrequency(u32),

    /// Thrown when the requested frequency for the [`MonotonicRtc`] yields a prescaler larger than
    /// 4095.
    TooLargePrescaler(u32),
}

/// A [`Monotonic`] implementation for Real Time Clocks (RTC)
///
/// This implementation allows scheduling [rtic](https://docs.rs/rtic/latest/rtic/) applications
/// using the [`Rtc`](crate::rtc::Rtc) (§6.22 in the
/// [data sheet](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.1.pdf)) peripheral. It is only
/// possible to instantiate this abstraction with frequencies using an integer prescaler between 0
/// and 4095.
pub struct MonotonicRtc<T: RtcInstance, const FREQ: u32> {
    instance: PhantomData<T>,
    overflow: u8,
}

impl<T, const FREQ: u32> MonotonicRtc<T, FREQ>
where
    T: RtcInstance,
{
    const MAX_PRESCALER: u32 = 4096;

    /// Instantiates a new [`Monotonic`](rtic_monotonic) RTC for the specified [`RtcInstance`].
    ///
    /// This function permits construction of the `MonotonicRtc` for a given frequency.
    pub fn new<H, L>(_: T, _: &Clocks<H, L, LfOscStarted>) -> Result<Self, Error> {
        let presc = Self::prescaler()?;
        unsafe { T::reg().prescaler.write(|w| w.bits(presc)) };

        Ok(Self {
            instance: PhantomData,
            overflow: 0,
        })
    }

    /// Checks if the given frequency is valid.
    const fn prescaler() -> Result<u32, Error> {
        let intermediate: u32 = 32_768 / FREQ;
        let presc: u32 = (32_768 / FREQ) - 1;

        if presc >= Self::MAX_PRESCALER {
            return Err(Error::TooLargePrescaler(FREQ));
        }

        match 32_768 / intermediate == FREQ && presc < Self::MAX_PRESCALER {
            true => Ok(presc),
            _ => Err(Error::InvalidFrequency(FREQ)),
        }
    }
}

impl<T: RtcInstance, const FREQ: u32> Monotonic for MonotonicRtc<T, FREQ> {
    type Instant = fugit::TimerInstantU32<FREQ>;
    type Duration = fugit::TimerDurationU32<FREQ>;

    // Since we are using extended counter we need to keep the
    // interrupts enabled for the rtc.
    const DISABLE_INTERRUPT_ON_EMPTY_QUEUE: bool = false;

    fn now(&mut self) -> Self::Instant {
        let rtcreg = T::reg();
        let cnt = rtcreg.counter.read().bits();

        let ovf = (if rtcreg.events_ovrflw.read().bits() == 1 {
            self.overflow.wrapping_add(1)
        } else {
            self.overflow
        }) as u32;

        fugit::TimerInstantU32::<FREQ>::from_ticks((ovf << 24) | cnt)
    }

    fn set_compare(&mut self, instant: Self::Instant) {
        // Based on @korken89 implementation
        // https://gist.github.com/korken89/fe94a475726414dd1bce031c76adc3dd
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

/// A [`Monotonic`] timer implementation
///
/// This implementation allows scheduling [rtic](https://docs.rs/rtic/latest/rtic/) applications
/// using the [`Timer`](crate::timer::Timer) (§6.30 in the
/// [data sheet](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.1.pdf)) peripheral. It is only
/// possible to instantiate this abstraction for the following frequencies since they are the only
/// ones that generate valid prescaler values:
///
///| frequency \[Hz\]            | time until overflow                          | source clock frequency   |
///|-----------------------------|----------------------------------------------|--------------------------|
///| <center> 16000000 </center> | <center> 4 min 28 seconds </center>          | <center> 16MHz </center> |
///| <center> 8000000 </center>  | <center> 8 min 56 seconds </center>          | <center> 16MHz </center> |
///| <center> 4000000 </center>  | <center> 17 min 53 seconds </center>         | <center> 16MHz </center> |
///| <center> 1000000 </center>  | <center> 1 hour 11 min 34 seconds </center>  | <center> 1MHz </center>  |
///| <center> 500000 </center>   | <center> 2 hours 23 min 9 seconds </center>  | <center> 1MHz </center>  |
///| <center> 250000 </center>   | <center> 4 hours 46 min 19 seconds </center> | <center> 1MHz </center>  |
///| <center> 125000 </center>   | <center> 9 hours 32 min 39 seconds </center> | <center> 1MHz </center>  |
///| <center> 62500 </center>    | <center> 19 hours 5 min 19 seconds </center> | <center> 1MHz </center>  |
pub struct MonotonicTimer<T: TimerInstance, const FREQ: u32> {
    instance: PhantomData<T>,
}

impl<T: TimerInstance, const FREQ: u32> MonotonicTimer<T, FREQ> {
    pub fn internal_new<const PRESC: u8>() -> Self {
        let reg = T::reg();
        reg.prescaler
            .write(|w| unsafe { w.prescaler().bits(PRESC) });
        reg.bitmode.write(|w| w.bitmode()._32bit());
        reg.mode.write(|w| w.mode().timer());
        Self {
            instance: PhantomData,
        }
    }
}

impl<T: TimerInstance, const FREQ: u32> Monotonic for MonotonicTimer<T, FREQ> {
    type Instant = fugit::TimerInstantU32<FREQ>;
    type Duration = fugit::TimerDurationU32<FREQ>;

    fn now(&mut self) -> Self::Instant {
        let reg: &TimerRegBlock = T::reg();
        T::enable_compare::<1>();

        let ticks = reg.cc[1].read().bits();
        fugit::TimerInstantU32::<FREQ>::from_ticks(ticks.into())
    }

    fn set_compare(&mut self, instant: Self::Instant) {
        T::set_compare::<2>(instant.duration_since_epoch().ticks())
    }

    fn clear_compare_flag(&mut self) {
        T::clear_compare_flag::<2>();
    }

    fn zero() -> Self::Instant {
        Self::Instant::from_ticks(0)
    }

    unsafe fn reset(&mut self) {
        let reg = T::reg();
        reg.intenset.write(|w| w.compare2().set());
        reg.tasks_clear.write(|w| w.bits(1));
        reg.tasks_start.write(|w| w.bits(1));
    }
}

macro_rules! impl_instance {
    (TimerRegBlock,$peripheral:ident) => {
        impl TimerInstance for $peripheral {
            fn set_compare<const IDX:usize>(compare:u32){
                #[cfg(feature = "51")]
                Self::reg().cc[IDX].write(|w| unsafe{w.bits(compare)});

                #[cfg(not(feature = "51"))]
                Self::reg().cc[IDX].write(|w| w.cc().variant(compare));
            }

            fn clear_compare_flag<const IDX:usize>(){
                // We clear the top bits manually.
                #[cfg(any(feature = "52832", feature = "51"))]
                Self::reg().events_compare[2].write(|w| unsafe { w.bits(0) });

                #[cfg(not(any(feature = "52832", feature = "51")))]
                Self::reg().events_compare[2].write(|w| w.events_compare().clear_bit());

            }

            fn enable_compare<const IDX:usize>() {
                // Bit idx 31 is the same as enabling tasks_capture.
                #[cfg(any(feature = "52832", feature = "51"))]
                Self::reg().tasks_capture[IDX].write(|w| unsafe { w.bits(1 << 31) });

                #[cfg(not(any(feature = "52832", feature = "51")))]
                Self::reg().reg.tasks_capture[IDX].write(|w| w.tasks_capture().set_bit());

            }
        }
    };
    (RtcRegBlock,$peripheral:ident) => {
        impl RtcInstance for $peripheral {}
    };
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
                impl_instance!($reg,$peripheral);
            )+
        )+
    };
}

macro_rules! freq_gate {
    (
        $(
            $freq:literal,$presc:literal
        )+
    ) => (
        $(
            impl<T:TimerInstance> MonotonicTimer<T,$freq>
            {
                /// Instantiates a new [`Monotonic`] enabled
                /// timer for the specified [`TimerInstance`]
                pub fn new(_: T) -> Self {
                    Self::internal_new::<$presc>()
                }
            }
        )+
    )
}

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
impl_instance! {
    TimerRegBlock : {
        TIMER0 TIMER1 TIMER2
        TIMER3
        TIMER4
    }
    RtcRegBlock : {
        RTC0 RTC1
        RTC2
    }
}

#[cfg(not(any(feature = "52832", feature = "52833", feature = "52840")))]
impl_instance! {
    TimerRegBlock : {
        TIMER0 TIMER1 TIMER2
    }
    RtcRegBlock : {
        RTC0 RTC1
    }
}

freq_gate! {
    16_000_000,0
    8_000_000,1
    4_000_000,2
    2_000_000,3
    1_000_000,4
    500_000,5
    250_000,6
    125_000,7
    62_500,8
}
