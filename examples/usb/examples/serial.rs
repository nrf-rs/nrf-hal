#![no_std]
#![no_main]

use panic_semihosting as _;

use cortex_m::peripheral::SCB;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nrf52840_hal::gpio::{p0, p1, Level};
use nrf52840_hal::prelude::*;
use nrf52840_hal::timer::{OneShot, Timer};
use nrf52840_hal::usbd::Usbd;
use nrf52840_hal::clocks::Clocks;
use nrf52840_pac::{interrupt, Peripherals, TIMER0};
use usb_device::device::{UsbDeviceBuilder, UsbDeviceState, UsbVidPid};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[interrupt]
fn TIMER0() {
    SCB::sys_reset();
}

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

    let mut timer = Timer::one_shot(periph.TIMER0);
    let usbd = periph.USBD;
    let p0 = p0::Parts::new(periph.P0);
    let p1 = p1::Parts::new(periph.P1);

    //let mut led = p0.p0_23.into_push_pull_output(Level::High);
    let btn = p1.p1_00.into_pullup_input();
    while btn.is_high().unwrap() {}

    let mut led = p0.p0_13.into_push_pull_output(Level::High);

    let usb_bus = Usbd::new_alloc(usbd, EP_BUF, &clocks);
    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .max_packet_size_0(64) // (makes control transfers 8x faster)
        .build();

    hprintln!("<start>").ok();
    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        let mut buf = [0u8; 64];

        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                led.set_low().ok(); // Turn on

                // Echo back in upper case
                for c in buf[0..count].iter_mut() {
                    if 0x61 <= *c && *c <= 0x7a {
                        *c &= !0x20;
                    }
                }

                let mut write_offset = 0;
                while write_offset < count {
                    match serial.write(&buf[write_offset..count]) {
                        Ok(len) if len > 0 => {
                            write_offset += len;
                        },
                        _ => {},
                    }
                }
            }
            _ => {}
        }

        led.set_high().ok(); // Turn off
    }
}
