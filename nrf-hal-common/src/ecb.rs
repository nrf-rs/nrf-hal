//! HAL interface to the AES electronic codebook mode encryption.
//!
//! The ECB encryption block supports 128 bit AES encryption (encryption only, not decryption).


#[cfg(not(feature = "5340-net"))]
use crate::pac::ECB;
#[cfg(feature = "5340-net")]
use crate::pac::ECB_NS as ECB;

use core::sync::atomic::{compiler_fence, Ordering};

/// Error type to represent a sharing conflict during encryption.
#[derive(Debug, Copy, Clone)]
pub struct EncryptionError {}

/// A safe, blocking wrapper around the AES-ECB peripheral.
///
/// It's really just blockwise AES and not an ECB stream cipher. Blocks can be
/// encrypted by calling `crypt_block`.
pub struct Ecb {
    regs: ECB,
}

impl Ecb {
    /// Takes ownership of the `ECB` peripheral, returning a safe wrapper.
    pub fn init(regs: ECB) -> Self {
        // Disable all interrupts
        regs.intenclr
            .write(|w| w.endecb().clear().errorecb().clear());

        // NOTE(unsafe) 1 is a valid pattern to write to this register.
        regs.tasks_stopecb.write(|w| unsafe { w.bits(1) });
        Self { regs }
    }

    /// Destroys `self`, giving the `ECB` peripheral back.
    pub fn into_inner(self) -> ECB {
        // Clear all events
        self.regs.events_endecb.reset();
        self.regs.events_errorecb.reset();

        self.regs
    }

    /// Blocking encryption.
    ///
    /// Encrypts a `block` with `key`.
    ///
    /// # Errors
    ///
    /// An error will be returned when the AES hardware raises an `ERRORECB`
    /// event. This can happen when an operation is started that shares the AES
    /// hardware resources with the AES ECB peripheral while an encryption
    /// operation is running.
    pub fn encrypt_block(
        &mut self,
        block: [u8; 16],
        key: [u8; 16],
    ) -> Result<[u8; 16], EncryptionError> {
        #[repr(C)]
        struct EcbData {
            key: [u8; 16],
            clear_text: [u8; 16],
            cipher_text: [u8; 16],
        }

        // We allocate the DMA'd buffer on the stack, which means that we must
        // not panic or return before the AES operation is finished.
        let mut buf = EcbData {
            key,
            clear_text: block,
            cipher_text: [0; 16],
        };

        // NOTE(unsafe) Any 32bits pattern is safe to write to this register.
        self.regs
            .ecbdataptr
            .write(|w| unsafe { w.bits(&mut buf as *mut _ as u32) });

        // Clear all events.
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
            // It's ok to return here, the events will be cleared before the next encryption.
            return Err(EncryptionError {});
        }
        Ok(buf.cipher_text)
    }
}
