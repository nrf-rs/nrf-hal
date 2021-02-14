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

    let mut led = p0.p0_23.into_push_pull_output(Level::High);
    let btn = p1.p1_00.into_pullup_input();
    while btn.is_high().unwrap() {}

    timer.enable_interrupt();
    timer.start(Timer::<TIMER0, OneShot>::TICKS_PER_SECOND * 3);

    led.set_low().unwrap();

    let usb_bus = Usbd::new_alloc(usbd, EP_BUF, &clocks);
    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .product("nRF52840 Serial Port Demo")
        .device_class(USB_CLASS_CDC)
        .max_packet_size_0(64) // (makes control transfers 8x faster)
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
