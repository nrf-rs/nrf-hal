//! HAL interface to the RNG peripheral.
//!
//! See nRF52832 product specification, chapter 26.

use core::convert::Infallible;
use rand_core::{TryCryptoRng, TryRng};

#[cfg(not(feature = "5340-net"))]
use crate::pac::RNG;
#[cfg(feature = "5340-net")]
use crate::pac::RNG_NS as RNG;

/// Interface to the RNG peripheral.
///
/// Right now, this is very basic, only providing blocking interfaces.
pub struct Rng(RNG);

impl Rng {
    pub fn new(rng: RNG) -> Self {
        rng.config.write(|w| w.dercen().enabled());
        Self(rng)
    }

    /// By default, the RNG peripheral uses a "debiasing
    /// algorithm" (undocumented, but probably a Von Neumann
    /// Corrector) to improve output quality. You can
    /// `set_debiasing(false)` to disable the debiasing,
    /// which will make the RNG peripheral deliver bits
    /// faster, but will likely substantially reduce the
    /// quality of these bits.
    pub fn set_debiasing(&mut self, enabled: bool) {
        if enabled {
            self.0.config.write(|w| w.dercen().enabled());
        } else {
            self.0.config.write(|w| w.dercen().disabled());
        }
    }

    /// Fill the provided buffer with random bytes.
    ///
    /// Will block until the buffer is full.
    pub fn random(&mut self, buf: &mut [u8]) {
        self.0.tasks_start.write(|w| unsafe { w.bits(1) });

        for b in buf {
            // Wait for random byte to become ready, reset the flag once it is.
            while self.0.events_valrdy.read().bits() == 0 {}
            self.0.events_valrdy.write(|w| unsafe { w.bits(0) });

            *b = self.0.value.read().value().bits();
        }

        self.0.tasks_stop.write(|w| unsafe { w.bits(1) });
    }

    /// Return a random `u8`.
    pub fn random_u8(&mut self) -> u8 {
        let mut buf = [0; 1];
        self.random(&mut buf);
        buf[0]
    }

    /// Return a random `u16`.
    pub fn random_u16(&mut self) -> u16 {
        let mut buf = [0; 2];
        self.random(&mut buf);
        buf[0] as u16 | (buf[1] as u16) << 8
    }

    /// Return a random `u32`.
    pub fn random_u32(&mut self) -> u32 {
        let mut buf = [0; 4];
        self.random(&mut buf);
        buf[0] as u32 | (buf[1] as u32) << 8 | (buf[2] as u32) << 16 | (buf[3] as u32) << 24
    }

    /// Return a random `u64`.
    pub fn random_u64(&mut self) -> u64 {
        let mut buf = [0; 8];
        self.random(&mut buf);
        buf[0] as u64
            | (buf[1] as u64) << 8
            | (buf[2] as u64) << 16
            | (buf[3] as u64) << 24
            | (buf[4] as u64) << 32
            | (buf[5] as u64) << 40
            | (buf[6] as u64) << 48
            | (buf[7] as u64) << 56
    }
}

impl TryRng for Rng {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(self.random_u32())
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        Ok(self.random_u64())
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        self.random(dest);
        Ok(())
    }
}

impl TryCryptoRng for Rng {}
