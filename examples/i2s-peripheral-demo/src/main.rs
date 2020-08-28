#![no_std]
#![no_main]

// I2S `peripheral mode` demo
// RMS level indicator using an RGB LED (APA102 on ItsyBitsy nRF52840)

use embedded_hal::blocking::spi::Write;
use m::Float;
use {
    core::{
        panic::PanicInfo,
        sync::atomic::{compiler_fence, Ordering},
    },
    hal::{
        gpio::Level,
        i2s::*,
        pac::SPIM0,
        spim::{Frequency, Mode as SPIMode, Phase, Pins, Polarity, Spim},
    },
    nrf52840_hal as hal,
    rtt_target::{rprintln, rtt_init_print},
};

const OFF: [u8; 9] = [0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0xFF];
const GREEN: [u8; 9] = [0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x10, 0x00, 0xFF];
const ORANGE: [u8; 9] = [0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x10, 0x10, 0xFF];
const RED: [u8; 9] = [0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x10, 0xFF];

#[rtic::app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        i2s: I2S,
        #[init([0; 128])]
        rx_buf: [i16; 128],
        rgb: Spim<SPIM0>,
    }

    #[init(resources = [rx_buf])]
    fn init(ctx: init::Context) -> init::LateResources {
        let _clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        rtt_init_print!();
        rprintln!("Play me some audio...");

        let p0 = hal::gpio::p0::Parts::new(ctx.device.P0);
        let mck_pin = p0.p0_25.into_floating_input().degrade();
        let sck_pin = p0.p0_24.into_floating_input().degrade();
        let lrck_pin = p0.p0_16.into_floating_input().degrade();
        let sdin_pin = p0.p0_14.into_floating_input().degrade();

        // Configure I2S reception
        let i2s = I2S::new_peripheral(
            ctx.device.I2S,
            Some(&mck_pin),
            &sck_pin,
            &lrck_pin,
            Some(&sdin_pin),
            None,
        );
        i2s.enable_interrupt(I2SEvent::RxPtrUpdated)
            .rx_buffer(&mut ctx.resources.rx_buf[..])
            .ok();
        i2s.enable().start();

        // Configure APA102 RGB LED control
        let p1 = hal::gpio::p1::Parts::new(ctx.device.P1);
        let rgb_data_pin = p0.p0_08.into_push_pull_output(Level::Low).degrade();
        let rgb_clk_pin = p1.p1_09.into_push_pull_output(Level::Low).degrade();

        let rgb = Spim::new(
            ctx.device.SPIM0,
            Pins {
                miso: None,
                mosi: Some(rgb_data_pin),
                sck: rgb_clk_pin,
            },
            Frequency::M4,
            SPIMode {
                polarity: Polarity::IdleLow,
                phase: Phase::CaptureOnFirstTransition,
            },
            0,
        );

        init::LateResources { i2s, rgb }
    }

    #[task(binds = I2S, resources = [i2s, rx_buf, rgb])]
    fn on_i2s(ctx: on_i2s::Context) {
        let on_i2s::Resources { i2s, rx_buf, rgb } = ctx.resources;
        if i2s.is_event_triggered(I2SEvent::RxPtrUpdated) {
            i2s.reset_event(I2SEvent::RxPtrUpdated);
            // Calculate mono summed RMS of received buffer
            let rms = Float::sqrt(
                (rx_buf.iter().map(|x| *x as i32).map(|x| x * x).sum::<i32>() / rx_buf.len() as i32)
                    as f32,
            ) as u16;
            let color = match rms {
                0..=4 => &OFF,
                5..=10_337 => &GREEN,
                10_338..=16_383 => &ORANGE,
                _ => &RED,
            };
            <Spim<SPIM0> as Write<u8>>::write(rgb, color).ok();
        }
    }
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
