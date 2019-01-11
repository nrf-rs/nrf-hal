pub mod blocking;
pub mod nonblocking;

pub use crate::uarte::blocking::*;

// Re-export SVD variants to allow user to directly set values
pub use crate::target::uarte0::{
    baudrate::BAUDRATEW as Baudrate,
    config::PARITYW as Parity,
};

use crate::gpio::{
    p0::P0_Pin,
    Output,
    PushPull,
};

pub struct Pins {
    pub rxd: P0_Pin<Output<PushPull>>,
    pub txd: P0_Pin<Output<PushPull>>,
    pub cts: Option<P0_Pin<Output<PushPull>>>,
    pub rts: Option<P0_Pin<Output<PushPull>>>,
}


#[derive(Debug)]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    Transmit,
    Receive,
}
