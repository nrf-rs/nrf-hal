//! HAL interface to the Non-Volatile Memory Controller (NVMC) peripheral.

use core::ops::Deref;

#[cfg(not(any(feature = "9160", feature = "5340-app")))]
use crate::pac::nvmc;
#[cfg(any(feature = "9160", feature = "5340-app"))]
use crate::pac::nvmc_ns as nvmc;
#[cfg(not(any(feature = "9160", feature = "5340-app")))]
use crate::pac::NVMC;
#[cfg(any(feature = "9160", feature = "5340-app"))]
use crate::pac::NVMC_NS as NVMC;

use core::convert::TryInto;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};

/// Interface to an NVMC instance.
pub struct Nvmc<T: Instance> {
    nvmc: T,
    storage: &'static mut [u32],
}

impl<T> Nvmc<T>
where
    T: Instance,
{
    /// Takes ownership of the peripheral and storage area.
    pub fn new(nvmc: T, storage: &'static mut [u32]) -> Nvmc<T> {
        Self { nvmc, storage }
    }

    /// Consumes `self` and returns back the raw peripheral and associated storage.
    pub fn free(self) -> (T, &'static mut [u32]) {
        (self.nvmc, self.storage)
    }

    fn enable_erase(&self) {
        #[cfg(not(any(feature = "9160", feature = "5340-app")))]
        self.nvmc.config.write(|w| w.wen().een());
        #[cfg(any(feature = "9160", feature = "5340-app"))]
        self.nvmc.configns.write(|w| w.wen().een());
    }

    fn enable_read(&self) {
        #[cfg(not(any(feature = "9160", feature = "5340-app")))]
        self.nvmc.config.write(|w| w.wen().ren());
        #[cfg(any(feature = "9160", feature = "5340-app"))]
        self.nvmc.configns.write(|w| w.wen().ren());
    }

    fn enable_write(&self) {
        #[cfg(not(any(feature = "9160", feature = "5340-app")))]
        self.nvmc.config.write(|w| w.wen().wen());
        #[cfg(any(feature = "9160", feature = "5340-app"))]
        self.nvmc.configns.write(|w| w.wen().wen());
    }

    #[inline]
    fn wait_ready(&self) {
        while !self.nvmc.ready.read().ready().bit_is_set() {}
    }

    #[cfg(any(feature = "9160", feature = "5340-app"))]
    #[inline]
    fn wait_write_ready(&self) {
        while !self.nvmc.readynext.read().readynext().bit_is_set() {}
    }

    #[cfg(not(any(feature = "9160", feature = "5340-app")))]
    #[inline]
    fn erase_page(&mut self, offset: usize) {
        let bits = &mut (self.storage[offset as usize >> 2]) as *mut _ as u32;
        self.nvmc.erasepage().write(|w| unsafe { w.bits(bits) });
        self.wait_ready();
    }

    #[cfg(any(feature = "9160", feature = "5340-app"))]
    #[inline]
    fn erase_page(&mut self, offset: usize) {
        self.storage[offset as usize >> 2] = 0xffffffff;
        self.wait_ready();
    }

    #[inline]
    fn write_word(&mut self, offset: usize, word: u32) {
        #[cfg(not(any(feature = "9160", feature = "5340-app")))]
        self.wait_ready();
        #[cfg(any(feature = "9160", feature = "5340-app"))]
        self.wait_write_ready();
        self.storage[offset] = word;
        cortex_m::asm::dmb();
    }
}

impl<T> ReadNorFlash for Nvmc<T>
where
    T: Instance,
{
    type Error = NvmcError;

    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, mut bytes: &mut [u8]) -> Result<(), Self::Error> {
        let mut offset = offset as usize;
        if bytes.len() > self.capacity() || offset > self.capacity() - bytes.len() {
            return Err(NvmcError::OutOfBounds);
        }
        self.wait_ready();
        if offset & 3 != 0 {
            let word = self.storage[offset >> 2].to_ne_bytes();
            let start = offset & 3;
            let length = 4 - start;
            if length > bytes.len() {
                bytes.copy_from_slice(&word[start..start + bytes.len()]);
                return Ok(());
            }
            bytes[..length].copy_from_slice(&word[start..]);
            offset = offset + length;
            bytes = &mut bytes[length..];
        }
        let mut word_offset = offset >> 2;
        let mut chunks = bytes.chunks_exact_mut(4);
        for bytes in &mut chunks {
            bytes.copy_from_slice(&self.storage[word_offset].to_ne_bytes());
            word_offset += 1;
        }
        let bytes = chunks.into_remainder();
        if !bytes.is_empty() {
            bytes.copy_from_slice(&self.storage[word_offset].to_ne_bytes()[..bytes.len()]);
        }
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.storage.len() << 2
    }
}

impl<T> NorFlash for Nvmc<T>
where
    T: Instance,
{
    const WRITE_SIZE: usize = 4;

    const ERASE_SIZE: usize = 4 * 1024;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        let from = from as usize;
        let to = to as usize;
        if from > to || to > self.capacity() {
            return Err(NvmcError::OutOfBounds);
        }
        if from % Self::ERASE_SIZE != 0 || to % Self::ERASE_SIZE != 0 {
            return Err(NvmcError::Unaligned);
        }
        self.enable_erase();
        for offset in (from..to).step_by(Self::ERASE_SIZE) {
            self.erase_page(offset);
        }
        self.enable_read();
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let offset = offset as usize;
        if bytes.len() > self.capacity() || offset as usize > self.capacity() - bytes.len() {
            return Err(NvmcError::OutOfBounds);
        }
        if offset % Self::WRITE_SIZE != 0 || bytes.len() % Self::WRITE_SIZE != 0 {
            return Err(NvmcError::Unaligned);
        }
        self.enable_write();
        let mut word_offset = offset >> 2;
        for bytes in bytes.chunks_exact(4) {
            // The unwrap is correct because chunks_exact always returns the correct size.
            self.write_word(word_offset, u32::from_ne_bytes(bytes.try_into().unwrap()));
            word_offset += 1;
        }
        self.enable_read();
        Ok(())
    }
}

pub trait Instance: Deref<Target = nvmc::RegisterBlock> + sealed::Sealed {}

impl Instance for NVMC {}

mod sealed {
    use super::*;

    pub trait Sealed {}

    impl Sealed for NVMC {}
}

#[derive(Debug)]
pub enum NvmcError {
    /// An operation was attempted on an unaligned boundary
    Unaligned,
    /// An operation was attempted outside the boundaries
    OutOfBounds,
}
