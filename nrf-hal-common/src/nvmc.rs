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
use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

type WORD = u32;
const WORD_SIZE: usize = core::mem::size_of::<WORD>();
const PAGE_SIZE: usize = 4 * 1024;

/// Interface to an NVMC instance.
pub struct Nvmc<T: Instance> {
    nvmc: T,
    storage: &'static mut [u8],
}

impl<T> Nvmc<T>
where
    T: Instance,
{
    /// Takes ownership of the peripheral and storage area.
    ///
    /// The storage area must be page-aligned.
    pub fn new(nvmc: T, storage: &'static mut [u8]) -> Nvmc<T> {
        assert!(storage.as_ptr() as usize % PAGE_SIZE == 0);
        assert!(storage.len() % PAGE_SIZE == 0);
        Self { nvmc, storage }
    }

    /// Consumes `self` and returns back the raw peripheral and associated storage.
    pub fn free(self) -> (T, &'static mut [u8]) {
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
    fn erase_page(&mut self, page_offset: usize) {
        let bits = &mut (self.storage[page_offset * PAGE_SIZE]) as *mut _ as u32;
        self.nvmc.erasepage().write(|w| unsafe { w.bits(bits) });
        self.wait_ready();
    }

    #[cfg(any(feature = "9160", feature = "5340-app"))]
    #[inline]
    fn erase_page(&mut self, page_offset: usize) {
        self.direct_write_word(page_offset * PAGE_SIZE, 0xffffffff);
        self.wait_ready();
    }

    #[inline]
    fn write_word(&mut self, word_offset: usize, word: u32) {
        #[cfg(not(any(feature = "9160", feature = "5340-app")))]
        self.wait_ready();
        #[cfg(any(feature = "9160", feature = "5340-app"))]
        self.wait_write_ready();
        self.direct_write_word(word_offset, word);
        cortex_m::asm::dmb();
    }

    #[inline]
    fn direct_write_word(&mut self, word_offset: usize, word: u32) {
        let target: &mut [u8] = &mut self.storage[word_offset * WORD_SIZE..][..WORD_SIZE];
        let target: &mut [u8; WORD_SIZE] = target.try_into().unwrap();
        let target: &mut u32 = unsafe { core::mem::transmute(target) };
        *target = word;
    }
}

impl<T: Instance> ErrorType for Nvmc<T> {
    type Error = NvmcError;
}

impl<T> ReadNorFlash for Nvmc<T>
where
    T: Instance,
{
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let offset = offset as usize;
        if bytes.len() > self.capacity() || offset > self.capacity() - bytes.len() {
            return Err(NvmcError::OutOfBounds);
        }
        self.wait_ready();
        bytes.copy_from_slice(&self.storage[offset..][..bytes.len()]);
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.storage.len()
    }
}

impl<T> NorFlash for Nvmc<T>
where
    T: Instance,
{
    const WRITE_SIZE: usize = WORD_SIZE;

    const ERASE_SIZE: usize = PAGE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        let (from, to) = (from as usize, to as usize);
        if from > to || to > self.capacity() {
            return Err(NvmcError::OutOfBounds);
        }
        if from % PAGE_SIZE != 0 || to % PAGE_SIZE != 0 {
            return Err(NvmcError::Unaligned);
        }
        let (page_from, page_to) = (from / PAGE_SIZE, to / PAGE_SIZE);
        self.enable_erase();
        for page_offset in page_from..page_to {
            self.erase_page(page_offset);
        }
        self.enable_read();
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let offset = offset as usize;
        if bytes.len() > self.capacity() || offset as usize > self.capacity() - bytes.len() {
            return Err(NvmcError::OutOfBounds);
        }
        if offset % WORD_SIZE != 0 || bytes.len() % WORD_SIZE != 0 {
            return Err(NvmcError::Unaligned);
        }
        let word_offset = offset / WORD_SIZE;
        self.enable_write();
        for (word_offset, bytes) in (word_offset..).zip(bytes.chunks_exact(WORD_SIZE)) {
            self.write_word(word_offset, u32::from_ne_bytes(bytes.try_into().unwrap()));
        }
        self.enable_read();
        Ok(())
    }
}

// Only nRF52 boards have been checked. There are 2 things to note:
//
// 1. The nRF52832 doesn't support 2 writes per word. Instead it supports 181 writes per block,
// where a block is 128 words. So on average it's a bit less than 2 writes per word, and thus we
// can't implement MultiwriteNorFlash.
//
// 2. The nRF52820 supports 2 writes per word but doesn't have an associated feature.
#[cfg(any(
    feature = "52810",
    feature = "52811",
    feature = "52833",
    feature = "52840",
))]
impl<T: Instance> embedded_storage::nor_flash::MultiwriteNorFlash for Nvmc<T> {}

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

impl NorFlashError for NvmcError {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            NvmcError::Unaligned => NorFlashErrorKind::NotAligned,
            NvmcError::OutOfBounds => NorFlashErrorKind::OutOfBounds,
        }
    }
}
