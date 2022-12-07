#![no_main]
#![no_std]

use core::{
    panic::PanicInfo,
    sync::atomic::{compiler_fence, Ordering},
};

use nrf52840_hal as hal;
use hal::{
    pdm::{self, Pdm},
    clocks::Clocks,
    gpio::Level,
};
use rtt_target::{rprintln, rtt_init_print};

// The lenght of the buffer allocated to the pdm samples (in mono mode it is
// exactly the number of samples per read)
const PDM_BUFFER_LEN: usize = 8192;

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello, world!");

    let peripherals = hal::pac::Peripherals::take().unwrap();
    let port0 = hal::gpio::p0::Parts::new(peripherals.P0);

    // Enable the high frequency oscillator if not already enabled
    let _clocks = Clocks::new(peripherals.CLOCK).enable_ext_hfosc();

    // Retreive the right pins used in the Arduino Nano 33 BLE Sense board
    let _mic_vcc = port0.p0_17.into_push_pull_output(Level::High);
    let mic_clk = port0.p0_26.into_push_pull_output(Level::Low).degrade();
    let mic_din = port0.p0_25.into_floating_input().degrade();
    
    let pdm = Pdm::new(peripherals.PDM, mic_clk, mic_din);
    pdm.sampling(pdm::Sampling::LEFTFALLING)
        .channel(pdm::Channel::MONO)
        .frequency(pdm::Frequency::_1280K)
        .left_gain(pdm::GainL::MAXGAIN)
        .enable();

    // Allocate the buffer
    let mut buffer = [0; PDM_BUFFER_LEN];
    
    // Skip a few samples as suggested by the nrf-52840 docs
    for i in 0..10 {
        rprintln!("{}", i);
        pdm.read(&mut buffer);
    }

    
    // Output the power of the received signal
    loop {
        // Ask the pdm peripheral to fill our buffer with samples
        pdm.read(&mut buffer);

        let square_sum = buffer.iter().fold(0i64, |sum, &item| {
            sum + (item as i64).pow(2)
        });
        rprintln!("Energy : {}", square_sum as f32 / PDM_BUFFER_LEN as f32);

        for _ in 0..10_000 {}
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    rprintln!("{}", info);
    loop {
        compiler_fence(Ordering::SeqCst);
    }
}