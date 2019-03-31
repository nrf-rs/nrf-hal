//! A wall clock fed by the Real Time Counter peripheral

use core::sync::atomic::{
    AtomicUsize,
    Ordering
};

use nrf52_hal_common::{
    Rtc,
    rtc::{
        Error,
        RtcExt,
        Started,
        Stopped,
        RtcInterrupt,
    },
    target::{
        RTC0,
        RTC1,
    }
};

#[cfg(not(feature = "52810"))]
use nrf52_hal_common::target::RTC2;

static RTC0_PENDING: AtomicUsize = AtomicUsize::new(0);
static RTC1_PENDING: AtomicUsize = AtomicUsize::new(0);

#[cfg(not(feature = "52810"))]
static RTC2_PENDING: AtomicUsize = AtomicUsize::new(0);


pub struct WallClockConfig {
    /// A 12 bit prescaler to the 32_768 Hz Low Frequency Oscillator
    ///
    /// NOTE: 1 is added to this number as a divisor.
    pub prescaler: u16,
}

impl Default for WallClockConfig {
    fn default() -> Self {
        WallClockConfig {
            // Default to 8Hz tick
            prescaler: 0xFFF
        }
    }
}

pub struct WallClock<C>
    where C: RtcExt,
{
    hal: Rtc<C, Started>,
    nanos_per_tick: u32,
}

impl<C> WallClock<C>
    where C: RtcExt,
{
    #[cfg(feature = "rtfm")]
    pub fn from_hal_rtfm(
        rtc: Rtc<C, Stopped>,
        cfg: WallClockConfig,
    ) -> Result<Self, Error> {
        Self::from_hal_inner(rtc, cfg)
    }

    pub fn from_hal(
        rtc: Rtc<C, Stopped>,
        cfg: WallClockConfig,
        // TODO: get nvic
    ) -> Result<Self, Error> {
        Self::from_hal_inner(rtc, cfg)
    }

    #[inline(always)]
    fn from_hal_inner(
        mut rtc: Rtc<C, Stopped>,
        cfg: WallClockConfig,
    ) -> Result<Self, Error> {
        // Set prescaler
        let prescaler32 = u32::from(cfg.prescaler);
        rtc.set_prescaler(prescaler32)?;

        // Make sure event is cleared before start
        let _ = rtc.get_event_triggered(
            RtcInterrupt::Tick,
            true,
        );

        let nanos = 1_000_000_000 / (32_768 / (prescaler32 + 1));

        Ok(WallClock {
            hal: rtc.enable_counter(),
            nanos_per_tick: nanos,
        })
    }

    fn update_ticks(&mut self) {
        let ticks = self.get_pending();
    }
}

impl WallClock<RTC0> {
    fn get_pending(&self) -> usize {
        RTC0_PENDING.swap(0, Ordering::SeqCst)
    }
}

impl WallClock<RTC1> {
    fn get_pending(&self) -> usize {
        RTC1_PENDING.swap(0, Ordering::SeqCst)
    }
}

#[cfg(not(feature = "52810"))]
impl WallClock<RTC2> {
    fn get_pending(&self) -> usize {
        RTC2_PENDING.swap(0, Ordering::SeqCst)
    }
}
