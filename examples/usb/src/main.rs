#![no_std]
#![no_main]

use panic_semihosting as _;

use cortex_m_rt::entry;
use nrf52840_hal::prelude::*;
use nrf52840_hal::timer::{OneShot, Timer};
use nrf52840_hal::usbd::Usbd;
use nrf52840_hal::clocks::Clocks;
use nrf52840_pac::{Peripherals, TIMER0};
use usb_device::device::{UsbDeviceBuilder, UsbDeviceState, UsbVidPid};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[entry]
fn main() -> ! {
    let periph = Peripherals::take().unwrap();
    let clocks = Clocks::new(periph.CLOCK);
    let clocks = clocks.enable_ext_hfosc();

    let mut timer = Timer::periodic(periph.TIMER0);
    timer.start(Timer::<TIMER0, OneShot>::TICKS_PER_SECOND);

    let usb_bus = Usbd::new(periph.USBD, &clocks);
    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .product("nRF52840 Serial Port Demo")
        .device_class(USB_CLASS_CDC)
        .max_packet_size_0(64) // (makes control transfers 8x faster)
        .build();

    loop {
        usb_dev.poll(&mut [&mut serial]);

        if usb_dev.state() == UsbDeviceState::Configured && serial.dtr() {
            if timer.wait().is_ok() {
                serial.write(b"Hello, world!\n").ok();
            }
        }
    }
}
