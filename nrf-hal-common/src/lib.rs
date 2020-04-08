#![no_std]

use embedded_hal as hal;

#[cfg(feature = "51")]
pub use nrf51 as target;

#[cfg(feature = "52810")]
pub use nrf52810_pac as target;

#[cfg(feature = "52832")]
pub use nrf52832_pac as target;

#[cfg(feature = "52840")]
pub use nrf52840_pac as target;

#[cfg(feature = "9160")]
pub use nrf9160_pac as target;

#[cfg(feature = "51")]
pub mod adc;
pub mod clocks;
#[cfg(not(feature = "51"))]
pub mod delay;
pub mod gpio;
#[cfg(not(feature = "9160"))]
pub mod rng;
pub mod rtc;
#[cfg(not(feature = "51"))]
pub mod saadc;
#[cfg(not(feature = "51"))]
pub mod spim;
#[cfg(not(feature = "9160"))]
pub mod temp;
pub mod time;
pub mod timer;
#[cfg(not(feature = "51"))]
pub mod twim;
#[cfg(feature = "51")]
pub mod uart;
#[cfg(not(feature = "51"))]
pub mod uarte;
#[cfg(not(feature = "9160"))]
pub mod uicr;

pub mod prelude {
    pub use crate::hal::digital::v2::*;
    pub use crate::hal::prelude::*;

    pub use crate::time::U32Ext;
}

/// Length of Nordic EasyDMA differs for MCUs
#[cfg(any(feature = "52810", feature = "52832"))]
pub mod target_constants {
    // NRF52832 8 bits1..0xFF
    pub const EASY_DMA_SIZE: usize = 255;
    // Easy DMA can only read from data ram
    pub const SRAM_LOWER: usize = 0x2000_0000;
    pub const SRAM_UPPER: usize = 0x3000_0000;
    pub const FORCE_COPY_BUFFER_SIZE: usize = 255;
}
#[cfg(any(feature = "52840", feature = "9160"))]
pub mod target_constants {
    // NRF52840 and NRF9160 16 bits 1..0xFFFF
    pub const EASY_DMA_SIZE: usize = 65535;
    // Limits for Easy DMA - it can only read from data ram
    pub const SRAM_LOWER: usize = 0x2000_0000;
    pub const SRAM_UPPER: usize = 0x3000_0000;
    pub const FORCE_COPY_BUFFER_SIZE: usize = 1024;
}

/// Does this slice reside entirely within RAM?
#[cfg(not(feature = "51"))]
pub(crate) fn slice_in_ram(slice: &[u8]) -> bool {
    let ptr = slice.as_ptr() as usize;
    ptr >= target_constants::SRAM_LOWER && (ptr + slice.len()) < target_constants::SRAM_UPPER
}

/// Return an error if slice is not in RAM
#[cfg(not(feature = "51"))]
pub(crate) fn slice_in_ram_or<T>(slice: &[u8], err: T) -> Result<(), T> {
    if slice_in_ram(slice) {
        Ok(())
    } else {
        Err(err)
    }
}

/// A handy structure for converting rust slices into ptr and len pairs
/// for use with EasyDMA. Care must be taken to make sure mutability
/// guarantees are respected
#[cfg(not(feature = "51"))]
pub(crate) struct DmaSlice {
    ptr: u32,
    len: u32,
}

#[cfg(not(feature = "51"))]
impl DmaSlice {
    pub fn null() -> Self {
        Self { ptr: 0, len: 0 }
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            ptr: slice.as_ptr() as u32,
            len: slice.len() as u32,
        }
    }
}

pub use crate::clocks::Clocks;
#[cfg(not(feature = "51"))]
pub use crate::delay::Delay;
#[cfg(not(feature = "9160"))]
pub use crate::rng::Rng;
pub use crate::rtc::Rtc;
pub use crate::timer::Timer;

#[cfg(feature = "51")]
pub use crate::adc::Adc;
#[cfg(not(feature = "51"))]
pub use crate::saadc::Saadc;

#[cfg(not(feature = "51"))]
pub use crate::spim::Spim;

#[cfg(not(feature = "51"))]
pub use crate::twim::Twim;

#[cfg(feature = "51")]
pub use crate::uart::Uart;
#[cfg(not(feature = "51"))]
pub use crate::uarte::Uarte;
