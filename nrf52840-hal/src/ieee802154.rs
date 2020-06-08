//! IEEE 802.15.4 radio

use core::{
    cmp,
    ops::{self, RangeFrom},
    sync::atomic::{self, Ordering},
};

use crate::clocks::{Clocks, ExternalOscillator};
use crate::pac::{
    generic::Variant,
    radio::{state::STATE_A, txpower::TXPOWER_A},
    RADIO,
};

/// IEEE 802.15.4 radio
pub struct Radio<'c> {
    radio: RADIO,
    // used to freeze `Clocks`
    _clocks: &'c (),
}

/// Default Clear Channel Assessment method = Carrier sense
pub const DEFAULT_CCA: Cca = Cca::CarrierSense;

/// Default radio channel = Channel 20 (`2_450` MHz)
pub const DEFAULT_CHANNEL: Channel = Channel::_20;

/// Default TX power = 0 dBm
pub const DEFAULT_TXPOWER: TxPower = TxPower::_0dBm;

/// Default Start of Frame Delimiter = `0xA7` (IEEE compliant)
pub const DEFAULT_SFD: u8 = 0xA7;

// TODO expose the other variants in `pac::CCAMODE_A`
/// Clear Channel Assessment method
pub enum Cca {
    /// Carrier sense
    CarrierSense,
}

/// IEEE 802.15.4 channels
///
/// NOTE these are NOT the same as WiFi 2.4 GHz channels
// TODO it is possible to use non-standard frequencies below 2_400 MHz; should those be exposed too?
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

/// Transmission power in dBm (decibel milliwatt)
// TXPOWERA enum minus the deprecated Neg30dBm variant and with better docs
#[derive(Clone, Copy, PartialEq)]
pub enum TxPower {
    /// +8 dBm
    Pos8dBm,
    /// +7 dBm
    Pos7dBm,
    /// +6 dBm (~4 mW)
    Pos6dBm,
    /// +5 dBm
    Pos5dBm,
    /// +4 dBm
    Pos4dBm,
    /// +3 dBm (~2 mW)
    Pos3dBm,
    /// +2 dBm
    Pos2dBm,
    /// 0 dBm (1 mW)
    _0dBm,
    /// -4 dBm
    Neg4dBm,
    /// -8 dBm
    Neg8dBm,
    /// -12 dBm
    Neg12dBm,
    /// -16 dBm
    Neg16dBm,
    /// -20 dBm (10 μW)
    Neg20dBm,
    /// -40 dBm (0.1 μW)
    Neg40dBm,
}

impl TxPower {
    fn _into(self) -> TXPOWER_A {
        match self {
            TxPower::Neg40dBm => TXPOWER_A::NEG40DBM,
            TxPower::Neg20dBm => TXPOWER_A::NEG20DBM,
            TxPower::Neg16dBm => TXPOWER_A::NEG16DBM,
            TxPower::Neg12dBm => TXPOWER_A::NEG12DBM,
            TxPower::Neg8dBm => TXPOWER_A::NEG8DBM,
            TxPower::Neg4dBm => TXPOWER_A::NEG4DBM,
            TxPower::_0dBm => TXPOWER_A::_0DBM,
            TxPower::Pos2dBm => TXPOWER_A::POS2DBM,
            TxPower::Pos3dBm => TXPOWER_A::POS3DBM,
            TxPower::Pos4dBm => TXPOWER_A::POS4DBM,
            TxPower::Pos5dBm => TXPOWER_A::POS5DBM,
            TxPower::Pos6dBm => TXPOWER_A::POS6DBM,
            TxPower::Pos7dBm => TXPOWER_A::POS7DBM,
            TxPower::Pos8dBm => TXPOWER_A::POS8DBM,
        }
    }
}

