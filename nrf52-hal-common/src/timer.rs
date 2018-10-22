//! HAL interface to the TIMER peripheral
//!
//! See product specification, chapter 24.


use core::ops::Deref;

use nb;
use target::{
    timer0,
    Interrupt,
    TIMER0,
    TIMER1,
    TIMER2,
    TIMER3,
    TIMER4,
};
use void::Void;


pub trait TimerExt : Deref<Target=timer0::RegisterBlock> + Sized {
    // The interrupt that belongs to this timer instance
    const INTERRUPT: Interrupt;

    fn constrain(self) -> Timer<Self>;
}

macro_rules! impl_timer_ext {
    ($($timer:tt,)*) => {
        $(
            impl TimerExt for $timer {
                const INTERRUPT: Interrupt = Interrupt::$timer;

                fn constrain(self) -> Timer<Self> {
                    Timer::new(self)
                }
            }
        )*
    }
}

impl_timer_ext!(
    TIMER0,
    TIMER1,
    TIMER2,
    TIMER3,
    TIMER4,
);


/// Interface to a TIMER instance
///
/// Right now, this is a very basic interface. The timer will always be
/// hardcoded to a frequency of 1 MHz and 32 bits accuracy.
pub struct Timer<T>(T);

impl<T> Timer<T> where T: TimerExt {
    fn new(timer: T) -> Self {
        timer.shorts.write(|w|
            w
                .compare0_clear().enabled()
                .compare0_stop().enabled()
        );
        timer.prescaler.write(|w|
            unsafe { w.prescaler().bits(4) } // 1 MHz
        );
        timer.bitmode.write(|w|
            w.bitmode()._32bit()
        );

        Timer(timer)
    }

    /// Return the raw interface to the underlying timer peripheral
    pub fn free(self) -> T {
        self.0
    }

    /// Start the timer
    ///
    /// The timer will run for the given number of cycles, then it will stop and
    /// reset.
    pub fn start(&mut self, cycles: u32) {
        // Configure timer to trigger EVENTS_COMPARE when given number of cycles
        // is reached.
        self.0.cc[0].write(|w|
            // The timer mode was set to 32 bits above, so all possible values
            // of `cycles` are valid.
            unsafe { w.cc().bits(cycles) }
        );

        // Clear the counter value
        self.0.tasks_clear.write(|w|
            unsafe { w.bits(1) }
        );

        // Start the timer
        self.0.tasks_start.write(|w|
            unsafe { w.bits(1) }
        );
    }

    /// Wait for the timer to stop
    ///
    /// Will return `Err(nb::Error::WouldBlock)` while the timer is still
    /// running. Once the timer reached the number of cycles given in the
    /// `start` method, it will return `Ok(())`.
    ///
    /// To block until the timer has stopped, use the `block!` macro from the
    /// `nb` crate. Please refer to the documentation of `nb` for other options.
    pub fn wait(&mut self) -> nb::Result<(), Void> {
        if self.0.events_compare[0].read().bits() == 0 {
            // EVENTS_COMPARE has not been triggered yet
            return Err(nb::Error::WouldBlock);
        }

        // Reset the event, otherwise it will always read `1` from now on.
        self.0.events_compare[0].write(|w| w);

        Ok(())
    }
}
