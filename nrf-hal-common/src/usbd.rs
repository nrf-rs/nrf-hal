use core::marker::PhantomData;

use crate::clocks::ExternalOscillator;
use crate::pac::USBD;
use crate::Clocks;

pub use nrf_usbd::Usbd;

pub struct UsbPeripheral<'a> {
    _usbd: USBD,
    _clocks: PhantomData<&'a ()>,
}

impl<'a> UsbPeripheral<'a> {
    pub fn new<L, LSTAT>(usbd: USBD, _clocks: &'a Clocks<ExternalOscillator, L, LSTAT>) -> Self {
        Self {
            _usbd: usbd,
            _clocks: PhantomData,
        }
    }
}

unsafe impl<'a> nrf_usbd::UsbPeripheral for UsbPeripheral<'a> {
    const REGISTERS: *const () = USBD::ptr() as *const _;
}
