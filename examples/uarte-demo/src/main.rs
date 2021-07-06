#![no_std]
#![no_main]

use {
    core::{
        str,
        fmt::Write,
        panic::PanicInfo,
        sync::atomic::{compiler_fence, Ordering},
    },
    cortex_m_rt::entry,
    hal::gpio::Level,
    nrf52840_hal as hal,
    rtt_target::{rprintln, rprint, rtt_init_print},
};

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    rprintln!("{}", info);
    loop {
        compiler_fence(Ordering::SeqCst);
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let peripherals = hal::pac::Peripherals::take().unwrap();
    hal::clocks::Clocks::new(peripherals.CLOCK).enable_ext_hfosc();

    let p0 = hal::gpio::p0::Parts::new(peripherals.P0);

    // configure UARTE0
    let uarte0 = peripherals.UARTE0;
    let txd = p0.p0_06.into_push_pull_output(Level::Low).degrade();
    let rxd = p0.p0_08.into_floating_input().degrade();

    let pins = hal::uarte::Pins {
        rxd,
        txd,
        cts: None,
        rts: None,
    };

    let mut serial = hal::uarte::Uarte::new(
        uarte0,
        pins,
        hal::uarte::Parity::EXCLUDED,
        hal::uarte::Baudrate::BAUD115200,
    );

    // duplicating messages over rtt to compare results
    writeln!(serial, "Hello, World!").unwrap();
    rprintln!("Hello, World!");

    let mut rx_buffer = [0u8; 255];
    let mut index = 0;

    // basic serial echoing
    loop {
        // read one byte
        serial.read(&mut rx_buffer[index .. index + 1]).unwrap();
        index += 1;

        // check if we've filled our buffer or a new line byte was sent
        if index == rx_buffer.len() || rx_buffer[index - 1] == '\n' as u8 {
            // write buffer back
            serial.write(&rx_buffer).unwrap();

            // duplicating messages over rtt to compare results
            rprint!(str::from_utf8(&rx_buffer).unwrap());

            // reset the buffer so we don't have stale data
            rx_buffer.iter_mut().for_each(|m| *m = 0);

            index = 0;
        }
    }
}
