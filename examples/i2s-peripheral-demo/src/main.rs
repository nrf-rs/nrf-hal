#![no_std]
#![no_main]

// I2S `peripheral mode` demo
// Signal average level indicator using an RGB LED (APA102 on ItsyBitsy nRF52840)

use embedded_hal::blocking::spi::Write;
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
        rgb: Spim<SPIM0>,
        transfer: Option<Transfer<&'static mut [i16]>>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        static mut RX_BUF: [i16; 128] = [0i16; 128];

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
        i2s.enable_interrupt(I2SEvent::RxPtrUpdated).start();

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
        init::LateResources {
            rgb,
            transfer: i2s.rx(&mut RX_BUF[..]).ok(),
        }
    }

    #[task(binds = I2S, resources = [rgb, transfer])]
    fn on_i2s(ctx: on_i2s::Context) {
        let (rx_buf, i2s) = ctx.resources.transfer.take().unwrap().wait();
        if i2s.is_event_triggered(I2SEvent::RxPtrUpdated) {
            i2s.reset_event(I2SEvent::RxPtrUpdated);
            // Calculate mono summed average of received buffer
            let avg = (rx_buf.iter().map(|x| (*x).abs() as u32).sum::<u32>() / rx_buf.len() as u32)
                as u16;
            let color = match avg {
                0..=4 => &OFF,
                5..=10_337 => &GREEN,
                10_338..=16_383 => &ORANGE,
                _ => &RED,
            };
            <Spim<SPIM0> as Write<u8>>::write(ctx.resources.rgb, color).ok();
        }
        *ctx.resources.transfer = i2s.rx(rx_buf).ok();
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
