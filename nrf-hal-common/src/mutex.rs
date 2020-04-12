//! Wrapper around the MUTEX peripheral.
//!
//! Only available on the nRF5340 CPUs.

#[cfg(feature = "5340-app")]
use crate::target::MUTEX_S as MutexPeriph;

#[cfg(feature = "5340-net")]
use crate::target::APPMUTEX_NS as MutexPeriph;

use core::marker::PhantomData;
use void::ResultVoidExt;

macro_rules! generate {
    (
        $(
            $field:ident $ty:ident $index:literal,
        )+
    ) => {
        $(
            pub struct $ty {}

            impl $ty {
                pub fn try_lock(&self) -> nb::Result<Guard<'_>, void::Void> {
                    let bits = unsafe {
                        (*MutexPeriph::ptr()).mutex[$index].read().bits()
                    };
                    if bits == 0 {
                        Ok(Guard {
                            index: $index,
                            _p: PhantomData,
                        })
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }

                pub fn lock(&self) -> Guard<'_> {
                    nb::block!(self.try_lock()).void_unwrap()
                }

                pub fn index(&self) -> u8 {
                    $index
                }
            }
        )+

        pub struct Mutexes {
            $( pub $field : $ty ),+
        }

        impl MutexExt for MutexPeriph {
            fn split(self) -> Mutexes {
                Mutexes {
                    $(
                        $field: $ty {},
                    )+
                }
            }
        }
    };
}

generate! {
    mutex0 Mutex0 0,
    mutex1 Mutex1 1,
    mutex2 Mutex2 2,
    mutex3 Mutex3 3,
    mutex4 Mutex4 4,
    mutex5 Mutex5 5,
    mutex6 Mutex6 6,
    mutex7 Mutex7 7,
    mutex8 Mutex8 8,
    mutex9 Mutex9 9,
    mutex10 Mutex10 10,
    mutex11 Mutex11 11,
    mutex12 Mutex12 12,
    mutex13 Mutex13 13,
    mutex14 Mutex14 14,
    mutex15 Mutex15 15,
}

pub struct Guard<'a> {
    index: u8,
    _p: PhantomData<&'a ()>,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        unsafe {
            (*MutexPeriph::ptr()).mutex[usize::from(self.index)].write(|w| w.mutex().clear_bit());
        }
    }
}

pub trait MutexExt {
    fn split(self) -> Mutexes;
}
