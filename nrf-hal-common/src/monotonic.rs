#[cfg(any(feature = "9160", feature = "5340-app", feature = "5340-net"))]
use crate::pac::{
    timer0_ns::RegisterBlock as RegBlock0, TIMER0_NS as TIMER0, TIMER1_NS as TIMER1,
    TIMER2_NS as TIMER2,
};

#[cfg(not(any(feature = "9160", feature = "5340-app", feature = "5340-net")))]
use crate::pac::{timer0::RegisterBlock as RegBlock0, TIMER0, TIMER1, TIMER2};

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
use crate::pac::{TIMER3, TIMER4};

// The 832 and 840 expose TIMER3 and TIMER for as timer3::RegisterBlock...
#[cfg(any(feature = "52832", feature = "52840"))]
use crate::pac::timer3::RegisterBlock as RegBlock3;

// ...but the 833 exposes them as timer0::RegisterBlock. This might be a bug
// in the PAC, and could be fixed later. For now, it is equivalent anyway.
#[cfg(feature = "52833")]
use crate::pac::timer0::RegisterBlock as RegBlock3;

use core::{convert::TryInto, marker::PhantomData};
use rtic_monotonic::Monotonic;

pub trait TimerInstance {
    type RegBlock;
    fn reg<'a>() -> &'a Self::RegBlock;
    fn init(presc: u8);
}

pub struct MonotonicTimer<T: TimerInstance, const FREQ: u32> {
    instance: PhantomData<T>,
}

impl<T: TimerInstance, const FREQ: u32> MonotonicTimer<T, FREQ> {
    fn _new(_: T, presc: u8) -> Self {
        T::init(presc);
        Self {
            instance: PhantomData,
        }
    }
}

macro_rules! impl_timer {
    (
        $(
            $(#[$feature_gate:meta])?
            $timer:ident -> $regblock:ident
        )+
    ) => {
        $(

            $( #[$feature_gate] )?
            impl TimerInstance for $timer{
                type RegBlock = $regblock;
                fn reg<'a>() -> &'a $regblock {
                    unsafe{ & *$timer::ptr() }
                }
                fn init(presc:u8){
                    let reg = Self::reg();
                    reg.prescaler
                        .write(|w| unsafe { w.prescaler().bits(presc) });
                    reg.bitmode.write(|w| w.bitmode()._32bit());
                }
            }

            $( #[$feature_gate] )?
            impl<const FREQ:u32> MonotonicTimer<$timer,FREQ>{
                fn reg<'a>() -> &'a $regblock {
                    $timer::reg()
                }
                fn _now(&mut self) -> fugit::TimerInstantU32<FREQ> {
                    let reg = Self::reg();
                    reg.tasks_capture[1].write(|w| w.tasks_capture().set_bit());
                    let ticks = reg.cc[1].read().bits();
                    fugit::TimerInstantU32::<FREQ>::from_ticks(ticks.into())
                }

                fn _set_compare(&mut self, instant: fugit::TimerInstantU32<FREQ>) {
                    Self::reg().cc[0].write(|w| {
                        w.cc()
                            .variant(instant.duration_since_epoch().ticks().try_into().unwrap())
                    });
                }

                fn _clear_compare_flag(&mut self) {
                    Self::reg().events_compare[0].write(|w| w.events_compare().clear_bit());
                }

                unsafe fn _reset(&mut self) {
                    let reg = Self::reg();
                    reg.intenset.write(|w| w.compare0().set_bit());
                    reg.tasks_clear.write(|w| w.tasks_clear().set_bit());
                    reg.tasks_start.write(|w| w.tasks_start().set_bit());
                }
            }
            $( #[$feature_gate] )?
            impl<const FREQ:u32> Monotonic for MonotonicTimer<$timer,FREQ>{
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

        )+
    };
}

impl_timer!(
    TIMER0 -> RegBlock0
    TIMER1 -> RegBlock0
    TIMER2 -> RegBlock0
    #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
    TIMER3 -> RegBlock3
    #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
    TIMER4 -> RegBlock3

);

macro_rules! freq_gate {
    (
    $(
        $freq:literal,$presc:literal,
    )+
    ) => (
        $(
            impl<T: TimerInstance>   MonotonicTimer<T,$freq> {
                pub fn new(instance: T) -> Self {
                    Self::_new(instance,($presc as u8))
                }
            }
        )+
    )
}
freq_gate! {
    16_000_000,0,
    8_000_000,1,
    4_000_000,2,
    2_000_000,3,
    1_000_000,4,
    500_000,5,
    250_000,6,
    125_000,7,
    62_500,8,
}
