//! HAL interface to the WDT peripheral.
//!
//! This HAL implements a basic watchdog timer with 1..=8 handles.
//! Once the watchdog has been started, it cannot be stopped.

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(any(feature = "9160", feature = "5340-net"))] {
        use crate::pac::WDT_NS as WDT;
    } else if #[cfg(feature = "5340-app")] {
        use crate::pac::WDT0_NS as WDT;
    } else {
        use crate::pac::WDT;
    }
}

use handles::*;

/// A type state representing a watchdog that has not been started.
pub struct Inactive;

/// A type state representing a watchdog that has been started and cannot be stopped.
pub struct Active;

/// An interface to the Watchdog.
pub struct Watchdog<T: sealed::WdMode> {
    wdt: WDT,
    _state: T,
}

/// A structure containing the active watchdog and all requested Watchdog handles.
pub struct Parts<T> {
    pub watchdog: Watchdog<Active>,
    pub handles: T,
}

/// An interface to feed the Watchdog.
pub struct WatchdogHandle<T: sealed::HandleId>(T);

impl<T> WatchdogHandle<T>
where
    T: sealed::HandleId,
{
    /// Pet the watchdog.
    ///
    /// This function pets the given watchdog handle.
    ///
    /// NOTE: All active handles must be pet within the time interval to
    /// prevent a reset from occurring.
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

    /// Convert the handle into a generic handle.
    ///
    /// This is useful if you need to place handles into an array.
    pub fn degrade(self) -> WatchdogHandle<HdlN> {
        WatchdogHandle(HdlN {
            idx: self.0.index() as u8,
        })
    }
}

impl Watchdog<Inactive> {
    /// Try to create a new watchdog instance from the peripheral.
    ///
    /// This function will return an error if the watchdog has already
    /// been activated, which may happen on a (non-watchdog) soft reset.
    /// In this case, it may be possible to still obtain the handles with
    /// the `Watchdog::try_recover()` method.
    ///
    /// If the watchdog has already started, configuration is no longer possible.
    #[inline]
    pub fn try_new(wdt: WDT) -> Result<Watchdog<Inactive>, WDT> {
        let watchdog = Watchdog {
            wdt,
            _state: Inactive,
        };

        if watchdog.is_active() {
            Err(watchdog.wdt)
        } else {
            Ok(watchdog)
        }
    }

    /// Release the peripheral.
    ///
    /// Note: The peripheral cannot be released after activation.
    #[inline]
    pub fn release(self) -> WDT {
        self.wdt
    }

    /// Activate the watchdog with the given number of handles.
    ///
    /// The watchdog cannot be deactivated after starting.
    ///
    /// NOTE: All activated handles must be pet within the configured time interval to
    /// prevent a reset from occurring.
    pub fn activate<H: sealed::Handles>(self) -> Parts<H::Handles> {
        self.wdt.rren.write(|w| unsafe { w.bits(H::ENABLE) });
        self.wdt.tasks_start.write(|w| unsafe { w.bits(1) });
        Parts {
            watchdog: Watchdog {
                wdt: self.wdt,
                _state: Active,
            },
            handles: H::create_handle(),
        }
    }

    /// Enable the watchdog interrupt.
    ///
    /// NOTE: Although the interrupt will occur, there is no way to prevent
    /// the reset from occurring. From the time the event was fired, the
    /// system will reset two LFCLK ticks later (61 microseconds) if the
    /// interrupt has been enabled.
    #[inline(always)]
    pub fn enable_interrupt(&mut self) {
        self.wdt.intenset.write(|w| w.timeout().set_bit());
    }

    /// Disable the watchdog interrupt.
    ///
    /// NOTE: This has no effect on the reset caused by the Watchdog.
    #[inline(always)]
    pub fn disable_interrupt(&mut self) {
        self.wdt.intenclr.write(|w| w.timeout().set_bit());
    }

    /// Set the number of 32.768kHz ticks in each watchdog period.
    ///
    /// This value defaults to 0xFFFF_FFFF (1.5 days) on reset.
    ///
    /// Note: there is a minimum of 15 ticks (458 microseconds). If a lower
    /// number is provided, 15 ticks will be used as the configured value.
    #[inline(always)]
    pub fn set_lfosc_ticks(&mut self, ticks: u32) {
        self.wdt
            .crv
            .write(|w| unsafe { w.bits(ticks.max(0x0000_000F)) });
    }

