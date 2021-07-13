#![no_std]
#![no_main]

use defmt_rtt as _;
use nrf52840_hal as _;
use panic_probe as _;

use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use nrf52840_hal::{nvmc::Nvmc, pac};

const CONFIG_SIZE: usize = 1024;
extern "C" {
    #[link_name = "_config"]
    static mut CONFIG: [u32; CONFIG_SIZE];
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
            nvmc: Nvmc::new(p.NVMC, unsafe { &mut CONFIG }),
        }
    }

    #[test]
    fn check_capacity(state: &mut State) {
        assert_eq!(state.nvmc.capacity(), CONFIG_SIZE * 4);
    }

    #[test]
    fn read_unaligned(state: &mut State) {
        let mut buf = [0u8; 1];
        assert!(state.nvmc.try_read(1, &mut buf).is_err());
    }

    #[test]
    fn read_beyond_buffer(state: &mut State) {
        let mut buf = [0u8; CONFIG_SIZE * 4 + 1];
        assert!(state.nvmc.try_read(0, &mut buf).is_err());
    }

    #[test]
    fn erase_unaligned_from(state: &mut State) {
        assert!(state.nvmc.try_erase(1, 4096).is_err());
    }

    #[test]
    fn erase_unaligned_to(state: &mut State) {
        assert!(state.nvmc.try_erase(0, 4097).is_err());
    }

    #[test]
    fn write_unaligned(state: &mut State) {
        let buf = [0u8; 1];
        assert!(state.nvmc.try_write(1, &buf).is_err());
    }

    #[test]
    fn read_write_and_then_read(state: &mut State) {
        assert!(state.nvmc.try_erase(0, CONFIG_SIZE as u32 * 4).is_ok());
        let mut read_buf = [0u8; 1];
        assert!(state.nvmc.try_read(0, &mut read_buf).is_ok());
        assert_eq!(read_buf[0], 0xff);
        let write_buf = [1u8; 4];
        assert!(state.nvmc.try_write(0, &write_buf).is_ok());
        assert!(state.nvmc.try_read(0, &mut read_buf).is_ok());
        assert_eq!(read_buf[0], 0x1);
    }

    #[test]
    fn read_what_is_written(state: &mut State) {
        assert!(state.nvmc.try_erase(0, CONFIG_SIZE as u32 * 4).is_ok());
        let write_buf: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        assert!(state.nvmc.try_write(0, &write_buf).is_ok());
        let mut read_buf = [0u8; 8];
        assert!(state.nvmc.try_read(0, &mut read_buf).is_ok());
        assert_eq!(read_buf, write_buf);
    }

    #[test]
    fn partially_read_what_is_written(state: &mut State) {
        assert!(state.nvmc.try_erase(0, CONFIG_SIZE as u32 * 4).is_ok());
        let write_buf: [u8; 4] = [1, 2, 3, 4];
        assert!(state.nvmc.try_write(0, &write_buf).is_ok());
        let mut read_buf = [0u8; 2];
        assert!(state.nvmc.try_read(0, &mut read_buf).is_ok());
        assert_eq!(read_buf, write_buf[0..2]);
    }
}
