//! HAL interface to the SAADC peripheral.
//!
//! Example usage:
//!
#![cfg_attr(feature = "52840", doc = "```no_run")]
#![cfg_attr(not(feature = "52840"), doc = "```ignore")]
//! # use nrf_hal_common as hal;
//! # use hal::pac::{saadc, SAADC};
//! // subsititute `hal` with the HAL of your board, e.g. `nrf52840_hal`
//! use hal::{
//!    pac::Peripherals,
//!    prelude::*,
//!    gpio::p0::Parts as P0Parts,
//!    saadc::{SaadcConfig, Saadc},
//! };
//!
//! let board = Peripherals::take().unwrap();
//! let gpios = P0Parts::new(board.P0);
//!
//! // initialize saadc interface
//! let saadc_config = SaadcConfig::default();
//! let mut saadc = Saadc::new(board.SAADC, saadc_config);
//! let mut saadc_pin = gpios.p0_02; // the pin your analog device is connected to
//!
//! // blocking read from saadc for `saadc_config.time` microseconds
//! let _saadc_result = saadc.read(&mut saadc_pin);
//! ```

#[cfg(feature = "9160")]
use crate::pac::{generic::Variant, saadc_ns as saadc, SAADC_NS as SAADC};

#[cfg(not(feature = "9160"))]
use crate::pac::{generic::Variant, saadc, SAADC};

use core::{
    hint::unreachable_unchecked,
    sync::atomic::{compiler_fence, Ordering::SeqCst},
};
use embedded_hal::adc::OneShot;

pub use saadc::{
    ch::{
        config::{
            GAIN_A as Gain, MODE_A as Mode, REFSEL_A as Reference, RESP_A as Resistor,
            TACQ_A as Time,
        },
        pseln::PSELN_A as Pseln,
        pselp::PSELP_A as Pselp,
    },
    oversample::OVERSAMPLE_A as Oversample,
    resolution::VAL_A as Resolution,
    RegisterBlock,
};

// Only 1 channel is allowed right now, a discussion needs to be had as to how
// multiple channels should work (See "scan mode" in the datasheet).
// Issue: https://github.com/nrf-rs/nrf-hal/issues/82

/// Interface for the SAADC peripheral.
///
/// External analog channels supported by the SAADC implement the `Channel` trait.
/// Currently, use of only one channel is allowed.
pub struct Saadc {
    saadc: SAADC,
}

impl Saadc {
    pub fn new(saadc: SAADC, config: SaadcConfig) -> Self {
        // The write enums do not implement clone/copy/debug, only the
        // read ones, hence the need to pull out and move the values.
        let SaadcConfig {
            resolution,
            oversample,
        } = config;

        saadc.enable.write(|w| w.enable().enabled());
        saadc.resolution.write(|w| w.val().variant(resolution));
        saadc
            .oversample
            .write(|w| w.oversample().variant(oversample));
        saadc.samplerate.write(|w| w.mode().task());

        // Calibrate
        saadc.tasks_calibrateoffset.write(|w| unsafe { w.bits(1) });
        while saadc.events_calibratedone.read().bits() == 0 {}

        Saadc { saadc }
    }

    pub fn channel(&mut self, n: usize) -> Channel {
        Channel {
            saadc: &mut self.saadc,
            channel: n,
        }
    }
}

pub struct Channel<'a> {
    saadc: &'a mut SAADC,
    channel: usize,
}

impl<'a> Channel<'a> {
    pub fn configure(
        &mut self,
        pseln: impl Into<Pseln>,
        pselp: impl Into<Pselp>,
        config: SaadcChannelConfig,
    ) {
        let SaadcChannelConfig {
            reference,
            gain,
            mode,
            resistor,
            time,
        } = config;

        self.saadc.ch[self.channel].config.write(|w| {
            w.refsel().variant(reference);
            w.gain().variant(gain);
            w.tacq().variant(time);
            w.mode().variant(mode);
            w.resp().variant(resistor);
            w.resn().bypass();
            w.burst().enabled();
            w
        });

        self.saadc.ch[self.channel]
            .pselp
            .write(|w| w.pselp().variant(pselp.into()));
        self.saadc.ch[self.channel]
            .pseln
            .write(|w| w.pseln().variant(pseln.into()));
    }

