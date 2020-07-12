//! HAL interface to the WDT peripheral
//!
//! This HAL implements a basic watchdog timer with 1..=8 handles.
//! Once the watchdog has been started, it cannot be stopped.

use crate::pac::WDT;

/// A type state representing a watchdog that has not been started
pub struct Inactive;

/// A type state representing a watchdog that has been started and cannot be stopped
pub struct Active;

/// An interface to the Watchdog
pub struct Watchdog<T: sealed::WdMode> {
    wdt: WDT,
    _state: T,
}

/// An interface to feed the Watchdog
pub struct WatchdogHandle<T: sealed::HandleId>(T);

impl<T> WatchdogHandle<T>
where
    T: sealed::HandleId,
{
    /// Pet the watchdog
    ///
    /// This function pets the given watchdog handle.
    ///
    /// NOTE: All active handles must be pet within the time interval to
    /// prevent a reset from occuring.
    #[inline]
    pub fn pet(&mut self) {
        let hdl = unsafe { &*WDT::ptr() };
        hdl.rr[self.0.index()].write(|w| w.rr().reload());
    }

    /// Has this handle been pet within the current window?
    pub fn is_pet(&self) -> bool {
        let hdl = unsafe { &*WDT::ptr() };
        let rd = hdl.reqstatus.read().bits();
        let idx = self.0.index();
        debug_assert!(idx < 8, "Bad Index!");
        ((rd >> idx) & 0x1) == 0
    }

    /// Convert the handle into a generic handle
    ///
    /// This is useful if you need to place handles into an array
    pub fn degrade(self) -> WatchdogHandle<HdlN> {
        WatchdogHandle(HdlN {
            idx: self.0.index() as u8,
        })
    }
}

/// A type state representing Watchdog Handle 0
pub struct Hdl0;
/// A type state representing Watchdog Handle 1
pub struct Hdl1;
/// A type state representing Watchdog Handle 2
pub struct Hdl2;
/// A type state representing Watchdog Handle 3
pub struct Hdl3;
/// A type state representing Watchdog Handle 4
pub struct Hdl4;
/// A type state representing Watchdog Handle 5
pub struct Hdl5;
/// A type state representing Watchdog Handle 6
pub struct Hdl6;
/// A type state representing Watchdog Handle 7
pub struct Hdl7;

/// A structure that represents a runtime stored Watchdog Handle
pub struct HdlN {
    idx: u8,
}

/// A structure containing the active watchdog and all requested
/// Watchdog handles
pub struct Parts {
    pub wdt: Watchdog<Active>,
    pub hdl0: WatchdogHandle<Hdl0>,
    pub hdl1: Option<WatchdogHandle<Hdl1>>,
    pub hdl2: Option<WatchdogHandle<Hdl2>>,
    pub hdl3: Option<WatchdogHandle<Hdl3>>,
    pub hdl4: Option<WatchdogHandle<Hdl4>>,
    pub hdl5: Option<WatchdogHandle<Hdl5>>,
    pub hdl6: Option<WatchdogHandle<Hdl6>>,
    pub hdl7: Option<WatchdogHandle<Hdl7>>,
}

