//! HAL interface for the PPI peripheral
//!
//! The Programmable Peripheral Interconnect interface allows for an autonomous interoperability
//! between peripherals through their events and tasks. There are fixed PPI channels and fully
//! configurable ones, fixed channels can only connect specific events to specific tasks. For fully
//! configurable channels, it is possible to choose, via software, the event and the task that it
//! will triggered by the event.
//!
//! On nRF52 devices, there is also a fork task endpoint, where the user can configure one more task
//! to be triggered by the same event, even fixed PPI channels have a configurable fork task.

use crate::target::PPI;
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "51")] {
        mod event_nrf51;
        mod task_nrf51;
    } else if #[cfg(feature = "52810")] {
        mod event_nrf52810;
        mod task_nrf52810;
    } else if #[cfg(feature = "52832")] {
        mod event_nrf52832;
        mod task_nrf52832;
    } else if #[cfg(feature = "52833")] {
        mod event_nrf52833;
        mod task_nrf52833;
    } else if #[cfg(feature = "52840")] {
        mod event_nrf52840;
        mod task_nrf52840;
    }
}

mod sealed {
    use super::{EventAddr, TaskAddr};

    pub trait Channel {
        const CH: usize;
    }

    pub trait Task {
        #[inline(always)]
        fn task_addr(&self) -> TaskAddr {
            TaskAddr(self as *const _ as *const u32 as u32)
        }
    }
    pub trait Event {
        #[inline(always)]
        fn event_addr(&self) -> EventAddr {
            EventAddr(self as *const _ as *const u32 as u32)
        }
    }

    pub trait NotFixed {}
}
use sealed::{Channel, Event, NotFixed, Task};

pub struct TaskAddr(pub(crate) u32);
pub struct EventAddr(pub(crate) u32);

/// Trait to represent a Programmable Peripheral Interconnect channel.
pub trait Ppi {
    /// Enables the channel.
    fn enable(&mut self);

    /// Disables the channel.
    fn disable(&mut self);

    #[cfg(not(feature = "51"))]
    /// Sets the fork task that must be triggered when the configured event occurs. The user must
    /// provide a reference to the task.
    fn set_fork_task_endpoint<T: Task>(&mut self, task: &T);
}

/// Traits that extends the [Ppi](trait.Ppi.html) trait, marking a channel as fully configurable.
pub trait ConfigurablePpi {
    /// Sets the task that must be triggered when the configured event occurs. The user must provide
    /// a reference to the task.
    fn set_task_endpoint<T: Task>(&mut self, task: &T);

    /// Sets the event that will trigger the chosen task(s). The user must provide a reference to
    /// the event.
    fn set_event_endpoint<E: Event>(&mut self, event: &E);
}

// All unsafe `ptr` calls only uses registers atomically, and only changes the resources owned by
// the type (guaranteed by the abstraction)
impl<P: Channel> Ppi for P {
    #[inline(always)]
    fn enable(&mut self) {
        let regs = unsafe { &*PPI::ptr() };
        regs.chenset.write(|w| unsafe { w.bits(1 << P::CH) });
    }

    #[inline(always)]
    fn disable(&mut self) {
        let regs = unsafe { &*PPI::ptr() };
        regs.chenclr.write(|w| unsafe { w.bits(1 << P::CH) });
    }

    #[cfg(not(feature = "51"))]
    #[inline(always)]
    fn set_fork_task_endpoint<T: Task>(&mut self, task: &T) {
        let regs = unsafe { &*PPI::ptr() };
        regs.fork[P::CH]
            .tep
            .write(|w| unsafe { w.bits(task.task_addr().0) });
    }
}

// All unsafe `ptr` calls only uses registers atomically, and only changes the resources owned by
// the type (guaranteed by the abstraction)
impl<P: Channel + NotFixed> ConfigurablePpi for P {
    #[inline(always)]
    fn set_task_endpoint<T: Task>(&mut self, task: &T) {
        let regs = unsafe { &*PPI::ptr() };
        regs.ch[P::CH]
            .tep
            .write(|w| unsafe { w.bits(task.task_addr().0) });
    }

    #[inline(always)]
    fn set_event_endpoint<E: Event>(&mut self, event: &E) {
        let regs = unsafe { &*PPI::ptr() };
        regs.ch[P::CH]
            .eep
            .write(|w| unsafe { w.bits(event.event_addr().0) });
    }
}

macro_rules! ppi {
    (
        not_fixed: [ $(
            $(#[$attr:meta])*
            ($ppix:ident, $PpixType:ident, $ch:expr),)+
        ],
        fixed: [$(($ppix_fixed:ident, $PpixTypeFixed:ident, $ch_fixed:expr),)+],
    ) => {

        $(
            /// Fully configurable PPI Channel.
            $(#[$attr])*
            pub struct $PpixType {
                _private: (),
            }

            $(#[$attr])*
            impl Channel for $PpixType {
                const CH: usize = $ch;
            }

            $(#[$attr])*
            impl NotFixed for $PpixType {}
        )+

        $(
            /// Fixed PPI channel.
            pub struct $PpixTypeFixed {
                _private: (),
            }

            impl Channel for $PpixTypeFixed {
                const CH: usize = $ch_fixed;
            }
        )+

        /// Type that abstracts all the PPI channels.
        pub struct Parts {
            $(
                $(#[$attr])*
                pub $ppix: $PpixType,
            )+
            $(
                pub $ppix_fixed: $PpixTypeFixed,
            )+
        }

        impl Parts {
            /// Gets access to the PPI abstraction, making it possible to separate the channels through
            /// different objects.
            pub fn new(_regs: PPI) -> Self {
                Self {
                    $(
                        $(#[$attr])*
                        $ppix: $PpixType {
                            _private: (),
                        },
                    )+
                    $(
                        $ppix_fixed: $PpixTypeFixed {
                            _private: (),
                        },
                    )+
                }
            }
        }
    };
}

ppi!(
    not_fixed: [
        (ppi0, Ppi0, 0),
        (ppi1, Ppi1, 1),
        (ppi2, Ppi2, 2),
        (ppi3, Ppi3, 3),
        (ppi4, Ppi4, 4),
        (ppi5, Ppi5, 5),
        (ppi6, Ppi6, 6),
        (ppi7, Ppi7, 7),
        (ppi8, Ppi8, 8),
        (ppi9, Ppi9, 9),
        (ppi10, Ppi10, 10),
        (ppi11, Ppi11, 11),
        (ppi12, Ppi12, 12),
        (ppi13, Ppi13, 13),
        (ppi14, Ppi14, 14),
        (ppi15, Ppi15, 15),
        #[cfg(not(feature = "51"))]
        (ppi16, Ppi16, 16),
        #[cfg(not(feature = "51"))]
        (ppi17, Ppi17, 17),
        #[cfg(not(feature = "51"))]
        (ppi18, Ppi18, 18),
        #[cfg(not(feature = "51"))]
        (ppi19, Ppi19, 19),
    ],
    fixed: [
        (ppi20, Ppi20, 20),
        (ppi21, Ppi21, 21),
        (ppi22, Ppi22, 22),
        (ppi23, Ppi23, 23),
        (ppi24, Ppi24, 24),
        (ppi25, Ppi25, 25),
        (ppi26, Ppi26, 26),
        (ppi27, Ppi27, 27),
        (ppi28, Ppi28, 28),
        (ppi29, Ppi29, 29),
        (ppi30, Ppi30, 30),
        (ppi31, Ppi31, 31),
    ],
);