impl<'c> Radio<'c> {
    /// Initializes the radio for IEEE 802.15.4 operation
    pub fn init<L, LSTAT>(radio: RADIO, _clocks: &'c Clocks<ExternalOscillator, L, LSTAT>) -> Self {
        let mut radio = Self {
            radio,
            _clocks: &(),
        };

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
                    .include() // LENGTH field includes CRC (2 bytes)
                    .cilen()
                    .bits(0) // no code indicator
                    .lflen()
                    .bits(8) // length = 8 bits (but highest bit is reserved and must be `0`)
                    .s0len()
                    .clear_bit() // no S0
                    .s1len()
                    .bits(0) // no S1
            });

            radio.radio.pcnf1.write(|w| {
                w.maxlen()
                    .bits(Packet::MAX_LEN + 2 /* CRC */) // payload length
                    .statlen()
                    .bits(0) // no static length
                    .balen()
                    .bits(0) // no base address
                    .endian()
                    .clear_bit() // little endian
                    .whiteen()
                    .clear_bit() // no data whitening
            });

            // CRC configuration required by the IEEE spec: x**16 + x**12 + x**5 + 1
            radio
                .radio
                .crccnf
                .write(|w| w.len().two().skipaddr().ieee802154());
            radio.radio.crcpoly.write(|w| w.crcpoly().bits(0x11021));
            radio.radio.crcinit.write(|w| w.crcinit().bits(0));

            // permanent shortcuts
            radio.radio.shorts.write(|w| w.txready_start().set_bit());
        }

        // set default settings
        radio.set_cca(DEFAULT_CCA);
        radio.set_channel(DEFAULT_CHANNEL);
        radio.set_sfd(DEFAULT_SFD);
        radio.set_txpower(DEFAULT_TXPOWER);

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

    /// Changes the Start of Frame Delimiter
    pub fn set_sfd(&mut self, sfd: u8) {
        // FIXME don't completely turn off the radio; RXIDLE or TXIDLE are probably OK
        self.disable();

        self.radio.sfd.write(|w| unsafe { w.sfd().bits(sfd) });
    }

    /// Changes the TX power
    pub fn set_txpower(&mut self, power: TxPower) {
        // FIXME don't completely turn off the radio; RXIDLE or TXIDLE are probably OK
        self.disable();

        self.radio
            .txpower
            .write(|w| w.txpower().variant(power._into()));
    }

    /// Recevies one radio packet and copies its contents into the given `packet` buffer
    ///
    /// This methods returns the `Ok` variant if the CRC included the packet was successfully
    /// validated by the hardware; otherwise it returns the `Err` variant. In either case, `packet`
    /// will be updated with the received packet's data
    pub fn recv(&mut self, packet: &mut Packet) -> Result<(), ()> {
        // NOTE we do NOT check the address of `packet`; see comment in `Packet::new` for details

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

        if self.radio.crcstatus.read().crcstatus().bit_is_set() {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Tries to send the given `packet`
    ///
    /// This method performs Clear Channel Assessment (CCA) first and sends the `packet` only if the
    /// channel is observed to be *clear* (no transmission is currently ongoing), otherwise no
    /// packet is transmitted and the `Err` variant is returned
    pub fn try_send(&mut self, packet: &Packet) -> Result<(), ()> {
        // NOTE we do NOT check the address of `packet`; see comment in `Packet::new` for details
        // go to the RXIDLE state
        self.enable_rx();

        // start CCA
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
                break;
            }

            if self
                .radio
                .events_ccabusy
                .read()
                .events_ccabusy()
                .bit_is_set()
            {
                // channel is busy
                self.radio.events_ccabusy.reset();
                return Err(());
            }
        }

        // NOTE(unsafe) DMA transfer has not yet started
        unsafe {
            self.radio
                .packetptr
                .write(|w| w.packetptr().bits(packet.buffer.as_ptr() as u32));
        }

        // the DMA transfer will start at some point after the following write operation so we place
        // the barrier here
        dma_start_fence();
        // enter TX mode
        self.radio.tasks_txen.write(|w| w.tasks_txen().set_bit());

        // due to a shortcut the transmission will start automatically so we just have to wait
        // until the PHYEND event is raised
        self.wait_for_event(Event::PhyEnd);
        dma_end_fence();

        Ok(())
    }

    /// Sends the given `packet`
    ///
    /// This is utility method that *consecutively* calls the `try_send` method until it succeeds.
    /// Note that this approach is *not* IEEE spec compliant -- there must be delay between failed
    /// CCA attempts to be spec compliant
    pub fn send(&mut self, packet: &Packet) {
        // NOTE we do NOT check the address of `packet`; see comment in `Packet::new` for details

        self.radio.shorts.modify(|_, w| w.ccaidle_txen().set_bit());

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

        'cca: loop {
            // start CCA
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
                    self.radio.events_ccabusy.reset();
                    continue 'cca;
                }
            }
        }

        // due to a shortcut the transmission will start automatically so we just have to wait
        // until the PHYEND event is raised
        self.wait_for_event(Event::PhyEnd);
        dma_end_fence();

        self.radio
            .shorts
            .modify(|_, w| w.ccaidle_txen().clear_bit());
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
            Event::PhyEnd => {
                while self
                    .radio
                    .events_phyend
                    .read()
                    .events_phyend()
                    .bit_is_clear()
                {
                    continue;
                }
                self.radio.events_phyend.reset();
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
    PhyEnd,
}