    /// Sample channel `PIN` for the configured ADC acquisition time in differential input mode.
    /// Note that this is a blocking operation.
    pub fn read(&mut self) -> nb::Result<i16, ()> {
        let mut val: i16 = 0;
        self.saadc
            .result
            .ptr
            .write(|w| unsafe { w.ptr().bits(((&mut val) as *mut _) as u32) });
        self.saadc
            .result
            .maxcnt
            .write(|w| unsafe { w.maxcnt().bits(1) });

        // Conservative compiler fence to prevent starting the ADC before the
        // pointer and maxcount have been set.
        compiler_fence(SeqCst);

        self.saadc.tasks_start.write(|w| unsafe { w.bits(1) });
        self.saadc.tasks_sample.write(|w| unsafe { w.bits(1) });

        while self.saadc.events_end.read().bits() == 0 {}
        self.saadc.events_end.reset();

        // Will only occur if more than one channel has been enabled.
        if self.saadc.result.amount.read().bits() != 1 {
            return Err(nb::Error::Other(()));
        }

        // Second fence to prevent optimizations creating issues with the EasyDMA-modified `val`.
        compiler_fence(SeqCst);

        Ok(val)
    }

    pub fn as_millis(&self, value: i16) -> i32 {
        let mode = match self.saadc.ch[self.channel].config.read().mode().variant() {
            Mode::SE => 1,
            Mode::DIFF => 2,
        };

        let lsbs = match self.saadc.resolution.read().val().variant() {
            Variant::Val(Resolution::_8BIT) => 256,
            Variant::Val(Resolution::_10BIT) => 1024,
            Variant::Val(Resolution::_12BIT) => 4096,
            Variant::Val(Resolution::_14BIT) => 16384,
            _ => unreachable!(),
        } / mode;

        let gain = match self.saadc.ch[self.channel].config.read().gain().variant() {
            Gain::GAIN1_6 => (1, 6),
            Gain::GAIN1_5 => (1, 5),
            Gain::GAIN1_4 => (1, 4),
            Gain::GAIN1_3 => (1, 3),
            Gain::GAIN1_2 => (1, 2),
            Gain::GAIN1 => (1, 1),
            Gain::GAIN2 => (2, 1),
            Gain::GAIN4 => (4, 1),
        };

        let reference = match self.saadc.ch[self.channel].config.read().refsel().variant() {
            Reference::INTERNAL => (3, 5),
            Reference::VDD1_4 => (33, 40),
        };

        value as i32 * gain.1 * reference.0 * 1000 / gain.0 / reference.1 / lsbs
    }
}

/// Used to configure the SAADC peripheral.
///
/// See the documentation of the `Default` impl for suitable default values.
pub struct SaadcConfig {
    /// Output resolution in bits.
    pub resolution: Resolution,
    /// Average 2^`oversample` input samples before transferring the result into memory.
    pub oversample: Oversample,
}

/// Used to configure a single SAADC peripheral channel.
///
/// See the documentation of the `Default` impl for suitable default values.
pub struct SaadcChannelConfig {
    /// Reference voltage of the SAADC input.
    pub reference: Reference,
    /// Gain used to control the effective input range of the SAADC.
    pub gain: Gain,
    /// The mode of the differential ADC
    pub mode: Mode,
    /// Positive channel resistor control.
    pub resistor: Resistor,
    /// Acquisition time in microseconds.
    pub time: Time,
}

