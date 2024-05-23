#![no_std]
#![no_main]

use defmt_rtt as _;
use nrf52840_hal as _;
use panic_probe as _;

use core::ptr::addr_of_mut;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use nrf52840_hal::{nvmc::Nvmc, pac};

const NUM_PAGES: u32 = 6; // must match memory.x
const PAGE_SIZE: u32 = 4 * 1024;
const LAST_PAGE: u32 = (NUM_PAGES - 1) * PAGE_SIZE;
const CONFIG_SIZE: u32 = NUM_PAGES * PAGE_SIZE;
extern "C" {
    #[link_name = "_config"]
    static mut CONFIG: [u8; CONFIG_SIZE as usize];
}

struct State {
    nvmc: Nvmc<pac::NVMC>,
}

#[defmt_test::tests]
mod tests {
    use defmt::{assert, unwrap};

    use super::*;

    #[init]
    fn init() -> State {
        let p = unwrap!(pac::Peripherals::take());

        State {
            nvmc: Nvmc::new(p.NVMC, unsafe { addr_of_mut!(CONFIG).as_mut().unwrap() }),
        }
    }

    #[test]
    fn check_capacity(state: &mut State) {
        assert_eq!(state.nvmc.capacity(), CONFIG_SIZE as usize);
    }

    #[test]
    fn read_outofbounds(state: &mut State) {
        assert!(state.nvmc.read(CONFIG_SIZE, &mut [0]).is_err());
        assert!(state.nvmc.read(CONFIG_SIZE - 1, &mut [0, 0]).is_err());
    }

    #[test]
    fn erase_unaligned(state: &mut State) {
        assert!(state.nvmc.erase(LAST_PAGE + 1, PAGE_SIZE).is_err());
        assert!(state.nvmc.erase(LAST_PAGE, PAGE_SIZE + 1).is_err());
    }

    #[test]
    fn erase_outofbounds(state: &mut State) {
        assert!(state
            .nvmc
            .erase(CONFIG_SIZE, CONFIG_SIZE + PAGE_SIZE)
            .is_err());
        assert!(state
            .nvmc
            .erase(LAST_PAGE, LAST_PAGE + 2 * PAGE_SIZE)
            .is_err());
    }

    #[test]
    fn write_unaligned(state: &mut State) {
        let buf = [0u8; 4];
        assert!(state.nvmc.write(LAST_PAGE + 1, &buf).is_err());
        assert!(state.nvmc.write(LAST_PAGE, &buf[..1]).is_err());
    }

    #[test]
    fn read_write_and_then_read(state: &mut State) {
        assert!(state.nvmc.erase(LAST_PAGE, CONFIG_SIZE).is_ok());
        let mut read_buf = [0];
        assert!(state.nvmc.read(LAST_PAGE, &mut read_buf).is_ok());
        assert_eq!(read_buf[0], 0xff);
        let write_buf = [1, 2, 3, 4];
        assert!(state.nvmc.write(LAST_PAGE, &write_buf).is_ok());
        assert!(state.nvmc.read(LAST_PAGE, &mut read_buf).is_ok());
        assert_eq!(read_buf[0], 1);
    }

    #[test]
    fn read_what_is_written(state: &mut State) {
        assert!(state.nvmc.erase(LAST_PAGE, CONFIG_SIZE).is_ok());
        let write_buf: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        assert!(state.nvmc.write(LAST_PAGE, &write_buf).is_ok());
        let mut read_buf = [0u8; 8];
        assert!(state.nvmc.read(LAST_PAGE, &mut read_buf).is_ok());
        assert_eq!(read_buf, write_buf);
        let mut partial_read_buf = [0u8; 4];
        assert!(state
            .nvmc
            .read(LAST_PAGE + 2, &mut partial_read_buf)
            .is_ok());
        assert_eq!(partial_read_buf, write_buf[2..][..4]);
    }
}
