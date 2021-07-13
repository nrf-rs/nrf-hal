//! HAL interface to the Non-Volatile Memory Controller (NVMC) peripheral.

#[cfg(any(feature = "52840"))]
use crate::pac::NVMC;
#[cfg(any(feature = "9160"))]
use crate::pac::NVMC_NS;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};

/// Interface to an NVMC instance.
pub struct Nvmc<'a, T: Instance, const N: usize> {
    nvmc: T,
    storage: &'a mut [u32; N],
}

impl<'a, T, const N: usize> Nvmc<'a, T, N>
where
    T: Instance,
{
    /// Takes ownership of the peripheral and storage area.
    pub fn new(nvmc: T, storage: &'a mut [u32; N]) -> Nvmc<'a, T, N> {
        Self { nvmc, storage }
    }

    /// Consumes `self` and returns back the raw peripheral and associated storage.
    pub fn free(self) -> (T, &'a mut [u32; N]) {
        (self.nvmc, self.storage)
    }
}

impl<'a, T, const N: usize> ReadNorFlash for Nvmc<'a, T, N>
where
    T: Instance,
{
    type Error = ();

    const READ_SIZE: usize = 4;

    fn try_read(&mut self, _offset: u32, _bytes: &mut [u8]) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn capacity(&self) -> usize {
        N
    }
}

impl<'a, T, const N: usize> NorFlash for Nvmc<'a, T, N>
where
    T: Instance,
{
    const WRITE_SIZE: usize = 4;

    const ERASE_SIZE: usize = 4 * 1024;

    fn try_erase(&mut self, _from: u32, _to: u32) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn try_write(&mut self, _offset: u32, _bytes: &[u8]) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

pub trait Instance {
    fn enable_erase();
    fn enable_write();
    fn reset();
    fn write_word(offset: u32, word: u32);
}

#[cfg(any(feature = "52840"))]
impl Instance for NVMC {
    fn enable_erase() {
        unimplemented!()
    }
    fn enable_write() {
        unimplemented!()
    }
    fn reset() {
        unimplemented!()
    }
    fn write_word(_offset: u32, _word: u32) {
        unimplemented!()
    }
}

#[cfg(any(feature = "9160"))]
impl Instance for NVMC_NS {
    fn enable_erase() {
        unimplemented!()
    }
    fn enable_write() {
        unimplemented!()
    }
    fn reset() {
        unimplemented!()
    }
    fn write_word(_offset: u32, _word: u32) {
        unimplemented!()
    }
}
