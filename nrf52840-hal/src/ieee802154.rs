//! IEEE 802.15.4 radio

use core::{
    cmp,
    ops::{self, RangeFrom},
    sync::atomic::{self, Ordering},
};

use crate::clocks::{Clocks, ExternalOscillator};
use crate::pac::{generic::Variant, radio::state::STATE_A, RADIO};

pub struct Radio<'c> {
    radio: RADIO,
    // used to freeze `Clocks`
    _clocks: &'c (),
}

/// Default Clear Channel Assessment method = Carrier sense
pub const DEFAULT_CCA: Cca = Cca::CarrierSense;

/// Default radio channel = Channel 20 (2_450 MHz)
pub const DEFAULT_CHANNEL: Channel = Channel::_20;

// TODO expose the other variants in `pac::CCAMODE_A`
/// Clear Channel Assessment method
pub enum Cca {
    /// Carrier sense
    CarrierSense,
}

/// IEEE 802.15.4 channels
///
/// NOTE these are NOT the same as WiFi 2.4 GHz channels
pub enum Channel {
    /// 2_405 MHz
    _11 = 5,
    /// 2_410 MHz
    _12 = 10,
    /// 2_415 MHz
    _13 = 15,
    /// 2_420 MHz
    _14 = 20,
    /// 2_425 MHz
    _15 = 25,
    /// 2_430 MHz
    _16 = 30,
    /// 2_435 MHz
    _17 = 35,
    /// 2_440 MHz
    _18 = 40,
    /// 2_445 MHz
    _19 = 45,
    /// 2_450 MHz
    _20 = 50,
    /// 2_455 MHz
    _21 = 55,
    /// 2_460 MHz
    _22 = 60,
    /// 2_465 MHz
    _23 = 65,
    /// 2_470 MHz
    _24 = 70,
    /// 2_475 MHz
    _25 = 75,
    /// 2_480 MHz
    _26 = 80,
}

// TODO add API to change TXPOWER
impl<'c> Radio<'c> {
    /// Initializes the radio for IEEE 802.15.4 operation
    pub fn init<L, LSTAT>(radio: RADIO, _clocks: &'c Clocks<ExternalOscillator, L, LSTAT>) -> Self {
        let mut radio = Self { radio, _clocks: &() };

        // go to a known state
        radio.disable();

        // clear any event interesting to us
        radio.radio.events_disabled.reset();
        radio.radio.events_end.reset();

        radio.radio.mode.write(|w| w.mode().ieee802154_250kbit());

        // NOTE(unsafe) radio is currently disabled
        unsafe {
            radio.radio.pcnf0.write(|w| {
                w.s1incl()
                    .clear_bit() // S1 not included in RAM
                    .plen()
                    ._32bit_zero() // 32-bit zero preamble
                    .crcinc()
                    .exclude() // no CRC
                    .cilen()
                    .bits(0) // no code indicator
                    .lflen()
                    .bits(8) // length = 8 bits
                    .s0len()
                    .clear_bit() // no S0
                    .s1len()
                    .bits(0) // no S1
            });

            radio.radio.pcnf1.write(|w| {
                w.maxlen()
                    .bits(1 /* PHY_HDR */ + Packet::MAX_LEN) // LQI is not transmitted
                    .statlen()
                    .bits(0) // no static length
                    .balen()
                    .bits(0) // no base address
                    .endian()
                    .clear_bit() // little endian
                    .whiteen()
                    .clear_bit() // no data whitenining
            });

            // CRC configuration required by the IEEE spec: x**16 + x**12 + x**5 + 1
            radio
                .radio
                .crccnf
                .write(|w| w.len().two().skipaddr().ieee802154());
            radio.radio.crcpoly.write(|w| w.crcpoly().bits(0x11021));
            radio.radio.crcinit.write(|w| w.crcinit().bits(0));

            // permanent shortcuts
            radio
                .radio
                .shorts
                .write(|w| w.ccaidle_txen().set_bit().txready_start().set_bit());
        }

        // set default settings
        radio.set_channel(DEFAULT_CHANNEL);
        radio.set_cca(DEFAULT_CCA);

        radio
    }