    /// Should the watchdog continue to count during sleep modes?
    ///
    /// This value defaults to ENABLED on reset.
    #[inline]
    pub fn run_during_sleep(&self, setting: bool) {
        self.wdt.config.modify(|_r, w| w.sleep().bit(setting));
    }

    /// Should the watchdog continue to count when the CPU is halted for debug?
    ///
    /// This value defaults to DISABLED on reset.
    #[inline]
    pub fn run_during_debug_halt(&self, setting: bool) {
        self.wdt.config.modify(|_r, w| w.halt().bit(setting));
    }
}

impl Watchdog<Active> {
    /// Is the watchdog still awaiting pets from any handle?
    ///
    /// This reports whether sufficient pets have been received from all
    /// handles to prevent a reset this time period.
    #[inline(always)]
    pub fn awaiting_pets(&self) -> bool {
        let enabled = self.wdt.rren.read().bits();
        let status = self.wdt.reqstatus.read().bits();
        (status & enabled) == 0
    }

    /// Try to recover a handle to an already running watchdog. If the
    /// number of requested handles matches the activated number of handles,
    /// an activated handle will be returned. Otherwise the peripheral will
    /// be returned.
    ///
    /// NOTE: Since the watchdog is already counting, you want to pet these dogs
    /// as soon as possible!
    pub fn try_recover<H: sealed::Handles>(wdt: WDT) -> Result<Parts<H::Handles>, WDT> {
        // Do we have the same number of handles at least?
        if wdt.rren.read().bits() == H::ENABLE {
            Ok(Parts {
                watchdog: Watchdog {
                    wdt,
                    _state: Active,
                },
                handles: H::create_handle(),
            })
        } else {
            Err(wdt)
        }
    }
}

impl<T> Watchdog<T>
where
    T: sealed::WdMode,
{
    /// Is the watchdog active?
    #[inline(always)]
    pub fn is_active(&self) -> bool {
        cfg_if! {
            if #[cfg(any(feature = "9160", feature = "5340-app", feature = "5340-net"))] {
                self.wdt.runstatus.read().runstatuswdt().bit_is_set()
            } else {
                self.wdt.runstatus.read().runstatus().bit_is_set()
            }
        }
    }
}

mod sealed {
    pub trait HandleId {
        fn index(&self) -> usize;
    }

    pub trait Handles {
        type Handles;
        const ENABLE: u32;
        fn create_handle() -> Self::Handles;
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

pub mod handles {
    //! Type states representing individual watchdog handles.

    /// A type state representing Watchdog Handle 0.
    pub struct Hdl0;
    /// A type state representing Watchdog Handle 1.
    pub struct Hdl1;
    /// A type state representing Watchdog Handle 2.
    pub struct Hdl2;
    /// A type state representing Watchdog Handle 3.
    pub struct Hdl3;
    /// A type state representing Watchdog Handle 4.
    pub struct Hdl4;
    /// A type state representing Watchdog Handle 5.
    pub struct Hdl5;
    /// A type state representing Watchdog Handle 6.
    pub struct Hdl6;
    /// A type state representing Watchdog Handle 7.
    pub struct Hdl7;

    /// A structure that represents a runtime stored Watchdog Handle.
    pub struct HdlN {
        pub(super) idx: u8,
    }
}

pub mod count {
    //! Type states representing the number of requested handles.

    use super::{sealed::Handles, Hdl0, Hdl1, Hdl2, Hdl3, Hdl4, Hdl5, Hdl6, Hdl7, WatchdogHandle};
    /// A type state representing the request for One handles.
    pub struct One;
    /// A type state representing the request for Two handles.
    pub struct Two;
    /// A type state representing the request for Three handles.
    pub struct Three;
    /// A type state representing the request for Four handles.
    pub struct Four;
    /// A type state representing the request for Five handles.
    pub struct Five;
    /// A type state representing the request for Six handles.
    pub struct Six;
    /// A type state representing the request for Seven handles.
    pub struct Seven;
    /// A type state representing the request for Eight handles.
    pub struct Eight;

