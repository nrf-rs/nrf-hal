#![no_std]
#![no_main]

// I2S `controller mode` demo
// Generates Morse code audio signals from text, playing back over I2S
// Tested with nRF52840-DK and a UDA1334a DAC

use embedded_hal::digital::v2::InputPin;
use heapless::{
    consts::*,
    spsc::{Consumer, Producer, Queue},
};
use small_morse::{encode, State};
use {
    core::{
        panic::PanicInfo,
        sync::atomic::{compiler_fence, Ordering},
    },
    hal::{
        gpio::{Input, Level, Pin, PullUp},
        gpiote::*,
        i2s::*,
    },
    nrf52840_hal as hal,
    rtic::cyccnt::U32Ext,
    rtt_target::{rprintln, rtt_init_print},
};

#[rtic::app(device = crate::hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        i2s: hal::i2s::I2S,
        #[init([0; 32])]
        signal_buf: [i16; 32],
        #[init([0; 32])]
        mute_buf: [i16; 32],
        #[init(None)]
        queue: Option<Queue<bool, U256>>,
        producer: Producer<'static, bool, U256>,
        consumer: Consumer<'static, bool, U256>,
        gpiote: Gpiote,
        btn1: Pin<Input<PullUp>>,
        btn2: Pin<Input<PullUp>>,
        btn3: Pin<Input<PullUp>>,
    }

    #[init(resources = [queue, signal_buf, mute_buf], spawn = [tick])]
    fn init(mut ctx: init::Context) -> init::LateResources {
        let _clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        // Enable the monotonic timer (CYCCNT)
        ctx.core.DCB.enable_trace();
        ctx.core.DWT.enable_cycle_counter();
        rtt_init_print!();

        let p0 = hal::gpio::p0::Parts::new(ctx.device.P0);
        let btn1 = p0.p0_11.into_pullup_input().degrade();
        let btn2 = p0.p0_12.into_pullup_input().degrade();
        let led1 = p0.p0_13.into_push_pull_output(Level::High).degrade();
        let btn3 = p0.p0_24.into_pullup_input().degrade();
        let btn4 = p0.p0_25.into_pullup_input().degrade();

        let mck_pin = p0.p0_28.into_push_pull_output(Level::Low).degrade();
        let sck_pin = p0.p0_29.into_push_pull_output(Level::Low).degrade();
        let lrck_pin = p0.p0_31.into_push_pull_output(Level::Low).degrade();
        let sdout_pin = p0.p0_30.into_push_pull_output(Level::Low).degrade();

        let i2s = I2S::new_controller(
            ctx.device.I2S,
            Some(&mck_pin),
            &sck_pin,
            &lrck_pin,
            None,
            Some(&sdout_pin),
        );
        i2s.enable_interrupt(I2SEvent::Stopped)
            .tx_buffer(&ctx.resources.mute_buf[..])
            .ok();
        i2s.enable().start();

        let signal_buf = ctx.resources.signal_buf;
        let len = signal_buf.len() / 2;
        for x in 0..len {
            signal_buf[2 * x] = triangle_wave(x as i32, len, 2048, 0, 1) as i16;
            signal_buf[2 * x + 1] = triangle_wave(x as i32, len, 2048, 0, 1) as i16;
        }

        let gpiote = Gpiote::new(ctx.device.GPIOTE);
        gpiote.channel0().output_pin(led1).init_high();
        gpiote.port().input_pin(&btn1).low();
        gpiote.port().input_pin(&btn2).low();
        gpiote.port().input_pin(&btn3).low();
        gpiote.port().input_pin(&btn4).low();
        gpiote.port().enable_interrupt();

        *ctx.resources.queue = Some(Queue::new());
        let (producer, consumer) = ctx.resources.queue.as_mut().unwrap().split();

        ctx.spawn.tick().ok();

        init::LateResources {
            i2s,
            producer,
            consumer,
            gpiote,
            btn1,
            btn2,
            btn3,
        }
    }
    #[idle]
    fn idle(_: idle::Context) -> ! {
        rprintln!("Press a button...");
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task(binds = I2S, resources = [i2s])]
    fn on_i2s(ctx: on_i2s::Context) {
        let i2s = ctx.resources.i2s;
        if i2s.is_event_triggered(I2SEvent::Stopped) {
            i2s.reset_event(I2SEvent::Stopped);
            rprintln!("I2S transmission was stopped");
        }
    }

    #[task(binds = GPIOTE, resources = [gpiote], schedule = [debounce])]
    fn on_gpiote(ctx: on_gpiote::Context) {
        ctx.resources.gpiote.reset_events();
        ctx.schedule.debounce(ctx.start + 3_000_000.cycles()).ok();
    }

    #[task(resources = [gpiote, consumer, i2s, signal_buf, mute_buf], schedule = [tick])]
    fn tick(ctx: tick::Context) {
        let i2s = ctx.resources.i2s;
        if let Some(on) = ctx.resources.consumer.dequeue() {
            if on {
                i2s.tx_buffer(&ctx.resources.signal_buf[..]).ok();
                ctx.resources.gpiote.channel0().clear();
            } else {
                i2s.tx_buffer(&ctx.resources.mute_buf[..]).ok();
                ctx.resources.gpiote.channel0().set();
            }
        } else {
            i2s.tx_buffer(&ctx.resources.mute_buf[..]).ok();
            ctx.resources.gpiote.channel0().set();
        }
        ctx.schedule.tick(ctx.scheduled + 5_000_000.cycles()).ok();
    }

    #[task(resources = [btn1, btn2, btn3, i2s, producer])]
    fn debounce(ctx: debounce::Context) {
        let msg = if ctx.resources.btn1.is_low().unwrap() {
            Some("Radioactivity")
        } else if ctx.resources.btn2.is_low().unwrap() {
            Some("Is in the air for you and me")
        } else {
            None
        };
        if let Some(m) = msg {
            rprintln!("{}", m);
            for action in encode(m) {
                for _ in 0..action.duration {
                    ctx.resources
                        .producer
                        .enqueue(action.state == State::On)
                        .ok();
                }
            }
        }
        if ctx.resources.btn3.is_low().unwrap() {
            ctx.resources.i2s.stop();
        } else {
            ctx.resources.i2s.start();
        }
    }

    extern "C" {
        fn SWI0_EGU0();
        fn SWI1_EGU1();
    }
};

fn triangle_wave(x: i32, length: usize, amplitude: i32, phase: i32, periods: i32) -> i32 {
    let length = length as i32;
    amplitude
        - ((2 * periods * (x + phase + length / (4 * periods)) * amplitude / length)
            % (2 * amplitude)
            - amplitude)
            .abs()
        - amplitude / 2
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    rprintln!("{}", info);
    loop {
        compiler_fence(Ordering::SeqCst);
    }
}
