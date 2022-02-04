#![no_std]
#![no_main]

// Simple NVMC example

#[cfg(feature = "52840")]
use nrf52840_hal as hal;

use core::convert::TryInto;
use embedded_storage::nor_flash::NorFlash;
use embedded_storage::nor_flash::ReadNorFlash;
use hal::nvmc::Nvmc;
use hal::pac::NVMC;
use panic_probe as _;
use rtt_target::{rprintln, rtt_init_print};

const NUM_PAGES: u32 = 6;
const PAGE_SIZE: u32 = 4 * 1024;
const LAST_PAGE: u32 = (NUM_PAGES - 1) * PAGE_SIZE;
extern "C" {
    #[link_name = "_config"]
    static mut CONFIG: [u8; (NUM_PAGES * PAGE_SIZE) as usize];
}

// To run this example:
// cargo build --features=52840 --target=thumbv7em-none-eabi && \
// probe-run --chip nRF52840_xxAA ../../target/thumbv7em-none-eabi/debug/nvmc-demo

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();

    let p = hal::pac::Peripherals::take().unwrap();

    #[cfg(feature = "52840")]
    let mut nvmc = Nvmc::new(p.NVMC, unsafe { &mut CONFIG });

    erase_if_needed(&mut nvmc, LAST_PAGE);

    let mut write_buf: [u8; 32] = [0; 32];
    for (i, x) in write_buf.iter_mut().enumerate() {
        *x = i as u8;
    }
    rprintln!("Writing at {:#x}: {:02x?}", LAST_PAGE, write_buf);
    nvmc.write(LAST_PAGE, &write_buf).unwrap();

    for i in 0..4 {
        compare_read::<11>(&mut nvmc, LAST_PAGE + i);
        compare_read::<10>(&mut nvmc, LAST_PAGE + i);
        compare_read::<9>(&mut nvmc, LAST_PAGE + i + 4);
        compare_read::<8>(&mut nvmc, LAST_PAGE + i + 4);
        compare_read::<7>(&mut nvmc, LAST_PAGE + i + 8);
        compare_read::<6>(&mut nvmc, LAST_PAGE + i + 8);
        compare_read::<5>(&mut nvmc, LAST_PAGE + i + 16);
        compare_read::<4>(&mut nvmc, LAST_PAGE + i + 16);
        compare_read::<3>(&mut nvmc, LAST_PAGE + i + 20);
        compare_read::<2>(&mut nvmc, LAST_PAGE + i + 20);
        compare_read::<1>(&mut nvmc, LAST_PAGE + i + 24);
    }

    erase_if_needed(&mut nvmc, LAST_PAGE);

    loop {
        cortex_m::asm::wfe();
    }
}

fn compare_read<const LENGTH: usize>(nvmc: &mut Nvmc<NVMC>, offset: u32) {
    let actual = read::<LENGTH>(nvmc, offset);
    let expected = unsafe { direct_read::<LENGTH>(offset) };
    if actual == expected {
        rprintln!("Read at {:#x}: {:02x?} as expected", offset, actual);
    } else {
        rprintln!(
            "Error: Read at {:#x}: {:02x?} instead of {:02x?}",
            offset,
            actual,
            expected,
        );
    }
}

fn read<const LENGTH: usize>(nvmc: &mut Nvmc<NVMC>, offset: u32) -> [u8; LENGTH] {
    let mut buf = [0; LENGTH];
    nvmc.read(offset, &mut buf).unwrap();
    buf
}

unsafe fn direct_read<const LENGTH: usize>(offset: u32) -> [u8; LENGTH] {
    CONFIG[offset as usize..][..LENGTH].try_into().unwrap()
}

fn erase_if_needed(nvmc: &mut Nvmc<NVMC>, offset: u32) {
    let mut page = [0; PAGE_SIZE as usize];
    nvmc.read(offset, &mut page).unwrap();
    if page_is_erased(&page) {
        return;
    }
    rprintln!("Erasing at {:#x}", offset);
    nvmc.erase(offset, offset + PAGE_SIZE).unwrap();
    nvmc.read(offset, &mut page).unwrap();
    if page_is_erased(&page) {
        rprintln!("The page was correctly erased.");
    } else {
        rprintln!("Error: The page was not correctly erased.");
    }
}

fn page_is_erased(page: &[u8; PAGE_SIZE as usize]) -> bool {
    page.iter().all(|&x| x == 0xff)
}