/// An IEEE 802.15.4 packet
///
/// This `Packet` is closest to the PPDU (PHY Protocol Data Unit) defined in the IEEE spec. The API
/// lets users modify the payload of the PPDU via the `deref` and `copy_from_slice` methods. End
/// users should write a MPDU (MAC protocol data unit), starting with a MAC header (MHDR), in the
/// PPDU payload to be IEEE compliant.
///
/// Note that the MAC level CRC, AKA the FCS (Frame Control Sequence), is fully computed in hardware
/// so it doesn't need to be included in the packet's payload
///
/// See figure 119 in the Product Specification of the nRF52840 for more details
pub struct Packet {
    buffer: [u8; Self::SIZE],
}

// See figure 124 in nRF52840-PS
impl Packet {
    const PHY_HDR: usize = 0;
    const DATA: RangeFrom<usize> = 1..;
    const CRC: u8 = 2; // size of the CRC, which is *never* copied to / from RAM
    const SIZE: usize = 1 /* PHY_HDR */ + Self::MAX_LEN as usize + 1 /* LQI */;

    /// The maximum length of the packet's payload
    pub const MAX_LEN: u8 = 125;

    /// Returns an empty packet (length = 0)
    // XXX I believe that be making this not `const` it is not possible to place a `Packet` in
    // `.rodata` (modulo `#[link_section]` shenanigans) thus it is not necessary to check the
    // address of packet in `Radio.{send,recv}` (EasyDMA can only operate on RAM addresses)
    pub fn new() -> Self {
        let mut packet = Self {
            buffer: [0; Self::SIZE],
        };
        packet.set_len(0);
        packet
    }

    /// Fills the packet with given `src` data
    ///
    /// NOTE `src` data will be truncated to `MAX_PACKET_SIZE` bytes
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        let len = cmp::min(src.len(), Self::MAX_LEN as usize) as u8;
        self.buffer[Self::DATA][..len as usize].copy_from_slice(src);
        self.set_len(len);
    }

    /// Returns the size of this packet's payload
    pub fn len(&self) -> u8 {
        self.buffer[Self::PHY_HDR] - Self::CRC
    }

    /// Changes the size of the packet's payload
    ///
    /// NOTE `len` will be truncated to `MAX_LEN` bytes
    pub fn set_len(&mut self, len: u8) {
        let len = cmp::min(len, Self::MAX_LEN);
        self.buffer[Self::PHY_HDR] = len + Self::CRC;
    }

    /// Returns the LQI (Link Quality Indicator) of the received packet
    ///
    /// Note that the LQI is stored in the `Packet`'s internal buffer by the hardware so the value
    /// returned by this method is only valid after a `Radio.recv` operation. Operations that
    /// modify the `Packet`, like `copy_from_slice` or `set_len`+`deref_mut`, will overwrite the
    /// stored LQI value.
    ///
    /// Also note that the hardware will *not* compute a LQI for packets smaller than 3 bytes so
    /// this method will return a junk value for those packets.
    pub fn lqi(&self) -> u8 {
        self.buffer[1 /* PHY_HDR */ + self.len() as usize /* data */]
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
