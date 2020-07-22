use crate::pac::{Interrupt, NVIC};
use bbqueue::{ArrayLength, Consumer, Error, GrantR, GrantW, Producer};
use core::ops::{Deref, DerefMut};

pub struct UarteApp<OutgoingLen, IncomingLen>
where
    OutgoingLen: ArrayLength<u8>,
    IncomingLen: ArrayLength<u8>,
{
    pub(crate) outgoing_prod: Producer<'static, OutgoingLen>,
    pub incoming_cons: Consumer<'static, IncomingLen>,
}

impl<OutgoingLen, IncomingLen> UarteApp<OutgoingLen, IncomingLen>
where
    OutgoingLen: ArrayLength<u8>,
    IncomingLen: ArrayLength<u8>,
{
    pub fn read(&mut self) -> Result<UarteGrantR<'static, IncomingLen>, Error> {
        self.incoming_cons
            .read()
            .map(|gr| UarteGrantR { grant_r: gr })
    }

    pub fn write_grant(
        &mut self,
        bytes: usize,
    ) -> Result<UarteGrantW<'static, OutgoingLen>, Error> {
        self.outgoing_prod
            .grant_exact(bytes)
            .map(|gr| UarteGrantW { grant_w: gr })
    }
}

/// A write grant for a single Uarte
///
/// NOTE: If the grant is dropped without explicitly commiting
/// the contents, then no Uarte will be comitted for writing.
#[derive(Debug, PartialEq)]
pub struct UarteGrantW<'a, N>
where
    N: ArrayLength<u8>,
{
    grant_w: GrantW<'a, N>,
}

/// A read grant for a single Uarte
///
/// NOTE: If the grant is dropped without explicitly releasing
/// the contents, then no Uarte will be released.
#[derive(Debug, PartialEq)]
pub struct UarteGrantR<'a, N>
where
    N: ArrayLength<u8>,
{
    grant_r: GrantR<'a, N>,
}

impl<'a, N> Deref for UarteGrantW<'a, N>
where
    N: ArrayLength<u8>,
{
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.grant_w
    }
}

impl<'a, N> DerefMut for UarteGrantW<'a, N>
where
    N: ArrayLength<u8>,
{
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.grant_w
    }
}

impl<'a, N> Deref for UarteGrantR<'a, N>
where
    N: ArrayLength<u8>,
{
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.grant_r
    }
}

impl<'a, N> DerefMut for UarteGrantR<'a, N>
where
    N: ArrayLength<u8>,
{
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.grant_r
    }
}

impl<'a, N> UarteGrantW<'a, N>
where
    N: ArrayLength<u8>,
{
    /// Commit a Uarte to make it available to the Consumer half.
    ///
    /// `used` is the size of the payload, in bytes, not
    /// including the Uarte header
    pub fn commit(self, used: usize) {
        // Commit the header + Uarte
        self.grant_w.commit(used);
        NVIC::pend(Interrupt::UARTE0_UART0);
    }
}

impl<'a, N> UarteGrantR<'a, N>
where
    N: ArrayLength<u8>,
{
    /// Release a Uarte to make the space available for future writing
    ///
    /// Note: The full Uarte is always released
    pub fn release(self, used: usize) {
        self.grant_r.release(used);
        NVIC::pend(Interrupt::UARTE0_UART0);
    }
}
