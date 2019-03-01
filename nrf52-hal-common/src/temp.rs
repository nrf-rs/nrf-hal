//! Temperature sensor interface.

use fpa::I30F2;
use nb;
use void::Void;
use crate::target::TEMP;

/// Integrated temperature sensor.
pub struct Temp(TEMP);

impl Temp {
    /// Creates a new `Temp`, taking ownership of the temperature sensor's register block.
    pub fn new(raw: TEMP) -> Self {
        Temp(raw)
    }

    /// Starts a new measurement and blocks until completion.
    ///
    /// If a measurement was already started, it will be canceled.
    pub fn measure(&mut self) -> I30F2 {
        self.stop_measurement();
        self.start_measurement();

        nb::block!(self.read()).unwrap()
    }

    /// Kicks off a temperature measurement.
    ///
    /// The measurement can be retrieved by calling `read`.
    pub fn start_measurement(&mut self) {
        unsafe {
            self.0.tasks_start.write(|w| w.bits(1));
        }
    }

    /// Cancels an in-progress temperature measurement.
    pub fn stop_measurement(&mut self) {
        unsafe {
            self.0.tasks_stop.write(|w| w.bits(1));
            self.0.events_datardy.reset();
        }
    }

    /// Tries to read a started measurement (non-blocking).
    ///
    /// Before calling this, `start_measurement` must be called.
    ///
    /// Returns the measured temperature in °C.
    pub fn read(&mut self) -> nb::Result<I30F2, Void> {
        if self.0.events_datardy.read().bits() == 0 {
            return Err(nb::Error::WouldBlock);
        } else {
            self.0.events_datardy.reset(); // clear event
            let raw = self.0.temp.read().bits();
            Ok(I30F2::from_bits(raw as i32))
        }
    }
}
