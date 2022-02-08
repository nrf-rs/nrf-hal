#![no_std]
#![no_main]

use {core::panic::PanicInfo, nrf52840_hal as hal, rtt_target::rprintln};

#[rtic::app(device = crate::hal::pac, peripherals = true, dispatchers = [SWI0_EGU0])]
mod app {
    use embedded_hal::digital::v2::InputPin;
    use {
        hal::{
            gpio::{p0::Parts, Input, Level, Pin, PullUp},
            gpiote::Gpiote,
            pac::PWM0,
            pwm::*,
            time::*,
        },
        nrf52840_hal as hal,
        rtt_target::{rprintln, rtt_init_print},
        systick_monotonic::*,
    };

    #[monotonic(binds = SysTick, default = true)]
    type Mono = Systick<1_000_000>;

    #[shared]
    struct Shared {
        #[lock_free]
        pwm: Option<PwmSeq<PWM0, SeqBuffer, SeqBuffer>>,
    }

    #[local]
    struct Local {
        gpiote: Gpiote,
        btn1: Pin<Input<PullUp>>,
        btn2: Pin<Input<PullUp>>,
        btn3: Pin<Input<PullUp>>,
        btn4: Pin<Input<PullUp>>,
        status: AppStatus,
    }

    #[derive(Debug, PartialEq)]
    pub enum AppStatus {
        Idle,
        Demo1A,
        Demo1B,
        Demo1C,
        Demo2A,
        Demo2B,
        Demo2C,
        Demo3,
        Demo4,
    }

    type SeqBuffer = &'static mut [u16; 48];

