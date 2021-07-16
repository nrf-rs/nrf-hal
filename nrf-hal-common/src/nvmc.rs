//! HAL interface to the Non-Volatile Memory Controller (NVMC) peripheral.

use core::ops::Deref;

#[cfg(any(feature = "52840"))]
use crate::pac::nvmc::*;
#[cfg(any(feature = "9160"))]
use crate::pac::nvmc_ns as nvmc;
#[cfg(any(feature = "52840"))]
use crate::pac::NVMC;
#[cfg(any(feature = "9160"))]
use crate::pac::NVMC_NS as NVMC;

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
        self.nvmc.configns.write(|w| w.wen().een());
    }

    fn enable_write(&self) {
        self.nvmc.configns.write(|w| w.wen().wen());
    }

    fn reset(&self) {
        self.nvmc.configns.reset();
    }

    #[inline]
    fn wait_ready(&self) {
        while !self.nvmc.ready.read().ready().bit_is_set() {}
    }

    #[inline]
    fn write_word(&mut self, offset: usize, word: u32) {
        self.wait_ready();
        self.storage[offset] = word;
        cortex_m::asm::dmb();
    }
}

impl<T> ReadNorFlash for Nvmc<T>
where
    T: Instance,
{
    type Error = NvmcError;

    const READ_SIZE: usize = 4;

    fn try_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let offset = offset as usize;
        let bytes_len = bytes.len();
        let read_len = (bytes_len >> 2)
            + if bytes_len % Self::READ_SIZE == 0 {
                0
            } else {
                1
            };
        let target_offset = offset + read_len;
        if offset % Self::READ_SIZE == 0 && target_offset <= self.capacity() {
            self.wait_ready();
            let mut bytes_offset = offset << 2;
            for offset in offset..(target_offset - 1) {
                let word = self.storage[offset];
                bytes[bytes_offset] = (word >> 24) as u8;
                bytes_offset += 1;
                bytes[bytes_offset] = (word >> 16) as u8;
                bytes_offset += 1;
                bytes[bytes_offset] = (word >> 8) as u8;
                bytes_offset += 1;
                bytes[bytes_offset] = (word >> 0) as u8;
            }
            if target_offset > 0 {
                let offset = target_offset - 1;
                let word = self.storage[offset];
                if bytes_offset < bytes_len {
                    bytes[bytes_offset] = (word >> 24) as u8;
                }
                bytes_offset += 1;
                if bytes_offset < bytes_len {
                    bytes[bytes_offset] = (word >> 16) as u8;
                }
                bytes_offset += 1;
                if bytes_offset < bytes_len {
                    bytes[bytes_offset] = (word >> 8) as u8;
                }
                bytes_offset += 1;
                if bytes_offset < bytes_len {
                    bytes[bytes_offset] = (word >> 0) as u8;
                }
            }
            Ok(())
        } else {
            Err(NvmcError::Unaligned)
        }
    }

    fn capacity(&self) -> usize {
        self.storage.len()
    }
}

impl<T> NorFlash for Nvmc<T>
where
    T: Instance,
{
    const WRITE_SIZE: usize = 4;

    const ERASE_SIZE: usize = 4 * 1024;

    fn try_erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if from as usize % Self::ERASE_SIZE == 0 && to as usize % Self::ERASE_SIZE == 0 {
            self.enable_erase();
            for offset in (from..to).step_by(Self::ERASE_SIZE) {
                self.storage[offset as usize >> 2] = 0xffffffff;
            }
            self.reset();
            Ok(())
        } else {
            Err(NvmcError::Unaligned)
        }
    }

    fn try_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let offset = offset as usize;
        if offset % Self::WRITE_SIZE == 0 && bytes.len() % Self::WRITE_SIZE == 0 {
            self.enable_write();
            for offset in (offset..(offset + bytes.len())).step_by(Self::WRITE_SIZE) {
                let word = ((bytes[offset] as u32) << 24)
                    | ((bytes[offset + 1] as u32) << 16)
                    | ((bytes[offset + 2] as u32) << 8)
                    | ((bytes[offset + 3] as u32) << 0);
                self.write_word(offset >> 2, word);
            }
            self.reset();
            Ok(())
        } else {
            Err(NvmcError::Unaligned)
        }
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
}
