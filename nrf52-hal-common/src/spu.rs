//! Wrapper around the System Protection Unit (SPU).
//!
//! Only available on the nRF5340 Application core.

use crate::target::spu_s::extdomain::perm::SECATTR_A;
use crate::target::SPU_S;

/// High-level interface to the System Protection Unit (SPU).
pub struct Spu {
    raw: SPU_S,
}

impl Spu {
    /// Creates an SPU wrapper by taking ownership of the peripheral.
    pub fn new(raw: SPU_S) -> Self {
        Self { raw }
    }

    /// Destroys the SPU wrapper and returns ownership of the SPU.
    ///
    /// This will not modify the SPU configuration.
    pub fn free(self) -> SPU_S {
        self.raw
    }

    /// Returns whether TrustZone is supported.
    pub fn supports_trustzone(&self) -> bool {
        self.raw.cap.read().tzm().is_enabled()
    }

    /// Returns the set of events currently signaled by the SPU.
    pub fn event_status(&self) -> Events {
        let mut events = Events::empty();
        if self
            .raw
            .events_ramaccerr
            .read()
            .events_ramaccerr()
            .is_generated()
        {
            events |= Events::RAM_ACCESS_ERR;
        }
        if self
            .raw
            .events_flashaccerr
            .read()
            .events_flashaccerr()
            .is_generated()
        {
            events |= Events::FLASH_ACCESS_ERR;
        }
        if self
            .raw
            .events_periphaccerr
            .read()
            .events_periphaccerr()
            .is_generated()
        {
            events |= Events::PERIPH_ACCESS_ERR;
        }
        events
    }

    /// Clears all SPU event flags.
    pub fn clear_events(&mut self) {
        self.raw.events_ramaccerr.reset();
        self.raw.events_flashaccerr.reset();
        self.raw.events_periphaccerr.reset();
    }

    /// Enables the SPU interrupt to fire when any of the given events is fired.
    ///
    /// This will add `events` to the configured set of events that triggers the SPU interrupt. To
    /// remove events from that set, use [`unlisten`].
    ///
    /// Note that a `SecureFault` or `BusFault` exception may be raised in addition to the SPU event
    /// and interrupt. Refer to the reference manual for precise information about this.
    ///
    /// [`unlisten`]: #method.unlisten
    pub fn listen(&mut self, events: Events) {
        self.raw
            .intenset
            .write(|w| unsafe { w.bits(events.bits()) });
    }

    /// Disables the SPU interrupt from firing when any of the given events are fired.
    ///
    /// This will remove `events` from the set of events that trigger the SPU interrupt. To add
    /// events to that set, use [`listen`].
    ///
    /// [`listen`]: #method.listen
    pub fn unlisten(&mut self, events: Events) {
        self.raw
            .intenclr
            .write(|w| unsafe { w.bits(events.bits()) });
    }

    /// Sets the security attribute for Network core bus accesses.
    ///
    /// By default, flash has the *secure* attribute bit set, meaning that accesses from non-secure
    /// code, including the Network core after reset, will cause a fault. Giving the Network core
    /// the `Secure` attribute prevents that.
    pub fn set_network_domain_security(&mut self, attr: SecAttr) {
        self.raw.extdomain[0].perm.write(|w| {
            w.secattr().variant(match attr {
                SecAttr::NonSecure => SECATTR_A::NONSECURE,
                SecAttr::Secure => SECATTR_A::SECURE,
            })
        });
    }
}

bitflags::bitflags! {
    /// Bit flags for SPU events.
    // NOTE: These must be the same bit order as in the INTEN/INTENSET/INTENCLR registers!
    pub struct Events: u32 {
        const RAM_ACCESS_ERR = 1 << 0;
        const FLASH_ACCESS_ERR = 1 << 1;
        const PERIPH_ACCESS_ERR = 1 << 2;
    }
}

/// Security attributes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SecAttr {
    NonSecure = 0,
    Secure = 1,
}
