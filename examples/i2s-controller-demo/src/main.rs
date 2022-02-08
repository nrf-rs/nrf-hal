#![no_std]
#![no_main]

// I2S `controller mode` demo
// Generates Morse code audio signals for text from UART, playing back over I2S
// Tested with nRF52840-DK and a UDA1334a DAC

use {core::panic::PanicInfo, nrf52840_hal as hal, rtt_target::rprintln};

#[repr(align(4))]
pub struct Aligned<T: ?Sized>(T);

#[rtic::app(device = crate::hal::pac, peripherals = true, dispatchers = [SWI0_EGU0, SWI1_EGU1])]
mod app {
    use crate::{hal, triangle_wave, Aligned};
    use embedded_hal::digital::v2::{InputPin, OutputPin};
    use heapless::spsc::{Consumer, Producer, Queue};
    use small_morse::{encode, State};
    use systick_monotonic::*;
    use {
        hal::{
            gpio::{Input, Level, Output, Pin, PullUp, PushPull},
            gpiote::*,
            i2s::{self, *},
            pac::{TIMER0, UARTE0},
            timer::Timer,
            uarte::{self, *},
        },
        rtt_target::{rprintln, rtt_init_print},
    };

    #[monotonic(binds = SysTick, default = true)]
    type Mono = Systick<1_000_000>;

    #[shared]
    struct Shared {
        speed: u32,
    }

    #[local]
    struct Local {
        signal_buf: &'static [i16; 32],
        mute_buf: &'static [i16; 32],
        producer: Producer<'static, State, 257>,
        consumer: Consumer<'static, State, 257>,
        uarte: Uarte<UARTE0>,
        uarte_timer: Timer<TIMER0>,
        gpiote: Gpiote,
        btn1: Pin<Input<PullUp>>,
        btn2: Pin<Input<PullUp>>,
        led: Pin<Output<PushPull>>,
        transfer: Option<Transfer<&'static [i16; 32]>>,
    }

