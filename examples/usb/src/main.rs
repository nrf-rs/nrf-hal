#![no_std]
#![no_main]

// use panic_semihosting as _;

use cortex_m_rt::entry;
use nrf52840_hal::gpio::{p0, p1, Level};
use nrf52840_hal::prelude::*;
use nrf52840_hal::usbd::Usbd;
use nrf52840_hal::clocks::Clocks;
use nrf52840_pac::Peripherals;
use nrf52840_pac::interrupt;
use usb_device::device::{UsbDeviceBuilder, UsbDeviceState, UsbVidPid};
use usb_device::UsbError;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use core::cell::RefCell;
use core::fmt::Write;
use core::ops::DerefMut;
use core::panic::PanicInfo;
use cortex_m::interrupt::{free, Mutex};
pub type SafeSerial = Mutex<RefCell<Option<nrf52840_hal::uarte::Uarte<nrf52840_hal::pac::UARTE0>>>>;
pub static SERIAL: SafeSerial = Mutex::new(RefCell::new(None));

#[interrupt]
fn USBD() {
    // SCB::sys_reset();
    // free(|cs| {
    //     if let Some(ref mut s) = SERIAL.borrow(cs).borrow_mut().deref_mut() {
    //         writeln!(s, "interrupt!").unwrap();
    //     }
    // });
}

#[entry]
fn main() -> ! {
    static mut EP_BUF: [u8; 1024] = [0; 1024];

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
    unsafe {
        usbd.inten.write(|w| { w.bits(0xFF) });
    }
    let p0 = p0::Parts::new(periph.P0);
    let _p1 = p1::Parts::new(periph.P1);

    // To remove
    let serial_uart = nrf52840_hal::Uarte::new(
        periph.UARTE0,
        nrf52840_hal::uarte::Pins {
            txd: p0.p0_06.into_push_pull_output(Level::High).degrade(),
            rxd: p0.p0_08.into_floating_input().degrade(),
            cts: None,
            rts: None,
        },
        nrf52840_hal::uarte::Parity::EXCLUDED,
        nrf52840_hal::uarte::Baudrate::BAUD115200,
    );

    free(|cs| {
        SERIAL.borrow(cs).replace(Some(serial_uart));
    });

    free(|cs| {
        if let Some(ref mut s) = SERIAL.borrow(cs).borrow_mut().deref_mut() {
            writeln!(s, "Initialization complete!").unwrap();
        }
    });

    let usb_bus = Usbd::new_alloc(usbd, EP_BUF, &clocks);
    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .product("nRF52840 Serial Port Demo")
        .device_class(USB_CLASS_CDC)
        .max_packet_size_0(64) // (makes control transfers 8x faster)
        .build();

    let mut state = UsbDeviceState::Default;
    loop {
        // free(|cs| {
        //     if let Some(ref mut s) = SERIAL.borrow(cs).borrow_mut().deref_mut() {
        //         writeln!(s, "loop").unwrap();
        //     }
        // });

        let new_state = usb_dev.state();
        if new_state != state {
            free(|cs| {
                if let Some(ref mut s) = SERIAL.borrow(cs).borrow_mut().deref_mut() {
                    writeln!(s, "State: {:?} -> {:?}", state, new_state).unwrap();
                }
            });
            state = new_state;
        }

        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        let mut buf = [0u8; 64];

        match serial.read(&mut buf) {
            Ok(_d) => {0},
            Err(UsbError::WouldBlock) => {0},
            Err(e) => Err(e).unwrap(),
        };
        match serial.write(&[0x3a, 0x29]) {
            Ok(_d) => {0},
            Err(UsbError::WouldBlock) => {0},
            Err(e) => Err(e).unwrap(),
        };

    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    free(|cs| {
        if let Some(ref mut s) = SERIAL.borrow(cs).borrow_mut().deref_mut() {
            writeln!(s, "{}", info).ok();
        }
    });

    loop {}
}
