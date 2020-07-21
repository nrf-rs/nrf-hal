#![no_std]
#![no_main]

// Import the right HAL/PAC crate, depending on the target chip
#[cfg(feature = "52810")]
use nrf52810_hal as hal;
#[cfg(feature = "52832")]
use nrf52832_hal as hal;
#[cfg(feature = "52840")]
use nrf52840_hal as hal;

use {
    bbqueue::{
        consts::*, BBBuffer, ConstBBBuffer,
    },
    core::sync::atomic::AtomicBool,
    hal::{
        gpio::Level,
        pac::{TIMER1, TIMER2},
        ppi::{Parts, Ppi0},
        Timer,
    },
    rtt_target::{rprintln, rtt_init_print},
};

use hal::pac::UARTE0;


// Panic provider crate
use panic_reset as _;

#[rtic::app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        timer: Timer<TIMER1>,

        uarte_timer: hal::bbq_uarte::irq::UarteTimer<TIMER2>,
        uarte_irq: hal::bbq_uarte::irq::UarteIrq<U1024, U1024, Ppi0, UARTE0>,
        uarte_app: hal::bbq_uarte::app::UarteApp<U1024, U1024>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        let _clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();

        let p0 = hal::gpio::p0::Parts::new(ctx.device.P0);

        let uart = ctx.device.UARTE0;

        static UBUF: hal::bbq_uarte::buffer::UarteBuffer<U1024, U1024> =
            hal::bbq_uarte::buffer::UarteBuffer {
                txd_buf: BBBuffer(ConstBBBuffer::new()),
                rxd_buf: BBBuffer(ConstBBBuffer::new()),
                timeout_flag: AtomicBool::new(false),
            };

        rtt_init_print!();

        let rxd = p0.p0_11.into_floating_input().degrade();
        let txd = p0.p0_05.into_push_pull_output(Level::Low).degrade();

        let ppi_channels = Parts::new(ctx.device.PPI);
        let channel0 = ppi_channels.ppi0;

        let uarte_pins = hal::uarte::Pins {
            rxd,
            txd,
            cts: None,
            rts: None,
        };

        let ue = UBUF
            .try_split(
                uarte_pins,
                hal::uarte::Parity::EXCLUDED,
                hal::uarte::Baudrate::BAUD230400,
                ctx.device.TIMER2,
                channel0,
                uart,
                32,
                1_000_000,
            )
            .unwrap();

        init::LateResources {
            timer: Timer::new(ctx.device.TIMER1),
            uarte_timer: ue.timer,
            uarte_irq: ue.irq,
            uarte_app: ue.app,
        }
    }

    #[idle(resources = [timer, uarte_app])]
    fn idle(ctx: idle::Context) -> ! {
        let timer = ctx.resources.timer;
        let uarte_app = ctx.resources.uarte_app;

        use embedded_hal::timer::CountDown;

        rprintln!("Start!");

        timer.start(5_000_000u32);

        loop {
            if let Ok(rgr) = uarte_app.read() {
                let len = rgr.len();
                rprintln!("Brr: {}", len);
                if let Ok(mut wgr) = uarte_app.write_grant(len) {
                    wgr.copy_from_slice(&rgr);
                    wgr.commit(len);
                }
                rgr.release(len);
            }
            if timer.wait().is_ok() {
                rprintln!("Hello from idle!");
                timer.start(5_000_000u32);
            }
        }
    }

    #[task(binds = TIMER2, resources = [uarte_timer])]
    fn timer2(ctx: timer2::Context) {
        // rprintln!("Hello from timer2!");
        ctx.resources.uarte_timer.interrupt();
    }

    #[task(binds = UARTE0_UART0, resources = [uarte_irq])]
    fn uarte0(ctx: uarte0::Context) {
        // rprintln!("Hello from uarte0!");
        ctx.resources.uarte_irq.interrupt();
    }
};
