//! HAL interface for the QDEC peripheral.
//!
//! The Quadrature decoder (QDEC) provides buffered decoding of quadrature-encoded sensor signals.
//! It is suitable for mechanical and optical sensors.

use {
    crate::gpio::{Input, Pin, PullUp},
    crate::pac::QDEC,
};

/// A safe wrapper around the `QDEC` peripheral with associated pins.
pub struct Qdec {
    qdec: QDEC,
}

impl Qdec {
    /// Takes ownership of the `QDEC` peripheral and associated pins, returning a safe wrapper.
    pub fn new(qdec: QDEC, pins: Pins, sample_period: SamplePeriod) -> Self {
        qdec.psel.a.write(|w| {
            unsafe { w.bits(pins.a.psel_bits()) };
            w.connect().connected()
        });
        qdec.psel.b.write(|w| {
            unsafe { w.bits(pins.b.psel_bits()) };
            w.connect().connected()
        });

        if let Some(p) = &pins.led {
            qdec.psel.led.write(|w| {
                unsafe { w.bits(p.psel_bits()) };
                w.connect().connected()
            });
        }

        match sample_period {
            SamplePeriod::_128us => qdec.sampleper.write(|w| w.sampleper()._128us()),
            SamplePeriod::_256us => qdec.sampleper.write(|w| w.sampleper()._256us()),
            SamplePeriod::_512us => qdec.sampleper.write(|w| w.sampleper()._512us()),
            SamplePeriod::_1024us => qdec.sampleper.write(|w| w.sampleper()._1024us()),
            SamplePeriod::_2048us => qdec.sampleper.write(|w| w.sampleper()._2048us()),
            SamplePeriod::_4096us => qdec.sampleper.write(|w| w.sampleper()._4096us()),
            SamplePeriod::_8192us => qdec.sampleper.write(|w| w.sampleper()._8192us()),
            SamplePeriod::_16384us => qdec.sampleper.write(|w| w.sampleper()._16384us()),
            SamplePeriod::_32ms => qdec.sampleper.write(|w| w.sampleper()._32ms()),
            SamplePeriod::_65ms => qdec.sampleper.write(|w| w.sampleper()._65ms()),
            SamplePeriod::_131ms => qdec.sampleper.write(|w| w.sampleper()._131ms()),
        }

        Self { qdec }
    }

    /// Enables/disables input debounce filters.
    #[inline(always)]
    pub fn debounce(&self, enable: bool) -> &Self {
        match enable {
            true => self.qdec.dbfen.write(|w| w.dbfen().enabled()),
            false => self.qdec.dbfen.write(|w| w.dbfen().disabled()),
        }
        self
    }

    /// LED output pin polarity.
    #[inline(always)]
    pub fn led_polarity(&self, polarity: LedPolarity) -> &Self {
        self.qdec.ledpol.write(|w| match polarity {
            LedPolarity::ActiveHigh => w.ledpol().active_high(),
            LedPolarity::ActiveLow => w.ledpol().active_low(),
        });
        self
    }

    /// Time period the LED is switched ON prior to sampling (0..511 us).
    #[inline(always)]
    pub fn led_pre(&self, usecs: u16) -> &Self {
        self.qdec
            .ledpre
            .write(|w| unsafe { w.ledpre().bits(usecs.min(511)) });
        self
    }

    /// Marks the interrupt trigger event as handled.
    #[inline(always)]
    pub fn reset_events(&self) {
        self.qdec.events_reportrdy.write(|w| w);
    }

    /// Triggers the QDEC interrupt on the specified number of non-zero samples.
    #[inline(always)]
    pub fn enable_interrupt(&self, num_samples: NumSamples) -> &Self {
        self.qdec.reportper.write(|w| match num_samples {
            NumSamples::_10smpl => w.reportper()._10smpl(),
            NumSamples::_40smpl => w.reportper()._40smpl(),
            NumSamples::_80smpl => w.reportper()._80smpl(),
            NumSamples::_120smpl => w.reportper()._120smpl(),
            NumSamples::_160smpl => w.reportper()._160smpl(),
            NumSamples::_200smpl => w.reportper()._200smpl(),
            NumSamples::_240smpl => w.reportper()._240smpl(),
            NumSamples::_280smpl => w.reportper()._280smpl(),
            NumSamples::_1smpl => w.reportper()._1smpl(),
        });
        self.qdec.intenset.write(|w| w.reportrdy().set_bit());
        self
    }

    /// Disables the QDEC interrupt triggering.
    #[inline(always)]
    pub fn disable_interrupt(&self) -> &Self {
        self.qdec.intenclr.write(|w| w.reportrdy().set_bit());
        self
    }

    /// Enables the quadrature decoder.
    #[inline(always)]
    pub fn enable(&self) {
        self.qdec.enable.write(|w| w.enable().set_bit());
        self.qdec.tasks_start.write(|w| unsafe { w.bits(1) });
    }

    /// Disables the quadrature decoder.
    #[inline(always)]
    pub fn disable(&self) {
        self.qdec.tasks_stop.write(|w| unsafe { w.bits(1) });
        while self.qdec.events_stopped.read().bits() == 0 {}
        self.qdec.enable.write(|w| w.enable().clear_bit());
    }

    /// Returns the accumulated change since last read (-1024..1023).
    #[inline(always)]
    pub fn read(&self) -> i16 {
        self.qdec.tasks_readclracc.write(|w| unsafe { w.bits(1) });
        self.qdec.accread.read().bits() as i16
    }

    /// Consumes `self` and returns back the raw `QDEC` peripheral.
    #[inline]
    pub fn free(self) -> (QDEC, Pins) {
        let a = unsafe { Pin::from_psel_bits(self.qdec.psel.a.read().bits()) };
        let b = unsafe { Pin::from_psel_bits(self.qdec.psel.b.read().bits()) };
        let led = {
            let led = self.qdec.psel.led.read();
            if led.connect().is_connected() {
                Some(unsafe { Pin::from_psel_bits(led.bits()) })
            } else {
                None
            }
        };
        self.qdec.psel.a.reset();
        self.qdec.psel.b.reset();
        self.qdec.psel.led.reset();

        (self.qdec, Pins { a, b, led })
    }
}

/// Pins for the QDEC
pub struct Pins {
    pub a: Pin<Input<PullUp>>,
    pub b: Pin<Input<PullUp>>,
    pub led: Option<Pin<Input<PullUp>>>,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SamplePeriod {
    _128us,
    _256us,
    _512us,
    _1024us,
    _2048us,
    _4096us,
    _8192us,
    _16384us,
    _32ms,
    _65ms,
    _131ms,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum NumSamples {
    _10smpl,
    _40smpl,
    _80smpl,
    _120smpl,
    _160smpl,
    _200smpl,
    _240smpl,
    _280smpl,
    _1smpl,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum LedPolarity {
    ActiveHigh,
    ActiveLow,
}
