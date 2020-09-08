//! HAL interface to the TIMER peripheral.
//!
//! See product specification, chapter 24.

#[cfg(feature = "9160")]
use crate::pac::{
    timer0_ns::RegisterBlock as RegBlock0, Interrupt, TIMER0_NS as TIMER0, TIMER1_NS as TIMER1,
    TIMER2_NS as TIMER2,
};

#[cfg(not(feature = "9160"))]
use crate::pac::{
    timer0::RegisterBlock as RegBlock0, timer3::RegisterBlock as RegBlock3, Interrupt, TIMER0,
    TIMER1, TIMER2,
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

use core::marker::PhantomData;

pub struct OneShot;
pub struct Periodic;

/// Interface to a TIMER instance.
///
/// Right now, this is a very basic interface. The timer will always be
/// hardcoded to a frequency of 1 MHz and 32 bits accuracy.
///
/// CC[0] is used for the current/most-recent delay period and CC[1] is used
/// to grab the current value of the counter at a given instant.
pub struct Timer<T, U = OneShot>(T, PhantomData<U>);

impl<T> Timer<T, OneShot>
where
    T: Instance,
{
    pub fn one_shot(timer: T) -> Timer<T, OneShot> {
        timer.set_oneshot();

        Timer::<T, OneShot>(timer, PhantomData)
    }

    pub fn new(timer: T) -> Timer<T, OneShot> {
        Timer::<T, OneShot>::one_shot(timer)
    }
}

impl<T> Timer<T, Periodic>
where
    T: Instance,
{
    pub fn periodic(timer: T) -> Timer<T, Periodic> {
        timer.set_periodic();

        Timer::<T, Periodic>(timer, PhantomData)
    }
}

impl<T, U> Timer<T, U>
where
    T: Instance,
{
    pub const TICKS_PER_SECOND: u32 = 1_000_000;

    pub fn into_periodic(self) -> Timer<T, Periodic> {
        self.0.set_shorts_periodic();

        Timer::<T, Periodic>(self.free(), PhantomData)
    }

    pub fn into_oneshot(self) -> Timer<T, OneShot> {
        self.0.set_shorts_oneshot();

        Timer::<T, OneShot>(self.free(), PhantomData)
    }

    /// Return the raw interface to the underlying timer peripheral.
    pub fn free(self) -> T {
        self.0
    }

    /// Return the current value of the counter, by capturing to CC[1].
    pub fn read(&self) -> u32 {
        self.0.read_counter()
    }

    /// Enables the interrupt for this timer.
    ///
    /// Enables an interrupt that is fired when the timer reaches the value that
    /// is given as an argument to `start`.
    ///
    /// Note that the interrupt also has to be unmasked in the NVIC, or the
    /// handler won't get called.
    pub fn enable_interrupt(&mut self) {
        // As of this writing, the timer code only uses
        // `cc[0]`/`events_compare[0]`. If the code is extended to use other
        // compare registers, the following needs to be adapted.
        self.0.enable_interrupt();
    }

    /// Disables the interrupt for this timer.
    ///
    /// Disables an interrupt that is fired when the timer reaches the value
    /// that is given as an argument to `start`.
    ///
    /// Note that the interrupt also has to be unmasked in the NVIC, or the
    /// handler won't get called.
    pub fn disable_interrupt(&mut self) {
        // As of this writing, the timer code only uses
        // `cc[0]`/`events_compare[0]`. If the code is extended to use other
        // compare registers, the following needs to be adapted.
        self.0.disable_interrupt();
    }

    pub fn delay(&mut self, cycles: u32) {
        self.start(cycles);
        match block!(self.wait()) {
            Ok(_) => {}
            Err(x) => unreachable(x),
        }
    }
}

impl<T, U> timer::CountDown for Timer<T, U>
where
    T: Instance,
{
    type Time = u32;

    /// Start the timer.
    ///
    /// The timer will run for the given number of cycles, then it will stop and
    /// reset.
    fn start<Time>(&mut self, cycles: Time)
    where
        Time: Into<Self::Time>,
    {
        self.0.timer_start(cycles);
    }

    /// Wait for the timer to stop.
    ///
    /// Will return `Err(nb::Error::WouldBlock)` while the timer is still
    /// running. Once the timer reached the number of cycles given in the
    /// `start` method, it will return `Ok(())`.
    ///
    /// To block until the timer has stopped, use the `block!` macro from the
    /// `nb` crate. Please refer to the documentation of `nb` for other options.
    fn wait(&mut self) -> nb::Result<(), Void> {
        if self.0.timer_running() {
            // EVENTS_COMPARE has not been triggered yet
            return Err(nb::Error::WouldBlock);
        }

        // Reset the event, otherwise it will always read `1` from now on.
        self.0.timer_reset_event();

        Ok(())
    }
}

impl<T, U> timer::Cancel for Timer<T, U>
where
    T: Instance,
{
    type Error = ();

    fn cancel(&mut self) -> Result<(), Self::Error> {
        self.0.timer_cancel();
        Ok(())
    }
}

impl<T> timer::Periodic for Timer<T, Periodic> where T: Instance {}

impl<T, U> DelayMs<u32> for Timer<T, U>
where
    T: Instance,
{
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us(ms * 1_000);
    }
}

impl<T, U> DelayMs<u16> for Timer<T, U>
where
    T: Instance,
{
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(u32(ms));
    }
}

impl<T, U> DelayMs<u8> for Timer<T, U>
where
    T: Instance,
{
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(u32(ms));
    }
}

impl<T, U> DelayUs<u32> for Timer<T, U>
where
    T: Instance,
{
    fn delay_us(&mut self, us: u32) {
        self.delay(us);
    }
}

