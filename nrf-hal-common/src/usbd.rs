use crate::clocks::ExternalOscillator;
use crate::pac::USBD;
use crate::Clocks;

pub use nrf_usbd::Usbd;

#[allow(dead_code)] // fields are unused and only hold ownership
pub struct UsbPeripheral<'a> {
    usbd: USBD,
    clocks: &'a (),
}

impl<'a> UsbPeripheral<'a> {
    pub fn new<L, LSTAT>(usbd: USBD, _clocks: &'a Clocks<ExternalOscillator, L, LSTAT>) -> Self {
        Self { usbd, clocks: &() }
    }
}

unsafe impl<'a> nrf_usbd::UsbPeripheral for UsbPeripheral<'a> {
    const REGISTERS: *const () = USBD::ptr() as *const _;
}
