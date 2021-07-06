#![no_std]
#![no_main]

use panic_semihosting as _;

use cortex_m_rt::entry;
use nrf52840_hal::usbd::Usbd;
use nrf52840_hal::clocks::Clocks;
use nrf52840_pac::Peripherals;
use usb_device::test_class::TestClass;

#[entry]
fn main() -> ! {
    let periph = Peripherals::take().unwrap();
    let clocks = Clocks::new(periph.CLOCK);
    let clocks = clocks.enable_ext_hfosc();

    let usb_bus = Usbd::new(periph.USBD, &clocks);

    let mut test = TestClass::new(&usb_bus);

    let mut usb_dev = { test.make_device(&usb_bus) };

    loop {
        if usb_dev.poll(&mut [&mut test]) {
            test.poll();
        }
    }
}