/// The number of watchdog handles to activate
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum NumHandles {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

impl Watchdog<Inactive> {
    /// Create a new watchdog instance from the peripheral
    pub fn new(wdt: WDT) -> Watchdog<Inactive> {
        Watchdog {
            wdt,
            _state: Inactive,
        }
    }

    /// Release the peripheral
    ///
    /// Note: The peripheral cannot be released after activation
    pub fn release(self) -> WDT {
        self.wdt
    }

    /// Activate the watchdog with the given number of handles
    ///
    /// The watchdog cannot be deactivated after starting.
    ///
    /// NOTE: All activated handles must be pet within the configured time interval to
    /// prevent a reset from occuring.
    pub fn activate(self, handles: NumHandles) -> Parts {
        self.wdt.rren.write(|w| unsafe {
            w.bits(match handles {
                NumHandles::One => 0b0000_0001,
                NumHandles::Two => 0b0000_0011,
                NumHandles::Three => 0b0000_0111,
                NumHandles::Four => 0b0000_1111,
                NumHandles::Five => 0b0001_1111,
                NumHandles::Six => 0b0011_1111,
                NumHandles::Seven => 0b0111_1111,
                NumHandles::Eight => 0b1111_1111,
            })
        });
        self.wdt.tasks_start.write(|w| unsafe { w.bits(1) });
        Parts {
            wdt: Watchdog {
                wdt: self.wdt,
                _state: Active,
            },
            hdl0: WatchdogHandle(Hdl0),
            hdl1: if handles >= NumHandles::Two {
                Some(WatchdogHandle(Hdl1))
            } else {
                None
            },
            hdl2: if handles >= NumHandles::Three {
                Some(WatchdogHandle(Hdl2))
            } else {
                None
            },
            hdl3: if handles >= NumHandles::Four {
                Some(WatchdogHandle(Hdl3))
            } else {
                None
            },
            hdl4: if handles >= NumHandles::Five {
                Some(WatchdogHandle(Hdl4))
            } else {
                None
            },
            hdl5: if handles >= NumHandles::Six {
                Some(WatchdogHandle(Hdl5))
            } else {
                None
            },
            hdl6: if handles >= NumHandles::Seven {
                Some(WatchdogHandle(Hdl6))
            } else {
                None
            },
            hdl7: if handles == NumHandles::Eight {
                Some(WatchdogHandle(Hdl7))
            } else {
                None
            },
        }
    }

    /// Enable the watchdog interrupt
    ///
    /// NOTE: Although the interrupt will occur, there is no way to prevent
    /// the reset from occuring. From the time the event was fired, the
    /// system will reset two LFCLK ticks later (61 microseconds) if the
    /// interrupt has been enabled.
    #[inline(always)]
    pub fn enable_interrupt(&mut self) {
        self.wdt.intenset.write(|w| w.timeout().set_bit());
    }

    /// Disable the watchdog interrupt
    ///
    /// NOTE: This has no effect on the reset caused by the Watchdog
    #[inline(always)]
    pub fn disable_interrupt(&mut self) {
        self.wdt.intenclr.write(|w| w.timeout().set_bit());
    }

    /// Set the number of 32.768kHz ticks in each watchdog period
    ///
    /// This value defaults to 0xFFFF_FFFF (1.5 days) on reset.
    ///
    /// Note: there is a minimum of 15 ticks (458 microseconds). If a lower
    /// number is provided, 15 ticks will be used as the configured value
    #[inline(always)]
    pub fn set_lfosc_ticks(&mut self, ticks: u32) {
        self.wdt
            .crv
            .write(|w| unsafe { w.bits(ticks.max(0x0000_000F)) });
    }

    /// Should the watchdog continue to count during sleep modes?
    ///
    /// This value defaults to ENABLED on reset.
    pub fn run_during_sleep(&self, setting: bool) {
        self.wdt.config.modify(|_r, w| w.sleep().bit(setting));
    }

    /// Should the watchdog continue to count when the CPU is halted for debug?
    ///
    /// This value defaults to DISABLED on reset.
    pub fn run_during_debug_halt(&self, setting: bool) {
        self.wdt.config.modify(|_r, w| w.halt().bit(setting));
    }
}

impl Watchdog<Active> {
    /// Is the watchdog still awaiting pets from any handle?
    ///
    /// This reports whether sufficient pets have been received from all
    /// handles to prevent a reset this time period
    #[inline(always)]
    pub fn awaiting_pets(&self) -> bool {
        let enabled = self.wdt.rren.read().bits();
        let status = self.wdt.reqstatus.read().bits();
        (status & enabled) == 0
    }
}

impl<T> Watchdog<T>
where
    T: sealed::WdMode,
{
    /// Is the watchdog active?
    #[inline(always)]
    pub fn is_running(&self) -> bool {
        // TODO: Should we just believe the type state?
        self.wdt.runstatus.read().runstatus().bit_is_set()
    }
}

mod sealed {
    pub trait HandleId {
        fn index(&self) -> usize;
    }

    pub trait WdMode {}
}

impl sealed::WdMode for Inactive {}
impl sealed::WdMode for Active {}

impl sealed::HandleId for Hdl0 {
    fn index(&self) -> usize {
        0
    }
}
impl sealed::HandleId for Hdl1 {
    fn index(&self) -> usize {
        1
    }
}
impl sealed::HandleId for Hdl2 {
    fn index(&self) -> usize {
        2
    }
}
impl sealed::HandleId for Hdl3 {
    fn index(&self) -> usize {
        3
    }
}
impl sealed::HandleId for Hdl4 {
    fn index(&self) -> usize {
        4
    }
}
impl sealed::HandleId for Hdl5 {
    fn index(&self) -> usize {
        5
    }
}
impl sealed::HandleId for Hdl6 {
    fn index(&self) -> usize {
        6
    }
}
impl sealed::HandleId for Hdl7 {
    fn index(&self) -> usize {
        7
    }
}
impl sealed::HandleId for HdlN {
    fn index(&self) -> usize {
        self.idx.into()
    }
}