    #[init(local = [
        BUF0: [u16; 48] = [0u16; 48],
        BUF1: [u16; 48] = [0u16; 48],
    ])]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let BUF0 = ctx.local.BUF0;
        let BUF1 = ctx.local.BUF1;

        let _clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        ctx.core.DCB.enable_trace();
        ctx.core.DWT.enable_cycle_counter();
        rtt_init_print!();

        let p0 = Parts::new(ctx.device.P0);
        let btn1 = p0.p0_11.into_pullup_input().degrade();
        let btn2 = p0.p0_12.into_pullup_input().degrade();
        let btn3 = p0.p0_24.into_pullup_input().degrade();
        let btn4 = p0.p0_25.into_pullup_input().degrade();
        let led1 = p0.p0_13.into_push_pull_output(Level::High).degrade();
        let led2 = p0.p0_14.into_push_pull_output(Level::High).degrade();
        let led3 = p0.p0_15.into_push_pull_output(Level::High).degrade();
        let led4 = p0.p0_16.into_push_pull_output(Level::High).degrade();

        let mut pwm = Pwm::new(ctx.device.PWM0);
        pwm.set_period(500u32.hz())
            .set_output_pin(Channel::C0, led1)
            .set_output_pin(Channel::C1, led2)
            .set_output_pin(Channel::C2, led3)
            .set_output_pin(Channel::C3, led4)
            .enable_interrupt(PwmEvent::Stopped);

        // In addition to `set_output_pin`, `swap_output_pin` and `clear_output_pin` can be used to
        // get the old pin back.
        let led1 = pwm.clear_output_pin(Channel::C0).unwrap();
        assert!(pwm.swap_output_pin(Channel::C0, led1).is_none());

        pwm.enable();

        let gpiote = Gpiote::new(ctx.device.GPIOTE);
        gpiote.port().input_pin(&btn1).low();
        gpiote.port().input_pin(&btn2).low();
        gpiote.port().input_pin(&btn3).low();
        gpiote.port().input_pin(&btn4).low();
        gpiote.port().enable_interrupt();

        let mono = Mono::new(ctx.core.SYST, 64_000_000);

        (
            Shared {
                pwm: pwm.load(Some(BUF0), Some(BUF1), false).ok(),
            },
            Local {
                gpiote,
                btn1,
                btn2,
                btn3,
                btn4,
                status: AppStatus::Idle,
            },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        rprintln!("Press a button to start a demo");
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task(binds = PWM0, shared = [pwm])]
    fn on_pwm(ctx: on_pwm::Context) {
        let pwm_seq = ctx.shared.pwm.as_ref().unwrap();
        if pwm_seq.is_event_triggered(PwmEvent::Stopped) {
            pwm_seq.reset_event(PwmEvent::Stopped);
            rprintln!("PWM generation was stopped");
        }
    }

    #[task(binds = GPIOTE, local = [gpiote])]
    fn on_gpiote(ctx: on_gpiote::Context) {
        ctx.local.gpiote.reset_events();
        debounce::spawn_after(50.millis()).ok();
    }

    #[task(shared = [pwm], local = [btn1, btn2, btn3, btn4, status])]
    fn debounce(ctx: debounce::Context) {
        let (buf0, buf1, pwm) = ctx.shared.pwm.take().unwrap().split();
        let BUF0 = buf0.unwrap();
        let BUF1 = buf1.unwrap();

        let max_duty = pwm.max_duty();
        let (ch0, ch1, ch2, ch3) = pwm.split_channels();
        let (grp0, grp1) = pwm.split_groups();

        let status = ctx.local.status;
        if ctx.local.btn1.is_low().unwrap() {
            match status {
                AppStatus::Demo1B => {
                    rprintln!("DEMO 1C: Individual channel duty cycle");
                    *status = AppStatus::Demo1C;
                    ch0.set_duty(max_duty / 10);
                    ch1.set_duty(max_duty / 50);
                    ch2.set_duty(max_duty / 100);
                    ch3.set_duty(max_duty / 500);
                }
                AppStatus::Demo1A => {
                    rprintln!("DEMO 1B: Group duty cycle");
                    *status = AppStatus::Demo1B;
                    grp0.set_duty(max_duty / 300);
                    grp1.set_duty(max_duty / 10);
                }
                _ => {
                    rprintln!("DEMO 1A: Common duty cycle for all channels");
                    *status = AppStatus::Demo1A;
                    pwm.set_duty_on_common(max_duty / 10);
                }
            }
            *ctx.shared.pwm = pwm.load(Some(BUF0), Some(BUF1), false).ok();
        } else if ctx.local.btn2.is_low().unwrap() {
            match status {
                AppStatus::Demo2B => {
                    rprintln!("DEMO 2C: Play grouped sequence 4 times");
                    *status = AppStatus::Demo2C;
                    let ampl = max_duty as i32 / 20;
                    let len: usize = BUF0.len() / 2;
                    // In `Grouped` mode, each step consists of two values [G0, G1]
                    for x in 0..len {
                        BUF0[x * 2] = triangle_wave(x, len, ampl, 6, 0) as u16;
                        BUF0[x * 2 + 1] = triangle_wave(x, len, ampl, 0, 0) as u16;
                    }
                    BUF1.copy_from_slice(&BUF0[..]);
                    pwm.set_load_mode(LoadMode::Grouped)
                        .set_step_mode(StepMode::Auto)
                        .set_seq_refresh(Seq::Seq0, 30) // Playback rate (periods per step)
                        .set_seq_refresh(Seq::Seq1, 10)
                        .repeat(4);
                    *ctx.shared.pwm = pwm.load(Some(BUF0), Some(BUF1), true).ok();
                }
                AppStatus::Demo2A => {
                    rprintln!("DEMO 2B: Loop individual sequences");
                    *status = AppStatus::Demo2B;
                    let ampl = max_duty as i32 / 5;
                    let offset = max_duty as i32 / 300;
                    let len = BUF0.len() / 4;
                    // In `Individual` mode, each step consists of four values [C0, C1, C2, C3]
                    for x in 0..len {
                        BUF0[4 * x] = triangle_wave(x, len, ampl, 0, offset) as u16;
                        BUF0[4 * x + 1] = triangle_wave(x, len, ampl, 3, offset) as u16;
                        BUF0[4 * x + 2] = triangle_wave(x, len, ampl, 6, offset) as u16;
                        BUF0[4 * x + 3] = triangle_wave(x, len, ampl, 9, offset) as u16;
                    }
                    BUF1.copy_from_slice(&BUF0[..]);
                    pwm.set_load_mode(LoadMode::Individual)
                        .set_seq_refresh(Seq::Seq0, 30)
                        .set_seq_refresh(Seq::Seq1, 30)
                        .loop_inf();
                    *ctx.shared.pwm = pwm.load(Some(BUF0), Some(BUF1), true).ok();
                }
                _ => {
                    rprintln!("DEMO 2A: Play common sequence once");
                    *status = AppStatus::Demo2A;
                    let len = BUF0.len();
                    // In `Common` mode, each step consists of one value for all channels.
                    for x in 0..len {
                        BUF0[x] = triangle_wave(x, len, 2000, 0, 100) as u16;
                    }
                    BUF1.copy_from_slice(&BUF0[..]);
                    pwm.set_load_mode(LoadMode::Common)
                        .set_step_mode(StepMode::Auto)
                        .set_seq_refresh(Seq::Seq0, 20)
                        .set_seq_refresh(Seq::Seq1, 20)
                        .one_shot();
                    *ctx.shared.pwm = pwm.load(Some(BUF0), Some(BUF1), true).ok();
                }
            }
        } else if ctx.local.btn3.is_low().unwrap() {
            match status {
                AppStatus::Demo3 => {
                    rprintln!("DEMO 3: Next step");
                    pwm.next_step();
                    if pwm.is_event_triggered(PwmEvent::SeqEnd(Seq::Seq1)) {
                        rprintln!("DEMO 3: End");
                        pwm.reset_event(PwmEvent::SeqEnd(Seq::Seq1));
                        pwm.stop();
                        *status = AppStatus::Idle;
                    }
                    *ctx.shared.pwm = pwm.load(Some(BUF0), Some(BUF1), false).ok();
                }
                _ => {
                    rprintln!("DEMO 3: Manually step through sequence");
                    *status = AppStatus::Demo3;
                    let amplitude = max_duty as i32 / 20;
                    let offset = max_duty as i32 / 300;
                    let len = BUF0.len();
                    for x in 0..len {
                        BUF0[x] = triangle_wave(x * 8, len, amplitude, 0, offset) as u16;
                    }
                    BUF1.copy_from_slice(&BUF0[..]);
                    pwm.set_load_mode(LoadMode::Common)
                        .set_step_mode(StepMode::NextStep)
                        .loop_inf();
                    *ctx.shared.pwm = pwm.load(Some(BUF0), Some(BUF1), true).ok();
                }
            }
        } else if ctx.local.btn4.is_low().unwrap() {
            rprintln!("DEMO 4: Waveform mode");
            *status = AppStatus::Demo4;
            let len = BUF0.len() / 4;
            // In `Waveform` mode, each step consists of four values [C0, C1, C2, MAX_DUTY]
            // So the maximum duty cycle can be set on a per step basis, affecting the PWM frequency
            for x in 0..len {
                let current_max = x * 2_200 + 5_000;
                BUF0[4 * x] = ((x % 3) * current_max / (5 * (x + 1))) as u16;
                BUF0[4 * x + 1] = (((x + 1) % 3) * current_max / (5 * (x + 1))) as u16;
                BUF0[4 * x + 2] = (((x + 2) % 3) * current_max / (5 * (x + 1))) as u16;
                BUF0[4 * x + 3] = current_max as u16;
            }
            BUF1.copy_from_slice(&BUF0[..]);
            pwm.set_load_mode(LoadMode::Waveform)
                .set_step_mode(StepMode::Auto)
                .set_seq_refresh(Seq::Seq0, 150)
                .set_seq_refresh(Seq::Seq1, 150)
                .loop_inf();
            *ctx.shared.pwm = pwm.load(Some(BUF0), Some(BUF1), true).ok();
        } else {
            *ctx.shared.pwm = pwm.load(Some(BUF0), Some(BUF1), false).ok();
        }
    }

    fn triangle_wave(x: usize, length: usize, ampl: i32, phase: i32, y_offset: i32) -> i32 {
        let x = x as i32;
        let length = length as i32;
        ampl - ((2 * (x + phase) * ampl / length) % (2 * ampl) - ampl).abs() + y_offset
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    rprintln!("{}", info);
    loop {}
}
