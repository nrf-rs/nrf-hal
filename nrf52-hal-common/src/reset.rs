//! Reset control (nRF5340-APP only).

use crate::target::RESET_S;

pub struct ResetController {
    raw: RESET_S,
}

impl ResetController {
    pub fn new(raw: RESET_S) -> Self {
        Self { raw }
    }

    pub fn is_network_forced_off(&self) -> bool {
        self.raw.network.forceoff.read().forceoff().is_hold()
    }

    pub fn set_network_power(&mut self, on: bool) {
        self.raw.network.forceoff.write(|w| w.forceoff().bit(!on))
    }
}
