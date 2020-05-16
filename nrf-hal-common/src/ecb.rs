//! HAL interface to the AES electronic codebook mode encryption
//!
//! The ECB encryption block supports 128 bit AES encryption (encryption only, not decryption).

use crate::target::ECB;
use core::sync::atomic::{compiler_fence, Ordering};

#[derive(Debug, Copy, Clone)]
/// Error type to represent a sharing conflict during encryption
pub struct EncryptionError {}

#[repr(C)]
/// Type that represents the data structure used by the ECB module
pub struct EcbData {
    key: [u8; 16],
    clear_text: [u8; 16],
    chiper_text: [u8; 16],
}

impl EcbData {
    /// Creates the data structures needed for ECB utilization
    ///
    /// If `clear_text` is `None` it will be initialized to zero
    pub fn new(key: [u8; 16], clear_text: Option<[u8; 16]>) -> Self {
        Self {
            key,
            clear_text: clear_text.unwrap_or_default(),
            chiper_text: [0; 16],
        }
    }
}

/// HAL structure interface to use the capabilities of the ECB peripheral
pub struct Ecb {
    regs: ECB,
    data: EcbData,
}

impl Ecb {
    /// Method for initialization
    pub fn init(regs: ECB, data: EcbData) -> Self {
        // Disable all interrupts
        regs.intenclr
            .write(|w| w.endecb().clear().errorecb().clear());

        // NOTE(unsafe) 1 is a valid pattern to write to this register
        regs.tasks_stopecb.write(|w| unsafe { w.bits(1) });
        Self { regs, data }
    }

    /// Gets a reference to the clear text memory
    ///
    /// This is the data that will be encrypted by the encrypt method
    #[inline]
    pub fn clear_text(&mut self) -> &mut [u8; 16] {
        &mut self.data.clear_text
    }

    /// Get a reference to the cipher text memory
    ///
    /// This will contain the encrypted data after a successful encryption
    #[inline]
    pub fn cipher_text(&mut self) -> &mut [u8; 16] {
        &mut self.data.chiper_text
    }

    /// Encrypts the data in the `clear_text` field, the encrypted data will be located in the
    /// cipher text field only if this method returns `Ok`
    ///
    /// In case of an error, this method will return `Err(EncryptionError)`, in this case, the data
    /// in `cipher_text` is not valid
    pub fn encrypt(&mut self) -> Result<(), EncryptionError> {
        // Ecb data is repr(C) and has no padding
        let data_ptr = &mut self.data as *mut _ as u32;

        // NOTE(unsafe) Any 32bits pattern is safe to write to this register
        self.regs.ecbdataptr.write(|w| unsafe { w.bits(data_ptr) });

        // Clear all events
        self.regs.events_endecb.reset();
        self.regs.events_errorecb.reset();

        // "Preceding reads and writes cannot be moved past subsequent writes."
        compiler_fence(Ordering::Release);
        // NOTE(unsafe) 1 is a valid pattern to write to this register
        self.regs.tasks_startecb.write(|w| unsafe { w.bits(1) });

        while self.regs.events_endecb.read().bits() == 0
            && self.regs.events_errorecb.read().bits() == 0
        {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        compiler_fence(Ordering::Acquire);

        if self.regs.events_errorecb.read().bits() == 1 {
            // It's ok to return here, the events will be cleared before the next encryption
            return Err(EncryptionError {});
        }
        Ok(())
    }
}
