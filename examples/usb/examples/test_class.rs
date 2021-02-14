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
    static mut EP_BUF: [u8; 512] = [0; 512];

    let periph = Peripherals::take().unwrap();
    while !periph
        .POWER
        .usbregstatus
        .read()
        .vbusdetect()
        .is_vbus_present()
    {}

    // wait until USB 3.3V supply is stable
    while !periph
        .POWER
        .events_usbpwrrdy
        .read()
        .events_usbpwrrdy()
        .bit_is_clear()
    {}

    let clocks = Clocks::new(periph.CLOCK);
    let clocks = clocks.enable_ext_hfosc();

    let usbd = periph.USBD;
    let usb_bus = Usbd::new_alloc(usbd, EP_BUF, &clocks);

    let mut test = TestClass::new(&usb_bus);

    let mut usb_dev = { test.make_device(&usb_bus) };

    loop {
        if usb_dev.poll(&mut [&mut test]) {
            test.poll();
        }
    }
}