    /// Changes the radio channel
    pub fn set_channel(&mut self, channel: Channel) {
        self.disable();

        // NOTE(unsafe) radio is currently disabled
        unsafe {
            self.radio
                .frequency
                .write(|w| w.map().clear_bit().frequency().bits(channel as u8))
        }
    }

    /// Changes the Clear Channel Assessment method
    pub fn set_cca(&mut self, cca: Cca) {
        self.disable();

        match cca {
            Cca::CarrierSense => self.radio.ccactrl.write(|w| w.ccamode().carrier_mode()),
        }
    }

    /// Recevies one radio packet and copies its contents into the given `packet` buffer
    pub fn recv(&mut self, packet: &mut Packet) {
        // go to the RXIDLE state
        self.enable_rx();

        // NOTE(unsafe) DMA transfer has not yet started
        // set up RX buffer
        unsafe {
            self.radio
                .packetptr
                .write(|w| w.packetptr().bits(packet.buffer.as_mut_ptr() as u32));
        }

        // start transfer
        dma_start_fence();
        self.radio.tasks_start.write(|w| w.tasks_start().set_bit());

        // wait until we have received something
        self.wait_for_event(Event::End);
        dma_end_fence();

        // the CRC is checked by the hardware; nothing else to do
    }

    /// Sends the given `data` as a single radio packet
    pub fn send(&mut self, packet: &Packet) {
        // go to the RXIDLE state
        self.enable_rx();

        // NOTE the DMA doesn't exactly start at this point but due to shortcuts it may occur at any
        // point after this volatile write
        dma_start_fence();
        // NOTE(unsafe) DMA transfer has not yet started
        unsafe {
            self.radio
                .packetptr
                .write(|w| w.packetptr().bits(packet.buffer.as_ptr() as u32));
        }

        // start CCA
        'cca: loop {
            self.radio
                .tasks_ccastart
                .write(|w| w.tasks_ccastart().set_bit());

            loop {
                if self
                    .radio
                    .events_ccaidle
                    .read()
                    .events_ccaidle()
                    .bit_is_set()
                {
                    // channel is clear
                    self.radio.events_ccaidle.reset();
                    break 'cca;
                }

                if self
                    .radio
                    .events_ccabusy
                    .read()
                    .events_ccabusy()
                    .bit_is_set()
                {
                    // channel is busy
                    self.radio.events_ccaidle.reset();
                    // FIXME according to the IEEE spec there should be a backoff delay before
                    // the next CCA
                    continue 'cca;
                }
            }
        }

