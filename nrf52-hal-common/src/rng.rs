//! HAL interface to the RNG peripheral
//!
//! See nRF52832 product specification, chapter 26.


use core::ops::Deref;
use rand_core::{CryptoRng, RngCore};

use crate::target::{
    rng,
    RNG,
};


pub trait RngExt : Deref<Target=rng::RegisterBlock> + Sized {
    fn constrain(self) -> Rng;
}

impl RngExt for RNG {
    fn constrain(self) -> Rng {
        self.config.write(|w| w.dercen().enabled());
        Rng(self)
    }
}


/// Interface to the RNG peripheral
///
/// Right now, this is very basic, only providing blocking interfaces.
pub struct Rng(RNG);

impl Rng {
    /// Fill the provided buffer with random bytes
    ///
    /// Will block until the buffer is full.
    pub fn random(&mut self, buf: &mut [u8]) {
        self.0.tasks_start.write(|w| unsafe { w.bits(1) });

        for b in buf {
            // Wait for random byte to become ready, reset the flag once it is
            while self.0.events_valrdy.read().bits() == 0 {}
            self.0.events_valrdy.write(|w| unsafe { w.bits(0) });

            *b = self.0.value.read().value().bits();
        }

        self.0.tasks_stop.write(|w| unsafe { w.bits(1) });
    }

    /// Return a random `u8`
    pub fn random_u8(&mut self) -> u8 {
        let mut buf = [0; 1];
        self.random(&mut buf);
        buf[0]
    }

    /// Return a random `u16`
    pub fn random_u16(&mut self) -> u16 {
        let mut buf = [0; 2];
        self.random(&mut buf);
        buf[0] as u16 |
            (buf[1] as u16) << 8
    }

    /// Return a random `u32`
    pub fn random_u32(&mut self) -> u32 {
        let mut buf = [0; 4];
        self.random(&mut buf);
        buf[0] as u32 |
            (buf[1] as u32) <<  8 |
            (buf[2] as u32) << 16 |
            (buf[3] as u32) << 24
    }

    /// Return a random `u64`
    pub fn random_u64(&mut self) -> u64 {
        let mut buf = [0; 8];
        self.random(&mut buf);
        buf[0] as u64 |
            (buf[1] as u64) <<  8 |
            (buf[2] as u64) << 16 |
            (buf[3] as u64) << 24 |
            (buf[4] as u64) << 32 |
            (buf[5] as u64) << 40 |
            (buf[6] as u64) << 48 |
            (buf[7] as u64) << 56
    }
}

impl RngCore for Rng {
    fn next_u32(&mut self) -> u32 {
        self.random_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.random_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.random(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        Ok(self.fill_bytes(dest))
    }
}

impl CryptoRng for Rng {}