/// Default SAADC configuration. 0 volts reads as 0, VDD volts reads as `u16::MAX`.
/// The returned SaadcConfig is configured with the following values:
///
#[cfg_attr(feature = "52840", doc = "```")]
#[cfg_attr(not(feature = "52840"), doc = "```ignore")]
/// # use nrf_hal_common::saadc::SaadcConfig;
/// # use nrf_hal_common::pac::{saadc, SAADC};
/// # use saadc::{
/// #    ch::config::{GAIN_A as Gain, REFSEL_A as Reference, RESP_A as Resistor, TACQ_A as Time},
/// #    oversample::OVERSAMPLE_A as Oversample,
/// #    resolution::VAL_A as Resolution,
/// # };
/// # let saadc =
/// SaadcConfig {
///     resolution: Resolution::_14BIT,
///     oversample: Oversample::OVER8X,
/// };
/// #
/// # // Ensure default values haven't changed
/// # let test_saadc = SaadcConfig::default();
/// # assert_eq!(saadc.resolution, test_saadc.resolution);
/// # assert_eq!(saadc.oversample, test_saadc.oversample);
/// # ()
/// ```
impl Default for SaadcConfig {
    fn default() -> Self {
        // Note: do not forget to update the docs above if you change values here
        SaadcConfig {
            resolution: Resolution::_14BIT,
            oversample: Oversample::OVER8X,
        }
    }
}

/// Default SAADC configuration. 0 volts reads as 0, VDD volts reads as `u16::MAX`.
/// The returned SaadcConfig is configured with the following values:
///
/// ```
/// # use nrf_hal_common::saadc::{SaadcConfig, SaadcChannelConfig};
/// #[cfg(feature = "9160")]
/// use nrf_hal_common::pac::{generic::Variant, saadc_ns as saadc, SAADC_NS as SAADC};
///
/// #[cfg(not(feature = "9160"))]
/// use nrf_hal_common::pac::{generic::Variant, saadc, SAADC};
/// # use saadc::{
/// #    ch::config::{GAIN_A as Gain, MODE_A as Mode, REFSEL_A as Reference, RESP_A as Resistor, TACQ_A as Time},
/// #    oversample::OVERSAMPLE_A as Oversample,
/// #    resolution::VAL_A as Resolution,
/// # };
/// # let saadc_channel =
/// SaadcChannelConfig {
///     reference: Reference::VDD1_4,
///     gain: Gain::GAIN1_4,
///     mode: Mode::SE,
///     resistor: Resistor::BYPASS,
///     time: Time::_20US,
/// };
/// #
/// # // ensure default values haven't changed
/// # let test_saadc_channel = SaadcChannelConfig::default();
/// # assert_eq!(saadc_channel.reference, test_saadc_channel.reference);
/// # assert_eq!(saadc_channel.gain, test_saadc_channel.gain);
/// # assert_eq!(saadc_channel.mode, test_saadc_channel.mode);
/// # assert_eq!(saadc_channel.resistor, test_saadc_channel.resistor);
/// # assert_eq!(saadc_channel.time, test_saadc_channel.time);
/// # ()
/// ```
impl Default for SaadcChannelConfig {
    fn default() -> Self {
        SaadcChannelConfig {
            reference: Reference::VDD1_4,
            gain: Gain::GAIN1_4,
            mode: Mode::SE,
            resistor: Resistor::BYPASS,
            time: Time::_20US,
        }
    }
}