        // due to a shortcut the transmission will start automatically so we just have to wait
        // until the END event
        self.wait_for_event(Event::End);
        dma_end_fence();
    }

    /// Moves the radio from any state to the DISABLED state
    fn disable(&mut self) {
        // See figure 110 in nRF52840-PS
        loop {
            if let Variant::Val(state) = self.radio.state.read().state().variant() {
                match state {
                    STATE_A::DISABLED => return,

                    STATE_A::RXRU | STATE_A::RXIDLE | STATE_A::TXRU | STATE_A::TXIDLE => {
                        self.radio
                            .tasks_disable
                            .write(|w| w.tasks_disable().set_bit());

                        self.wait_for_state(STATE_A::DISABLED);
                        return;
                    }

                    // ramping down
                    STATE_A::RXDISABLE | STATE_A::TXDISABLE => {
                        self.wait_for_state(STATE_A::DISABLED);
                        return;
                    }

                    // cancel ongoing transfer
                    STATE_A::RX => {
                        self.radio.tasks_stop.write(|w| w.tasks_stop().set_bit());
                        self.wait_for_state(STATE_A::RXIDLE);
                    }
                    STATE_A::TX => {
                        self.radio.tasks_stop.write(|w| w.tasks_stop().set_bit());
                        self.wait_for_state(STATE_A::TXIDLE);
                    }
                }
            } else {
                // STATE register is in an invalid state
                unreachable!()
            }
        }
    }

    /// Moves the radio from any state to the RXEN state
    fn enable_rx(&mut self) {
        // See figure 110 in nRF52840-PS
        loop {
            if let Variant::Val(state) = self.radio.state.read().state().variant() {
                match state {
                    STATE_A::RXIDLE => return,

                    STATE_A::DISABLED => {
                        self.radio.tasks_rxen.write(|w| w.tasks_rxen().set_bit());
                        self.wait_for_state(STATE_A::RXIDLE);
                        return;
                    }

                    // ramping up
                    STATE_A::RXRU => {
                        self.wait_for_state(STATE_A::RXIDLE);
                        return;
                    }

                    // NOTE to avoid errata 204 (see rev1 v1.4) we first go to the DISABLED state
                    STATE_A::TXIDLE | STATE_A::TXRU => {
                        self.radio
                            .tasks_disable
                            .write(|w| w.tasks_disable().set_bit());
                        self.wait_for_state(STATE_A::DISABLED);
                    }

                    // ramping down
                    STATE_A::RXDISABLE | STATE_A::TXDISABLE => {
                        self.wait_for_state(STATE_A::DISABLED);
                    }

                    // cancel ongoing transfer
                    STATE_A::RX => {
                        self.radio.tasks_stop.write(|w| w.tasks_stop().set_bit());
                        self.wait_for_state(STATE_A::RXIDLE);
                        return;
                    }
                    STATE_A::TX => {
                        self.radio.tasks_stop.write(|w| w.tasks_stop().set_bit());
                        self.wait_for_state(STATE_A::TXIDLE);
                    }
                }
            } else {
                // STATE register is in an invalid state
                unreachable!()
            }
        }
    }

    fn wait_for_event(&self, event: Event) {
        match event {
            Event::End => {
                while self.radio.events_end.read().events_end().bit_is_clear() {
                    continue;
                }
                self.radio.events_end.reset();
            }
        }
    }

    /// Waits until the radio state matches the given `state`
    fn wait_for_state(&self, state: STATE_A) {
        while self.radio.state.read().state() != state {
            continue;
        }
    }
}

/// NOTE must be followed by a volatile write operation
fn dma_start_fence() {
    atomic::compiler_fence(Ordering::Release);
}

/// NOTE must be preceded by a volatile read operation
fn dma_end_fence() {
    atomic::compiler_fence(Ordering::Acquire);
}

enum Event {
    End,
}

/// An IEEE 802.15.4 packet
pub struct Packet {
    buffer: [u8; Self::SIZE],
}

// See figure 124 in nRF52840-PS
impl Packet {
    const PHY_HDR: usize = 0;
    const DATA: RangeFrom<usize> = 1..;
    // TODO add API to extract the LQI (Link Quality Indicator)
    const SIZE: usize = 1 /* PHY_HDR */ + Self::MAX_LEN as usize + 1 /* LQI */;

    /// The maximum length of a packet
    pub const MAX_LEN: u8 = 127;

    /// Returns an empty packe
    pub fn new() -> Self {
        Self {
            buffer: [0; Self::SIZE],
        }
    }

    /// Fills the packet with given `src` data
    ///
    /// NOTE `src` data will be truncated to `MAX_PACKET_SIZE` bytes
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        let len = cmp::min(src.len(), Self::MAX_LEN as usize) as u8;
        self.buffer[Self::DATA][..len as usize].copy_from_slice(src);
        self.buffer[Self::PHY_HDR] = len;
    }

    /// Returns the size of this packet
    pub fn len(&self) -> u8 {
        self.buffer[Self::PHY_HDR]
    }

    /// Changes the `len` of the packet
    ///
    /// NOTE `len` will be truncated to `MAX_PACKET_SIZE` bytes
    pub fn set_len(&mut self, len: u8) {
        let len = cmp::min(len, Self::MAX_LEN);
        self.buffer[Self::PHY_HDR] = len;
    }
}

impl ops::Deref for Packet {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.buffer[Self::DATA][..self.len() as usize]
    }
}

impl ops::DerefMut for Packet {
    fn deref_mut(&mut self) -> &mut [u8] {
        let len = self.len();
        &mut self.buffer[Self::DATA][..len as usize]
    }
}
