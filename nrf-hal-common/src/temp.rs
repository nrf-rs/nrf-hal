//! Temperature sensor interface.

#[cfg(not(feature = "5340-net"))]
use crate::pac::TEMP;
#[cfg(feature = "5340-net")]
use crate::pac::TEMP_NS as TEMP;

use fixed::types::I30F2;
use void::Void;

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
    ///
    /// Returns the measured temperature in °C.
    ///
    /// Note: This return type is [fixed::types::I30F2]. It
    /// can be converted into [f32] or [f64] (or other numeric types)
    /// via the `to_num()` method.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut temp = Temp::new(board.TEMP);
    /// let deg_c: f32 = temp.measure().to_num();
    /// ```
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
            Err(nb::Error::WouldBlock)
        } else {
            self.0.events_datardy.reset(); // clear event
            let raw = self.0.temp.read().bits();
            Ok(I30F2::from_bits(raw as i32))
        }
    }
}
