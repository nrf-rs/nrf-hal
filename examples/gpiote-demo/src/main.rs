#![no_std]
#![no_main]

use {core::panic::PanicInfo, nrf52840_hal as hal, rtt_target::rprintln};

#[rtic::app(device = crate::hal::pac, peripherals = true, dispatchers = [SWI0_EGU0])]
mod app {
    use embedded_hal::digital::v2::InputPin;
    use systick_monotonic::*;
    use {
        hal::{
            gpio::{Input, Level, Pin, PullUp},
            gpiote::*,
            ppi::{self, ConfigurablePpi, Ppi},
        },
        nrf52840_hal as hal,
        rtt_target::{rprintln, rtt_init_print},
    };

    #[monotonic(binds = SysTick, default = true)]
    type Timer = Systick<1_000_000>;

    #[shared]
    struct Shared {
        gpiote: Gpiote,
    }

    #[local]
    struct Local {
        btn1: Pin<Input<PullUp>>,
        btn3: Pin<Input<PullUp>>,
        btn4: Pin<Input<PullUp>>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
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
        gpiote
            .channel0()
            .input_pin(&btn1)
            .hi_to_lo()
            .enable_interrupt();

        // Set both btn3 & btn4 to generate port event
        gpiote.port().input_pin(&btn3).low();
        gpiote.port().input_pin(&btn4).low();
        // Enable interrupt for port event
        gpiote.port().enable_interrupt();

        // PPI usage, channel 2 event triggers "task out" operation (toggle) on channel 1 (toggles led1)
        gpiote
            .channel1()
            .output_pin(led1)
            .task_out_polarity(TaskOutPolarity::Toggle)
            .init_high();
        gpiote.channel2().input_pin(&btn2).hi_to_lo();
        let ppi_channels = ppi::Parts::new(ctx.device.PPI);
        let mut ppi0 = ppi_channels.ppi0;
        ppi0.set_task_endpoint(gpiote.channel1().task_out());
        ppi0.set_event_endpoint(gpiote.channel2().event());
        ppi0.enable();

        let mono = Systick::new(ctx.core.SYST, 64_000_000);

        rprintln!("Press a button");

        (
            Shared { gpiote },
            Local { btn1, btn3, btn4 },
            init::Monotonics(mono),
        )
    }

    #[task(binds = GPIOTE, shared = [gpiote])]
    fn on_gpiote(mut ctx: on_gpiote::Context) {
        ctx.shared.gpiote.lock(|gpiote| {
            if gpiote.channel0().is_event_triggered() {
                rprintln!("Interrupt from channel 0 event");
            }
            if gpiote.port().is_event_triggered() {
                rprintln!("Interrupt from port event");
            }
            // Reset all events
            gpiote.reset_events();
            // Debounce
            debounce::spawn_after(50.millis()).ok();
        });
    }

    #[task(shared = [gpiote], local = [btn1, btn3, btn4])]
    fn debounce(mut ctx: debounce::Context) {
        let btn1_pressed = ctx.local.btn1.is_low().unwrap();
        let btn3_pressed = ctx.local.btn3.is_low().unwrap();
        let btn4_pressed = ctx.local.btn4.is_low().unwrap();

        ctx.shared.gpiote.lock(|gpiote| {
            if btn1_pressed {
                rprintln!("Button 1 was pressed!");
                // Manually run "task out" operation (toggle) on channel 1 (toggles led1)
                gpiote.channel1().out();
            }
            if btn3_pressed {
                rprintln!("Button 3 was pressed!");
                // Manually run "task clear" on channel 1 (led1 on)
                gpiote.channel1().clear();
            }
            if btn4_pressed {
                rprintln!("Button 4 was pressed!");
                // Manually run "task set" on channel 1 (led1 off)
                gpiote.channel1().set();
            }
        });
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    rprintln!("{}", info);
    loop {}
}
