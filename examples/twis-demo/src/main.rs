#![no_std]
#![no_main]

use cortex_m::singleton;
use embedded_hal::digital::v2::OutputPin;
use {
    core::{
        panic::PanicInfo,
        sync::atomic::{compiler_fence, Ordering},
    },
    hal::{
        gpio::{p0::Parts, Level, Output, Pin, PushPull},
        pac::TWIS0,
        twis::*,
    },
    nrf52840_hal as hal,
    rtt_target::{rprintln, rtt_init_print},
};

const ADDR0: u8 = 0x1A;
const ADDR1: u8 = 0x1B;
const BUF0_SZ: usize = 8;
const BUF1_SZ: usize = 4;

#[rtic::app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        twis: Twis<TWIS0>,
        buffer0: &'static mut [u8; BUF0_SZ],
        buffer1: &'static mut [u8; BUF1_SZ],
        led: Pin<Output<PushPull>>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        let _clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        rtt_init_print!();

        let p0 = Parts::new(ctx.device.P0);
        let led = p0.p0_06.into_push_pull_output(Level::High).degrade();
        let scl = p0.p0_14.into_floating_input().degrade();
        let sda = p0.p0_16.into_floating_input().degrade();

        let twis = Twis::new(ctx.device.TWIS0, Pins { scl, sda }, ADDR0);
        twis.address1(ADDR1) // Add a secondary i2c address
            .enable_interrupt(TwiEvent::Write) // Trigger interrupt on WRITE command
            .enable_interrupt(TwiEvent::Read) // Trigger interrupt on READ command
            .enable();

        let buffer0 = singleton!(: [u8; BUF0_SZ] = [0; BUF0_SZ]).unwrap();
        let buffer1 = singleton!(: [u8; BUF1_SZ] = [0; BUF1_SZ]).unwrap();
        init::LateResources {
            twis,
            buffer0,
            buffer1,
            led,
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        rprintln!("Waiting for commands from controller...");
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task(binds = SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0, resources = [twis, buffer0, buffer1, led])]
    fn on_twis(ctx: on_twis::Context) {
        let twis = ctx.resources.twis;
        let buffer0 = ctx.resources.buffer0;
        let buffer1 = ctx.resources.buffer1;
        let led = ctx.resources.led;

        if twis.is_event_triggered(TwiEvent::Read) {
            twis.reset_event(TwiEvent::Read);
            led.set_low().ok();
            rprintln!("\nREAD cmd received on addr 0x{:x}", twis.address_match());
            rprintln!("Writing data to controller...");
            match twis.address_match() {
                ADDR0 => {
                    let res = twis.write(&buffer0[..]);
                    rprintln!("Result: {:?}\n{:?}", res, buffer0);
                }
                ADDR1 => {
                    let res = twis.write(&buffer1[..]);
                    rprintln!("Result: {:?}\n{:?}", res, buffer1);
                }
                _ => unreachable!(),
            }
        }
        if twis.is_event_triggered(TwiEvent::Write) {
            twis.reset_event(TwiEvent::Write);
            led.set_high().ok();
            rprintln!("\nWRITE cmd received on addr 0x{:x}", twis.address_match());
            rprintln!("Reading data from controller...");
            match twis.address_match() {
                ADDR0 => {
                    let res = twis.read(&mut buffer0[..]);
                    rprintln!("Result: {:?}\n{:?}", res, buffer0);
                }
                ADDR1 => {
                    let res = twis.read(&mut buffer1[..]);
                    rprintln!("Result: {:?}\n{:?}", res, buffer1);
                }
                _ => unreachable!(),
            }
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
