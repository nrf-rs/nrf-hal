use crate::target::{uicr, NVMC, UICR};

use core::ops::Deref;

pub struct Uicr<T>(T);

impl<T> Uicr<T>
where
    T: Instance,
{
    pub fn new(uicr: T) -> Self {
        Self(uicr)
    }

    pub fn erase(&mut self, nvmc: &mut NVMC) {
        assert!(nvmc.config.read().wen().is_wen() == false); // write + erase is forbidden!

        nvmc.config.write(|w| w.wen().een());
        nvmc.eraseuicr.write(|w| w.eraseuicr().erase());
        nvmc.config.reset()
    }

    pub fn store_customer(&mut self, nvmc: &mut NVMC, offset: usize, values: &[u32]) {
        assert!(values.len() + offset <= self.0.customer.len()); // ensure we fit
        assert!(nvmc.config.read().wen().is_een() == false); // write + erase is forbidden!

        nvmc.config.write(|w| w.wen().wen());
        for (i, value) in values.iter().enumerate() {
            self.0.customer[offset + i].write(|w| unsafe { w.customer().bits(*value) });
        }
        nvmc.config.reset()
    }

    pub fn load_customer<'a>(&mut self, offset: usize, values: &'a mut [u32]) -> &'a [u32] {
        let range = offset..offset + values.len();
        for (i, reg_i) in range.enumerate() {
            values[i] = self.0.customer[reg_i].read().customer().bits()
        }

        values
    }
}

pub trait Instance: Deref<Target = uicr::RegisterBlock> {}

impl Instance for UICR {}
