#![no_std]
#![no_main]

use embedded_hal::digital::v2::InputPin;
use {
    core::{
        panic::PanicInfo,
        sync::atomic::{compiler_fence, Ordering},
    },
    hal::{
        gpio::{Input, Level, Pin, PullUp},
        gpiote::*,
        ppi::{self, ConfigurablePpi, Ppi},
    },
    nrf52840_hal as hal,
    rtic::cyccnt::U32Ext,
    rtt_target::{rprintln, rtt_init_print},
};

#[rtic::app(device = crate::hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        gpiote: Gpiote,
        btn1: Pin<Input<PullUp>>,
        btn3: Pin<Input<PullUp>>,
        btn4: Pin<Input<PullUp>>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> init::LateResources {
        let _clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        rtt_init_print!();
        let p0 = hal::gpio::p0::Parts::new(ctx.device.P0);
        let btn1 = p0.p0_11.into_pullup_input().degrade();
        let btn2 = p0.p0_12.into_pullup_input().degrade();
        let btn3 = p0.p0_24.into_pullup_input().degrade();
        let btn4 = p0.p0_25.into_pullup_input().degrade();
        let led1 = p0.p0_13.into_push_pull_output(Level::High).degrade();

        let gpiote = Gpiote::new(ctx.device.GPIOTE);

        // Set btn1 to generate event on channel 0 and enable interrupt
        gpiote.channel0().input_pin(&btn1).hi_to_lo(true);

        // Set both btn3 & btn4 to generate port event
        gpiote.port().input_pin(&btn3).low();
        gpiote.port().input_pin(&btn4).low();
        // Enable interrupt for port event
        gpiote.port().enable_interrupt();

        // PPI usage, channel 2 event triggers "task out" (toggle) on channel 1 (toggles led1)
        gpiote
            .channel1()
            .output_pin(led1)
            .task_out_polarity(TaskOutPolarity::Toggle)
            .init_high();
        gpiote.channel2().input_pin(&btn2).hi_to_lo(false);
        let ppi_channels = ppi::Parts::new(ctx.device.PPI);
        let mut channel0 = ppi_channels.ppi0;
        channel0.set_task_endpoint(gpiote.channel1().task_out());
        channel0.set_event_endpoint(gpiote.channel2().event());
        channel0.enable();

        // Enable the monotonic timer (CYCCNT)
        ctx.core.DCB.enable_trace();
        ctx.core.DWT.enable_cycle_counter();

        rprintln!("Press a button");

        init::LateResources {
            gpiote,
            btn1,
            btn3,
            btn4,
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task(binds = GPIOTE, resources = [gpiote], schedule = [debounce])]
    fn on_gpiote(ctx: on_gpiote::Context) {
        // Reset all events
        ctx.resources.gpiote.reset_events();
        // Debounce
        ctx.schedule.debounce(ctx.start + 3_000_000.cycles()).ok();
    }

    #[task(resources = [gpiote, btn1, btn3, btn4])]
    fn debounce(ctx: debounce::Context) {
        let btn1_pressed = ctx.resources.btn1.is_low().unwrap();
        let btn3_pressed = ctx.resources.btn3.is_low().unwrap();
        let btn4_pressed = ctx.resources.btn4.is_low().unwrap();

        if btn1_pressed {
            rprintln!("Button 1 was pressed!");
            // Manually run "task out" (toggle) on channel 1 (toggles led1)
            ctx.resources.gpiote.channel1().out();
        }
        if btn3_pressed {
            rprintln!("Button 3 was pressed!");
        }
        if btn4_pressed {
            rprintln!("Button 4 was pressed!");
        }
    }

    extern "C" {
        fn SWI0_EGU0();
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
