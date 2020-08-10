#![no_std]
#![no_main]

use embedded_hal::digital::v2::InputPin;
use {
    core::{
        panic::PanicInfo,
        sync::atomic::{compiler_fence, Ordering},
    },
    hal::{
        gpio::{p0::Parts, Input, Level, Pin, PullUp},
        gpiote::Gpiote,
        pac::PWM0,
        pwm::*,
        time::*,
    },
    nrf52840_hal as hal,
    rtic::cyccnt::U32Ext as _,
    rtt_target::{rprintln, rtt_init_print},
};

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

#[rtic::app(device = crate::hal::pac, peripherals = true,  monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        gpiote: Gpiote,
        btn1: Pin<Input<PullUp>>,
        btn2: Pin<Input<PullUp>>,
        btn3: Pin<Input<PullUp>>,
        btn4: Pin<Input<PullUp>>,
        pwm: Pwm<PWM0>,
        #[init(AppStatus::Idle)]
        status: AppStatus,
    }

    #[init]
    fn init(mut ctx: init::Context) -> init::LateResources {
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

        let pwm = Pwm::new(ctx.device.PWM0);
        pwm.set_period(100u32.hz().into())
            .set_output_pin(Channel::C0, &led1)
            .set_output_pin(Channel::C1, &led2)
            .set_output_pin(Channel::C2, &led3)
            .set_output_pin(Channel::C3, &led4)
            .enable_interrupt(PwmEvent::Stopped)
            .enable();

        let gpiote = Gpiote::new(ctx.device.GPIOTE);
        gpiote.port().input_pin(&btn1).low();
        gpiote.port().input_pin(&btn2).low();
        gpiote.port().input_pin(&btn3).low();
        gpiote.port().input_pin(&btn4).low();
        gpiote.port().enable_interrupt();

        init::LateResources {
            gpiote,
            btn1,
            btn2,
            btn3,
            btn4,
            pwm,
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        rprintln!("Press a button to start a demo");
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task(binds = PWM0, resources = [pwm])]
    fn on_pwm(ctx: on_pwm::Context) {
        let pwm = ctx.resources.pwm;
        if pwm.is_event_triggered(PwmEvent::Stopped) {
            pwm.reset_event(PwmEvent::Stopped);
            rprintln!("PWM generation stopped");
        }
    }

    #[task(binds = GPIOTE, resources = [gpiote], schedule = [debounce])]
    fn on_gpiote(ctx: on_gpiote::Context) {
        ctx.resources.gpiote.reset_events();
        ctx.schedule.debounce(ctx.start + 3_000_000.cycles()).ok();
    }

    #[task(resources = [btn1, btn2, btn3, btn4, pwm, status])]
    fn debounce(ctx: debounce::Context) {
        static mut BUF: [u16; 48] = [0u16; 48];
        let status = ctx.resources.status;

        let pwm = ctx.resources.pwm;
        let max_duty = pwm.get_max_duty();
        let (ch0, ch1, ch2, ch3) = pwm.split_channels();
        let (grp0, grp1) = pwm.split_groups();

        if ctx.resources.btn1.is_low().unwrap() {
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
                    grp1.set_duty(max_duty / 10);
                    grp0.set_duty(max_duty / 300);
                }
                _ => {
                    rprintln!("DEMO 1A: Common duty cycle for all channels");
                    *status = AppStatus::Demo1A;
                    pwm.set_duty_on_common(max_duty / 10);
                }
            }
        }
        if ctx.resources.btn2.is_low().unwrap() {
            match status {
                AppStatus::Demo2A => {
                    rprintln!("DEMO 2B: Loop individual sequences");
                    *status = AppStatus::Demo2B;
                    let amplitude = max_duty as i32 / 5;
                    let offset = max_duty as i32 / 300;
                    for x in 0..12 {
                        BUF[4 * x] = triangle_wave(x as i32, 12, amplitude, 0, 0) as u16;
                        BUF[4 * x + 1] = triangle_wave(x as i32, 12, amplitude, 3, offset) as u16;
                        BUF[4 * x + 2] = triangle_wave(x as i32, 12, amplitude, 6, offset) as u16;
                        BUF[4 * x + 3] = triangle_wave(x as i32, 12, amplitude, 9, offset) as u16;
                    }
                    pwm.set_load_mode(LoadMode::Individual)
                        .set_seq_refresh(Seq::Seq0, 30)
                        .set_seq_refresh(Seq::Seq1, 30)
                        .loop_inf();
                    pwm.load_seq(Seq::Seq0, &BUF[..48]).ok();
                    pwm.load_seq(Seq::Seq1, &BUF[..48]).ok();
                    pwm.start_seq(Seq::Seq0);
                }
                _ => {
                    rprintln!("DEMO 2A: Play sequence once");
                    *status = AppStatus::Demo2A;
                    for x in 0..10 {
                        BUF[x] = triangle_wave(x as i32, 10, 2000, 0, 100) as u16;
                    }
                    pwm.set_load_mode(LoadMode::Common)
                        .one_shot()
                        .set_seq_refresh(Seq::Seq0, 50)
                        .set_step_mode(StepMode::Auto)
                        .load_seq(Seq::Seq0, &BUF[..10])
                        .ok();
                    pwm.start_seq(Seq::Seq0);
                }
            }
        }
        if ctx.resources.btn3.is_low().unwrap() {
            match status {
                AppStatus::Demo3 => {
                    rprintln!("DEMO 3: Next step");
                    if pwm.is_event_triggered(PwmEvent::SeqEnd(Seq::Seq1)) {
                        rprintln!("DEMO 3: End");
                        pwm.reset_event(PwmEvent::SeqEnd(Seq::Seq1));
                        pwm.stop();
                        *status = AppStatus::Idle;
                    } else {
                        pwm.next_step();
                    }
                }
                _ => {
                    rprintln!("DEMO 3: Manually step through sequence");
                    *status = AppStatus::Demo3;
                    for x in 0..8 {
                        BUF[x] = triangle_wave(
                            x as i32,
                            8,
                            max_duty as i32 / 50,
                            0,
                            max_duty as i32 / 800,
                        ) as u16;
                    }
                    pwm.set_load_mode(LoadMode::Common)
                        .loop_inf()
                        .set_step_mode(StepMode::NextStep);
                    pwm.load_seq(Seq::Seq0, &BUF[..4]).ok();
                    pwm.load_seq(Seq::Seq1, &BUF[4..8]).ok();
                    pwm.start_seq(Seq::Seq0);
                }
            }
        }
        if ctx.resources.btn4.is_low().unwrap() {
            rprintln!("DEMO 4: Play complex sequence 4 times");
            *status = AppStatus::Demo4;
            for x in 0..12 {
                BUF[x] = triangle_wave(x as i32, 12, max_duty as i32 / 20, 0, max_duty as i32 / 800)
                    as u16;
            }
            pwm.set_load_mode(LoadMode::Common)
                .set_step_mode(StepMode::Auto)
                .set_seq_refresh(Seq::Seq0, 100)
                .set_seq_refresh(Seq::Seq1, 20)
                .repeat(4);
            pwm.load_seq(Seq::Seq0, &BUF[..6]).ok();
            pwm.load_seq(Seq::Seq1, &BUF[6..12]).ok();
            pwm.start_seq(Seq::Seq0);
        }
    }

    extern "C" {
        fn SWI0_EGU0();
        fn SWI1_EGU1();
        fn SWI2_EGU2();
    }
};

fn triangle_wave(x: i32, length: i32, amplitude: i32, phase: i32, y_offset: i32) -> i32 {
    (amplitude - (((x + phase) * amplitude / (length / (2))) % (2 * amplitude) - amplitude).abs())
        + y_offset
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
