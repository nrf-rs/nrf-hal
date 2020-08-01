//! Implementation details of the nRF HAL crates. Don't use this directly, use one of the specific
//! HAL crates instead (`nrfXYZ-hal`).

#![no_std]

use embedded_hal as hal;

#[cfg(feature = "51")]
pub use nrf51 as pac;

#[cfg(feature = "52810")]
pub use nrf52810_pac as pac;

#[cfg(feature = "52832")]
pub use nrf52832_pac as pac;

#[cfg(feature = "52833")]
pub use nrf52833_pac as pac;

#[cfg(feature = "52840")]
pub use nrf52840_pac as pac;

#[cfg(feature = "9160")]
pub use nrf9160_pac as pac;

#[cfg(feature = "51")]
pub mod adc;
#[cfg(not(feature = "9160"))]
pub mod ccm;
pub mod clocks;
#[cfg(not(any(feature = "51", feature = "9160")))]
pub mod comp;
#[cfg(not(feature = "51"))]
pub mod delay;
#[cfg(not(feature = "9160"))]
pub mod ecb;
pub mod gpio;
#[cfg(not(feature = "9160"))]
pub mod gpiote;
#[cfg(not(feature = "9160"))]
pub mod ppi;
#[cfg(not(feature = "9160"))]
pub mod rng;
pub mod rtc;
#[cfg(not(feature = "51"))]
pub mod saadc;
#[cfg(feature = "51")]
pub mod spi;
#[cfg(not(feature = "51"))]
pub mod spim;
#[cfg(not(feature = "9160"))]
pub mod temp;
pub mod time;
pub mod timer;
#[cfg(feature = "51")]
pub mod twi;
#[cfg(not(feature = "51"))]
pub mod twim;
#[cfg(feature = "51")]
pub mod uart;
#[cfg(not(feature = "51"))]
pub mod uarte;
#[cfg(not(feature = "9160"))]
pub mod uicr;
#[cfg(not(feature = "9160"))]
pub mod wdt;

pub mod prelude {
    pub use crate::hal::digital::v2::*;
    pub use crate::hal::prelude::*;

    #[cfg(not(feature = "9160"))]
    pub use crate::ppi::{ConfigurablePpi, Ppi};
    pub use crate::time::U32Ext;
}

/// Length of Nordic EasyDMA differs for MCUs
#[cfg(any(feature = "52810", feature = "52832", feature = "51"))]
pub mod target_constants {
    // NRF52832 8 bits1..0xFF
    pub const EASY_DMA_SIZE: usize = 255;
    // Easy DMA can only read from data ram
    pub const SRAM_LOWER: usize = 0x2000_0000;
    pub const SRAM_UPPER: usize = 0x3000_0000;
    pub const FORCE_COPY_BUFFER_SIZE: usize = 255;
    const _CHECK_FORCE_COPY_BUFFER_SIZE: usize = EASY_DMA_SIZE - FORCE_COPY_BUFFER_SIZE;
    // ERROR: FORCE_COPY_BUFFER_SIZE must be <= EASY_DMA_SIZE
}
#[cfg(any(feature = "52840", feature = "52833", feature = "9160"))]
pub mod target_constants {
    // NRF52840 and NRF9160 16 bits 1..0xFFFF
    pub const EASY_DMA_SIZE: usize = 65535;
    // Limits for Easy DMA - it can only read from data ram
    pub const SRAM_LOWER: usize = 0x2000_0000;
    pub const SRAM_UPPER: usize = 0x3000_0000;
    pub const FORCE_COPY_BUFFER_SIZE: usize = 1024;
    const _CHECK_FORCE_COPY_BUFFER_SIZE: usize = EASY_DMA_SIZE - FORCE_COPY_BUFFER_SIZE;
    // ERROR: FORCE_COPY_BUFFER_SIZE must be <= EASY_DMA_SIZE
}

/// Does this slice reside entirely within RAM?
pub(crate) fn slice_in_ram(slice: &[u8]) -> bool {
    let ptr = slice.as_ptr() as usize;
    ptr >= target_constants::SRAM_LOWER && (ptr + slice.len()) < target_constants::SRAM_UPPER
}

/// Return an error if slice is not in RAM.
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

#[cfg(feature = "51")]
pub use crate::spi::Spi;
#[cfg(not(feature = "51"))]
pub use crate::spim::Spim;

#[cfg(feature = "51")]
pub use crate::twi::Twi;
#[cfg(not(feature = "51"))]
pub use crate::twim::Twim;

#[cfg(feature = "51")]
pub use crate::uart::Uart;
#[cfg(not(feature = "51"))]
pub use crate::uarte::Uarte;