    impl Handles for One {
        type Handles = (WatchdogHandle<Hdl0>,);
        const ENABLE: u32 = 0b0000_0001;
        fn create_handle() -> Self::Handles {
            (WatchdogHandle(Hdl0),)
        }
    }
    impl Handles for Two {
        type Handles = (WatchdogHandle<Hdl0>, WatchdogHandle<Hdl1>);
        const ENABLE: u32 = 0b0000_0011;
        fn create_handle() -> Self::Handles {
            (WatchdogHandle(Hdl0), WatchdogHandle(Hdl1))
        }
    }
    impl Handles for Three {
        type Handles = (
            WatchdogHandle<Hdl0>,
            WatchdogHandle<Hdl1>,
            WatchdogHandle<Hdl2>,
        );
        const ENABLE: u32 = 0b0000_0111;
        fn create_handle() -> Self::Handles {
            (
                WatchdogHandle(Hdl0),
                WatchdogHandle(Hdl1),
                WatchdogHandle(Hdl2),
            )
        }
    }
    impl Handles for Four {
        type Handles = (
            WatchdogHandle<Hdl0>,
            WatchdogHandle<Hdl1>,
            WatchdogHandle<Hdl2>,
            WatchdogHandle<Hdl3>,
        );
        const ENABLE: u32 = 0b0000_1111;
        fn create_handle() -> Self::Handles {
            (
                WatchdogHandle(Hdl0),
                WatchdogHandle(Hdl1),
                WatchdogHandle(Hdl2),
                WatchdogHandle(Hdl3),
            )
        }
    }
    impl Handles for Five {
        type Handles = (
            WatchdogHandle<Hdl0>,
            WatchdogHandle<Hdl1>,
            WatchdogHandle<Hdl2>,
            WatchdogHandle<Hdl3>,
            WatchdogHandle<Hdl4>,
        );
        const ENABLE: u32 = 0b0001_1111;
        fn create_handle() -> Self::Handles {
            (
                WatchdogHandle(Hdl0),
                WatchdogHandle(Hdl1),
                WatchdogHandle(Hdl2),
                WatchdogHandle(Hdl3),
                WatchdogHandle(Hdl4),
            )
        }
    }
    impl Handles for Six {
        type Handles = (
            WatchdogHandle<Hdl0>,
            WatchdogHandle<Hdl1>,
            WatchdogHandle<Hdl2>,
            WatchdogHandle<Hdl3>,
            WatchdogHandle<Hdl4>,
            WatchdogHandle<Hdl5>,
        );
        const ENABLE: u32 = 0b0011_1111;
        fn create_handle() -> Self::Handles {
            (
                WatchdogHandle(Hdl0),
                WatchdogHandle(Hdl1),
                WatchdogHandle(Hdl2),
                WatchdogHandle(Hdl3),
                WatchdogHandle(Hdl4),
                WatchdogHandle(Hdl5),
            )
        }
    }
    impl Handles for Seven {
        type Handles = (
            WatchdogHandle<Hdl0>,
            WatchdogHandle<Hdl1>,
            WatchdogHandle<Hdl2>,
            WatchdogHandle<Hdl3>,
            WatchdogHandle<Hdl4>,
            WatchdogHandle<Hdl5>,
            WatchdogHandle<Hdl6>,
        );
        const ENABLE: u32 = 0b0111_1111;
        fn create_handle() -> Self::Handles {
            (
                WatchdogHandle(Hdl0),
                WatchdogHandle(Hdl1),
                WatchdogHandle(Hdl2),
                WatchdogHandle(Hdl3),
                WatchdogHandle(Hdl4),
                WatchdogHandle(Hdl5),
                WatchdogHandle(Hdl6),
            )
        }
    }
    impl Handles for Eight {
        type Handles = (
            WatchdogHandle<Hdl0>,
            WatchdogHandle<Hdl1>,
            WatchdogHandle<Hdl2>,
            WatchdogHandle<Hdl3>,
            WatchdogHandle<Hdl4>,
            WatchdogHandle<Hdl5>,
            WatchdogHandle<Hdl6>,
            WatchdogHandle<Hdl7>,
        );
        const ENABLE: u32 = 0b1111_1111;
        fn create_handle() -> Self::Handles {
            (
                WatchdogHandle(Hdl0),
                WatchdogHandle(Hdl1),
                WatchdogHandle(Hdl2),
                WatchdogHandle(Hdl3),
                WatchdogHandle(Hdl4),
                WatchdogHandle(Hdl5),
                WatchdogHandle(Hdl6),
                WatchdogHandle(Hdl7),
            )
        }
    }
}
