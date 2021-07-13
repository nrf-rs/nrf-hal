#![no_std]
#![no_main]

// Simple NVMC example

#[cfg(feature = "52840")]
use nrf52840_hal as hal;

use embedded_storage::nor_flash::NorFlash;
use embedded_storage::nor_flash::ReadNorFlash;
use hal::nvmc::Nvmc;
use rtt_target::{rprintln, rtt_init_print};

const CONFIG_SIZE: usize = 1024;
extern "C" {
    #[link_name = "_config"]
    static mut CONFIG: [u32; CONFIG_SIZE];
}

// To run this: `cargo embed --features "52840" --target thumbv7em-none-eabihf`

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();

    let p = hal::pac::Peripherals::take().unwrap();

    #[cfg(feature = "52840")]
    let mut nvmc = Nvmc::new(p.NVMC, unsafe { &mut CONFIG });

    assert!(nvmc.try_erase(0, CONFIG_SIZE as u32 * 4).is_ok());
    let write_buf: [u8; 4] = [1, 2, 3, 4];
    assert!(nvmc.try_write(0, &write_buf).is_ok());
    let mut read_buf = [0u8; 2];
    assert!(nvmc.try_read(0, &mut read_buf).is_ok());
    assert_eq!(read_buf, write_buf[0..2]);

    rprintln!("What was written to flash was read!");

    loop {
        cortex_m::asm::wfe();
    }
}

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