    #[init(local = [
        // The I2S buffer address must be 4 byte aligned.
        static_mute_buf: Aligned<[i16; 32]> = Aligned([0i16; 32]),
        static_signal_buf: Aligned<[i16; 32]> = Aligned([0i16; 32]),
        queue: Option<Queue<State, 257>> = None,
    ])]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let signal_buf = ctx.local.static_signal_buf;
        let mute_buf = ctx.local.static_mute_buf;

        // Fill signal buffer with triangle waveform, 2 channels interleaved
        let len = signal_buf.0.len() / 2;
        for x in 0..len {
            signal_buf.0[2 * x] = triangle_wave(x as i32, len, 2048, 0, 1) as i16;
            signal_buf.0[2 * x + 1] = triangle_wave(x as i32, len, 2048, 0, 1) as i16;
        }

        let _clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        // Enable the monotonic timer (CYCCNT)
        ctx.core.DCB.enable_trace();
        ctx.core.DWT.enable_cycle_counter();
        rtt_init_print!();

        let p0 = hal::gpio::p0::Parts::new(ctx.device.P0);

        // Configure I2S controller
        let i2s = I2S::new(
            ctx.device.I2S,
            i2s::Pins::Controller {
                mck: Some(p0.p0_28.into_push_pull_output(Level::Low).degrade()),
                sck: p0.p0_29.into_push_pull_output(Level::Low).degrade(),
                lrck: p0.p0_31.into_push_pull_output(Level::Low).degrade(),
                sdin: None,
                sdout: Some(p0.p0_30.into_push_pull_output(Level::Low).degrade()),
            },
        );
        i2s.start();

        // Configure buttons
        let btn1 = p0.p0_11.into_pullup_input().degrade();
        let btn2 = p0.p0_12.into_pullup_input().degrade();
        let gpiote = Gpiote::new(ctx.device.GPIOTE);
        gpiote.port().input_pin(&btn1).low();
        gpiote.port().input_pin(&btn2).low();
        gpiote.port().enable_interrupt();

        // Configure the onboard USB CDC UARTE
        let uarte = Uarte::new(
            ctx.device.UARTE0,
            uarte::Pins {
                txd: p0.p0_06.into_push_pull_output(Level::High).degrade(),
                rxd: p0.p0_08.into_floating_input().degrade(),
                cts: None,
                rts: None,
            },
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );

        *ctx.local.queue = Some(Queue::new());
        let (producer, consumer) = ctx.local.queue.as_mut().unwrap().split();

        let mono = Mono::new(ctx.core.SYST, 64_000_000);

        rprintln!("Morse code generator");
        rprintln!("Send me text over UART @ 115_200 baud");
        rprintln!("Press button 1 to slow down or button 2 to speed up");

        tick::spawn().ok();

        (
            Shared { speed: 5_000_000 },
            Local {
                producer,
                consumer,
                gpiote,
                btn1,
                btn2,
                led: p0.p0_13.into_push_pull_output(Level::High).degrade(),
                uarte,
                uarte_timer: Timer::new(ctx.device.TIMER0),
                transfer: i2s.tx(&mute_buf.0).ok(),
                signal_buf: &signal_buf.0,
                mute_buf: &mute_buf.0,
            },
            init::Monotonics(mono),
        )
    }

    #[idle(local = [uarte, uarte_timer, producer])]
    fn idle(ctx: idle::Context) -> ! {
        let uarte = ctx.local.uarte;
        let uarte_timer = ctx.local.uarte_timer;
        let producer = ctx.local.producer;
        let uarte_rx_buf = &mut [0u8; 64][..];
        loop {
            match uarte.read_timeout(uarte_rx_buf, uarte_timer, 100_000) {
                Ok(_) => {
                    if let Ok(msg) = core::str::from_utf8(&uarte_rx_buf[..]) {
                        rprintln!("{}", msg);
                        for action in encode(msg) {
                            for _ in 0..action.duration {
                                producer.enqueue(action.state).ok();
                            }
                        }
                    }
                }
                Err(hal::uarte::Error::Timeout(n)) if n > 0 => {
                    if let Ok(msg) = core::str::from_utf8(&uarte_rx_buf[..n]) {
                        rprintln!("{}", msg);
                        for action in encode(msg) {
                            for _ in 0..action.duration {
                                producer.enqueue(action.state).ok();
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    #[task(local = [consumer, transfer, signal_buf, mute_buf, led], shared = [speed])]
    fn tick(mut ctx: tick::Context) {
        let (_buf, i2s) = ctx.local.transfer.take().unwrap().wait();
        match ctx.local.consumer.dequeue() {
            Some(State::On) => {
                // Move TX pointer to signal buffer (sound ON)
                *ctx.local.transfer = i2s.tx(*ctx.local.signal_buf).ok();
                ctx.local.led.set_low().ok();
            }
            _ => {
                // Move TX pointer to silent buffer (sound OFF)
                *ctx.local.transfer = i2s.tx(*ctx.local.mute_buf).ok();
                ctx.local.led.set_high().ok();
            }
        }
        let speed: u64 = ctx.shared.speed.lock(|speed| *speed).into();
        tick::spawn_after(speed.micros()).ok();
    }

    #[task(binds = GPIOTE, local = [gpiote])]
    fn on_gpiote(ctx: on_gpiote::Context) {
        ctx.local.gpiote.reset_events();
        debounce::spawn_after(50.millis()).ok();
    }

    #[task(local = [btn1, btn2], shared = [speed])]
    fn debounce(mut ctx: debounce::Context) {
        if ctx.local.btn1.is_low().unwrap() {
            rprintln!("Go slower");
            ctx.shared.speed.lock(|speed| *speed += 600_000);
        }
        if ctx.local.btn2.is_low().unwrap() {
            rprintln!("Go faster");
            ctx.shared.speed.lock(|speed| *speed -= 600_000);
        }
    }
}

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
    loop {}
}
