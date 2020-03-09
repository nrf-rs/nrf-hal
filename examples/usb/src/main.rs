#![no_std]
#![no_main]

use panic_semihosting as _;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nrf52840_hal::usb::Usb;
use nrf52840_pac::Peripherals;
use usb_device::device::{UsbDeviceBuilder, UsbDeviceState, UsbVidPid};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[entry]
fn main() -> ! {
    static mut EP_BUF: [u8; 256] = [0; 256];

    let periph = Peripherals::take().unwrap();
    while !periph
        .POWER
        .usbregstatus
        .read()
        .vbusdetect()
        .is_vbus_present()
    {}

    let usb_bus = Usb::new_alloc(periph.USBD, EP_BUF);
    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .product("nRF52840 Serial Port Demo")
        .device_class(USB_CLASS_CDC)
        .build();

    hprintln!("<start>").ok();
    let mut state = UsbDeviceState::Default;
    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        let new_state = usb_dev.state();
        if new_state != state {
            hprintln!("{:?} {:#x}", new_state, usb_dev.bus().device_address()).ok();
            state = new_state;
        }
    }
}
