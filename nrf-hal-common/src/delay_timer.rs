//! Implementations of the embedded-hal `Delay` trait using Timers

use cast::u32;

use embedded_hal::blocking::delay::{DelayMs, DelayUs};

use crate::timer::{self, Timer};
pub struct Delay<T, U>(Timer<T, U>);

impl<T: timer::Instance, U> Delay<T, U> {
    /// Configures a Timer as a delay provider
    pub fn new(timer: Timer<T, U>) -> Self {
        Self(timer)
    }

    /// Releases the Timer resource
    pub fn free(self) -> Timer<T, U> {
        self.0
    }
}

impl<T: timer::Instance, U> DelayMs<u32> for Delay<T, U> {
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us(ms * 1_000);
    }
}

impl<T: timer::Instance, U> DelayMs<u16> for Delay<T, U> {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(u32(ms));
    }
}

impl<T: timer::Instance, U> DelayMs<u8> for Delay<T, U> {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(u32(ms));
    }
}

impl<T: timer::Instance, U> DelayUs<u32> for Delay<T, U> {
    fn delay_us(&mut self, us: u32) {
        self.0.delay(us);
    }
}

impl<T: timer::Instance, U> DelayUs<u16> for Delay<T, U> {
    fn delay_us(&mut self, us: u16) {
        self.delay_us(u32(us))
    }
}

impl<T: timer::Instance, U> DelayUs<u8> for Delay<T, U> {
    fn delay_us(&mut self, us: u8) {
        self.delay_us(u32(us))
    }
}