impl<PIN> OneShot<Saadc, i16, PIN> for Saadc
where
    PIN: embedded_hal::adc::Channel<Saadc, ID = u8>,
{
    type Error = ();

    /// Sample channel `PIN` for the configured ADC acquisition time in differential input mode.
    /// Note that this is a blocking operation.
    fn read(&mut self, _pin: &mut PIN) -> nb::Result<i16, Self::Error> {
        match PIN::channel() {
            0 => self.saadc.ch[0].pselp.write(|w| w.pselp().analog_input0()),
            1 => self.saadc.ch[0].pselp.write(|w| w.pselp().analog_input1()),
            2 => self.saadc.ch[0].pselp.write(|w| w.pselp().analog_input2()),
            3 => self.saadc.ch[0].pselp.write(|w| w.pselp().analog_input3()),
            4 => self.saadc.ch[0].pselp.write(|w| w.pselp().analog_input4()),
            5 => self.saadc.ch[0].pselp.write(|w| w.pselp().analog_input5()),
            6 => self.saadc.ch[0].pselp.write(|w| w.pselp().analog_input6()),
            7 => self.saadc.ch[0].pselp.write(|w| w.pselp().analog_input7()),
            #[cfg(not(feature = "9160"))]
            8 => self.saadc.ch[0].pselp.write(|w| w.pselp().vdd()),
            // This can never happen the only analog pins have already been defined
            // PAY CLOSE ATTENTION TO ANY CHANGES TO THIS IMPL OR THE `channel_mappings!` MACRO
            _ => unsafe { unreachable_unchecked() },
        }

        let mut val: i16 = 0;
        self.saadc
            .result
            .ptr
            .write(|w| unsafe { w.ptr().bits(((&mut val) as *mut _) as u32) });
        self.saadc
            .result
            .maxcnt
            .write(|w| unsafe { w.maxcnt().bits(1) });

        // Conservative compiler fence to prevent starting the ADC before the
        // pointer and maxcount have been set.
        compiler_fence(SeqCst);

        self.saadc.tasks_start.write(|w| unsafe { w.bits(1) });
        self.saadc.tasks_sample.write(|w| unsafe { w.bits(1) });

        while self.saadc.events_end.read().bits() == 0 {}
        self.saadc.events_end.reset();

        // Will only occur if more than one channel has been enabled.
        if self.saadc.result.amount.read().bits() != 1 {
            return Err(nb::Error::Other(()));
        }

        // Second fence to prevent optimizations creating issues with the EasyDMA-modified `val`.
        compiler_fence(SeqCst);

        Ok(val)
    }
}

macro_rules! channel_mappings {
    ( $($n:expr => $pin:ident,)*) => {
        $(
            impl<STATE> embedded_hal::adc::Channel<Saadc> for crate::gpio::p0::$pin<STATE> {
                type ID = u8;

                fn channel() -> <Self as embedded_hal::adc::Channel<Saadc>>::ID {
                    $n
                }
            }
        )*
    };
}

#[cfg(feature = "9160")]
channel_mappings! {
    0 => P0_13,
    1 => P0_14,
    2 => P0_15,
    3 => P0_16,
    4 => P0_17,
    5 => P0_18,
    6 => P0_19,
    7 => P0_20,
}

#[cfg(not(feature = "9160"))]
channel_mappings! {
    0 => P0_02,
    1 => P0_03,
    2 => P0_04,
    3 => P0_05,
    4 => P0_28,
    5 => P0_29,
    6 => P0_30,
    7 => P0_31,
}

#[cfg(not(feature = "9160"))]
impl embedded_hal::adc::Channel<Saadc> for InternalVdd {
    type ID = u8;

    fn channel() -> <Self as embedded_hal::adc::Channel<Saadc>>::ID {
        8
    }
}

#[cfg(not(feature = "9160"))]
/// Channel that doesn't sample a pin, but the internal VDD voltage.
pub struct InternalVdd;

macro_rules! psel_mappings {
    ( $($psel:ident => $pin:ident,)*) => {
        $(
            impl<STATE> Into<Pseln> for crate::gpio::p0::$pin<STATE> {
                fn into(self) -> Pseln {
                    Pseln::$psel
                }
            }

            impl<STATE> Into<Pselp> for crate::gpio::p0::$pin<STATE> {
                fn into(self) -> Pselp {
                    Pselp::$psel
                }
            }
        )*
    };
}

psel_mappings! {
    ANALOGINPUT0 => P0_02,
    ANALOGINPUT1 => P0_03,
    ANALOGINPUT2 => P0_04,
    ANALOGINPUT3 => P0_05,
    ANALOGINPUT4 => P0_28,
    ANALOGINPUT5 => P0_29,
    ANALOGINPUT6 => P0_30,
    ANALOGINPUT7 => P0_31,
}