impl<T, U> DelayUs<u16> for Timer<T, U>
where
    T: Instance,
{
    fn delay_us(&mut self, us: u16) {
        self.delay_us(u32(us))
    }
}

impl<T, U> DelayUs<u8> for Timer<T, U>
where
    T: Instance,
{
    fn delay_us(&mut self, us: u8) {
        self.delay_us(u32(us))
    }
}

/// Implemented by all TIMER* instances.
pub trait Instance: sealed::Sealed {
    /// This interrupt associated with this RTC instance.
    const INTERRUPT: Interrupt;

    fn as_timer0(&self) -> &RegBlock0;

    fn timer_start<Time>(&self, cycles: Time)
    where
        Time: Into<u32>,
    {
        // If the following sequence of events occurs, the COMPARE event will be
        // set here:
        // 1. `start` is called.
        // 2. The timer runs out but `wait` is _not_ called.
        // 3. `start` is called again
        //
        // If that happens, then we need to reset the event here explicitly, as
        // nothing else this method does will reset the event, and if it's still
        // active after this method exits, then the next call to `wait` will
        // return immediately, no matter how much time has actually passed.
        self.as_timer0().events_compare[0].reset();

        // Configure timer to trigger EVENTS_COMPARE when given number of cycles
        // is reached.
        #[cfg(not(feature = "51"))]
        self.as_timer0().cc[0].write(|w|
            // The timer mode was set to 32 bits above, so all possible values
            // of `cycles` are valid.
            unsafe { w.cc().bits(cycles.into()) });

        #[cfg(feature = "51")]
        self.as_timer0().cc[0].write(|w| unsafe { w.bits(cycles.into()) });

        // Clear the counter value.
        self.as_timer0().tasks_clear.write(|w| unsafe { w.bits(1) });

        // Start the timer.
        self.as_timer0().tasks_start.write(|w| unsafe { w.bits(1) });
    }

    fn timer_reset_event(&self) {
        self.as_timer0().events_compare[0].write(|w| w);
    }

    fn timer_cancel(&self) {
        self.as_timer0().tasks_stop.write(|w| unsafe { w.bits(1) });
        self.timer_reset_event();
    }

    fn timer_running(&self) -> bool {
        self.as_timer0().events_compare[0].read().bits() == 0
    }

    fn read_counter(&self) -> u32 {
        self.as_timer0().tasks_capture[1].write(|w| unsafe { w.bits(1) });
        self.as_timer0().cc[1].read().bits()
    }

    fn disable_interrupt(&self) {
        self.as_timer0()
            .intenclr
            .modify(|_, w| w.compare0().clear());
    }

    fn enable_interrupt(&self) {
        self.as_timer0().intenset.modify(|_, w| w.compare0().set());
    }

    fn set_shorts_periodic(&self) {
        self.as_timer0()
            .shorts
            .write(|w| w.compare0_clear().enabled().compare0_stop().disabled());
    }

    fn set_shorts_oneshot(&self) {
        self.as_timer0()
            .shorts
            .write(|w| w.compare0_clear().enabled().compare0_stop().enabled());
    }

    fn set_periodic(&self) {
        self.set_shorts_periodic();
        self.as_timer0().prescaler.write(
            |w| unsafe { w.prescaler().bits(4) }, // 1 MHz
        );
        self.as_timer0().bitmode.write(|w| w.bitmode()._32bit());
    }

    fn set_oneshot(&self) {
        self.set_shorts_oneshot();
        self.as_timer0().prescaler.write(
            |w| unsafe { w.prescaler().bits(4) }, // 1 MHz
        );
        self.as_timer0().bitmode.write(|w| w.bitmode()._32bit());
    }
}

impl Instance for TIMER0 {
    const INTERRUPT: Interrupt = Interrupt::TIMER0;

    #[inline(always)]
    fn as_timer0(&self) -> &RegBlock0 {
        self
    }
}

impl Instance for TIMER1 {
    const INTERRUPT: Interrupt = Interrupt::TIMER1;

    #[inline(always)]
    fn as_timer0(&self) -> &RegBlock0 {
        self
    }
}

impl Instance for TIMER2 {
    const INTERRUPT: Interrupt = Interrupt::TIMER2;

    #[inline(always)]
    fn as_timer0(&self) -> &RegBlock0 {
        self
    }
}

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
impl Instance for TIMER3 {
    const INTERRUPT: Interrupt = Interrupt::TIMER3;

    #[inline(always)]
    fn as_timer0(&self) -> &RegBlock0 {
        let rb: &RegBlock3 = self;
        let rb_ptr: *const RegBlock3 = rb;

        // SAFETY: TIMER0 and TIMER3 register layouts are identical, except
        // that TIMER3 has 6 CC registers, while TIMER0 has 4. There is
        // appropriate padding to allow other operations to work correctly
        unsafe { &*rb_ptr.cast() }
    }
}

#[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
impl Instance for TIMER4 {
    const INTERRUPT: Interrupt = Interrupt::TIMER4;

    #[inline(always)]
    fn as_timer0(&self) -> &RegBlock0 {
        let rb: &RegBlock3 = self;
        let rb_ptr: *const RegBlock3 = rb;

        // SAFETY: TIMER0 and TIMER3 register layouts are identical, except
        // that TIMER3 has 6 CC registers, while TIMER0 has 4. There is
        // appropriate padding to allow other operations to work correctly
        unsafe { &*rb_ptr.cast() }
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::TIMER0 {}
    impl Sealed for super::TIMER1 {}
    impl Sealed for super::TIMER2 {}

    #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
    impl Sealed for super::TIMER3 {}

    #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
    impl Sealed for super::TIMER4 {}
}
