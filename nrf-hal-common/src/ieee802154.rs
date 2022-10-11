//! IEEE 802.15.4 radio

use core::{
    marker::PhantomData,
    ops::{self, RangeFrom},
    sync::atomic::{self, Ordering},
};

use embedded_hal::timer::CountDown as _;

use crate::{
    clocks::{Clocks, ExternalOscillator},
    pac::{
        radio::{state::STATE_A, txpower::TXPOWER_A},
        RADIO,
    },
    timer::{self, Timer},
};

/// IEEE 802.15.4 radio
pub struct Radio<'c> {
    radio: RADIO,
    // RADIO needs to be (re-)enabled to pick up new settings
    needs_enable: bool,
    // used to freeze `Clocks`
    _clocks: PhantomData<&'c ()>,
}

/// Default Clear Channel Assessment method = Carrier sense
pub const DEFAULT_CCA: Cca = Cca::CarrierSense;

/// Default radio channel = Channel 11 (`2_405` MHz)
pub const DEFAULT_CHANNEL: Channel = Channel::_11;

/// Default TX power = 0 dBm
pub const DEFAULT_TXPOWER: TxPower = TxPower::_0dBm;

/// Default Start of Frame Delimiter = `0xA7` (IEEE compliant)
pub const DEFAULT_SFD: u8 = 0xA7;

// TODO expose the other variants in `pac::CCAMODE_A`
/// Clear Channel Assessment method
pub enum Cca {
    /// Carrier sense
    CarrierSense,
    /// Energy Detection / Energy Above Threshold
    EnergyDetection {
        /// Energy measurements above this value mean that the channel is assumed to be busy.
        /// Note the the measurement range is 0..0xFF - where 0 means that the received power was
        /// less than 10 dB above the selected receiver sensitivity. This value is not given in dBm,
        /// but can be converted. See the nrf52840 Product Specification Section 6.20.12.4
        /// for details.
        ed_threshold: u8,
    },
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
            TxPower::Neg40dBm => TXPOWER_A::NEG40D_BM,
            TxPower::Neg20dBm => TXPOWER_A::NEG20D_BM,
            TxPower::Neg16dBm => TXPOWER_A::NEG16D_BM,
            TxPower::Neg12dBm => TXPOWER_A::NEG12D_BM,
            TxPower::Neg8dBm => TXPOWER_A::NEG8D_BM,
            TxPower::Neg4dBm => TXPOWER_A::NEG4D_BM,
            TxPower::_0dBm => TXPOWER_A::_0D_BM,
            TxPower::Pos2dBm => TXPOWER_A::POS2D_BM,
            TxPower::Pos3dBm => TXPOWER_A::POS3D_BM,
            TxPower::Pos4dBm => TXPOWER_A::POS4D_BM,
            TxPower::Pos5dBm => TXPOWER_A::POS5D_BM,
            TxPower::Pos6dBm => TXPOWER_A::POS6D_BM,
            TxPower::Pos7dBm => TXPOWER_A::POS7D_BM,
            TxPower::Pos8dBm => TXPOWER_A::POS8D_BM,
        }
    }
}

impl<'c> Radio<'c> {
    /// Initializes the radio for IEEE 802.15.4 operation
    pub fn init<L, LSTAT>(radio: RADIO, _clocks: &'c Clocks<ExternalOscillator, L, LSTAT>) -> Self {
        let mut radio = Self {
            needs_enable: false,
            radio,
            _clocks: PhantomData,
        };

        // shortcuts will be kept off by default and only be temporarily enabled within blocking
        // functions
        radio.radio.shorts.reset();

        // go to a known state
        radio.disable();

        // clear any event of interest to us
        radio.radio.events_disabled.reset();
        radio.radio.events_end.reset();
        radio.radio.events_phyend.reset();

        radio.radio.mode.write(|w| w.mode().ieee802154_250kbit());

        // NOTE(unsafe) radio is currently disabled
        unsafe {
            radio.radio.pcnf0.write(|w| {
                w.s1incl()
                    .clear_bit() // S1 not included in RAM
                    .plen()
                    ._32bit_zero() // 32-bit zero preamble
                    .crcinc()
                    .include() // the LENGTH field (the value) also accounts for the CRC (2 bytes)
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
                    .bits(Packet::MAX_PSDU_LEN) // payload length
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
        }

        // set default settings
        radio.set_channel(DEFAULT_CHANNEL);
        radio.set_cca(DEFAULT_CCA);
        radio.set_sfd(DEFAULT_SFD);
        radio.set_txpower(DEFAULT_TXPOWER);

        radio
    }

    /// Changes the radio channel
    pub fn set_channel(&mut self, channel: Channel) {
        self.needs_enable = true;
        unsafe {
            self.radio
                .frequency
                .write(|w| w.map().clear_bit().frequency().bits(channel as u8))
        }
    }

    /// Changes the Clear Channel Assessment method
    pub fn set_cca(&mut self, cca: Cca) {
        self.needs_enable = true;
        match cca {
            Cca::CarrierSense => self.radio.ccactrl.write(|w| w.ccamode().carrier_mode()),
            Cca::EnergyDetection { ed_threshold } => {
                // "[ED] is enabled by first configuring the field CCAMODE=EdMode in CCACTRL
                // and writing the CCAEDTHRES field to a chosen value."
                self.radio
                    .ccactrl
                    .write(|w| unsafe { w.ccamode().ed_mode().ccaedthres().bits(ed_threshold) });
            }
        }
    }

    /// Changes the Start of Frame Delimiter
    pub fn set_sfd(&mut self, sfd: u8) {
        // self.needs_enable = true; // this appears to not be needed
        self.radio.sfd.write(|w| unsafe { w.sfd().bits(sfd) });
    }

    /// Changes the TX power
    pub fn set_txpower(&mut self, power: TxPower) {
        self.needs_enable = true;
        self.radio
            .txpower
            .write(|w| w.txpower().variant(power._into()));
    }

    /// Sample the received signal power (i.e. the presence of possibly interfering signals)
    /// within the bandwidth of the currently used channel for `sample_cycles` iterations.
    /// Note that one iteration has a sample time of 128μs, and that each iteration produces the
    /// average RSSI value measured during this sample time.
    ///
    /// Returns the *maximum* measurement recorded during sampling as reported by the hardware (not in dBm!).
    /// The result can be used to find a suitable ED threshold for Energy Detection-based CCA mechanisms.
    ///
    /// For details, see Section 6.20.12.3 Energy detection (ED) of the PS.
    /// RSSI samples are averaged over a measurement time of 8 symbol periods (128 μs).
    pub fn energy_detection_scan(&mut self, sample_cycles: u32) -> u8 {
        unsafe {
            // Increase the time spent listening
            self.radio.edcnt.write(|w| w.edcnt().bits(sample_cycles));
        }

        // ensure that the shortcut between READY event and START task is disabled before putting
        // the radio into recv mode
        self.radio.shorts.reset();
        self.put_in_rx_mode();

        // clear related events
        self.radio.events_edend.reset();

        // start energy detection sampling
        self.radio
            .tasks_edstart
            .write(|w| w.tasks_edstart().set_bit());

        loop {
            if self.radio.events_edend.read().events_edend().bit_is_set() {
                // sampling period is over; collect value
                self.radio.events_edend.reset();

                // note that since we have increased EDCNT, the EDSAMPLE register contains the
                // maximum recorded value, not the average
                let read_lvl = self.radio.edsample.read().edlvl().bits();
                return read_lvl;
            }
        }
    }

    /// Receives one radio packet and copies its contents into the given `packet` buffer
    ///
    /// This methods returns the `Ok` variant if the CRC included the packet was successfully
    /// validated by the hardware; otherwise it returns the `Err` variant. In either case, `packet`
    /// will be updated with the received packet's data
    pub fn recv(&mut self, packet: &mut Packet) -> Result<u16, u16> {
        // Start the read
        // NOTE(unsafe) We block until reception completes or errors
        unsafe {
            self.start_recv(packet);
        }

        // wait until we have received something
        self.wait_for_event(Event::End);
        dma_end_fence();

        let crc = self.radio.rxcrc.read().rxcrc().bits() as u16;
        if self.radio.crcstatus.read().crcstatus().bit_is_set() {
            Ok(crc)
        } else {
            Err(crc)
        }
    }

    /// Listens for a packet for no longer than the specified amount of microseconds
    /// and copies its contents into the given `packet` buffer
    ///
    /// If no packet is received within the specified time then the `Timeout` error is returned
    ///
    /// If a packet is received within the time span then the packet CRC is checked. If the CRC is
    /// incorrect then the `Crc` error is returned; otherwise the `Ok` variant is returned.
    /// Note that `packet` will contain the packet in any case, even if the CRC check failed.
    ///
    /// Note that the time it takes to switch the radio to RX mode is included in the timeout count.
    /// This transition may take up to a hundred of microseconds; see the section 6.20.15.8 in the
    /// Product Specification for more details about timing
    pub fn recv_timeout<I>(
        &mut self,
        packet: &mut Packet,
        timer: &mut Timer<I>,
        microseconds: u32,
    ) -> Result<u16, Error>
    where
        I: timer::Instance,
    {
        // Start the timeout timer
        timer.start(microseconds);

        // Start the read
        // NOTE(unsafe) We block until reception completes or errors
        unsafe {
            self.start_recv(packet);
        }

        // Wait for transmission to end
        let mut recv_completed = false;

        loop {
            if self.radio.events_end.read().bits() != 0 {
                // transfer complete
                dma_end_fence();
                recv_completed = true;
                break;
            }

            if timer.wait().is_ok() {
                // timeout
                break;
            }
        }

        if !recv_completed {
            // Cancel the reception if it did not complete until now
            self.cancel_recv();
            Err(Error::Timeout)
        } else {
            let crc = self.radio.rxcrc.read().rxcrc().bits() as u16;
            if self.radio.crcstatus.read().crcstatus().bit_is_set() {
                Ok(crc)
            } else {
                Err(Error::Crc(crc))
            }
        }
    }

    unsafe fn start_recv(&mut self, packet: &mut Packet) {
        // NOTE we do NOT check the address of `packet` because the mutable reference ensures it's
        // allocated in RAM

        // clear related events
        self.radio.events_phyend.reset();
        self.radio.events_end.reset();

        self.put_in_rx_mode();

        // NOTE(unsafe) DMA transfer has not yet started
        // set up RX buffer
        self.radio
            .packetptr
            .write(|w| w.packetptr().bits(packet.buffer.as_mut_ptr() as u32));

        // start transfer
        dma_start_fence();
        self.radio.tasks_start.write(|w| w.tasks_start().set_bit());
    }

    fn cancel_recv(&mut self) {
        self.radio.tasks_stop.write(|w| w.tasks_stop().set_bit());
        self.wait_for_state_a(STATE_A::RX_IDLE);
        // DMA transfer may have been in progress so synchronize with its memory operations
        dma_end_fence();
    }

    /// Tries to send the given `packet`
    ///
    /// This method performs Clear Channel Assessment (CCA) first and sends the `packet` only if the
    /// channel is observed to be *clear* (no transmission is currently ongoing), otherwise no
    /// packet is transmitted and the `Err` variant is returned
    ///
    /// NOTE this method will *not* modify the `packet` argument. The mutable reference is used to
    /// ensure the `packet` buffer is allocated in RAM, which is required by the RADIO peripheral
    // NOTE we do NOT check the address of `packet` because the mutable reference ensures it's
    // allocated in RAM
    pub fn try_send(&mut self, packet: &mut Packet) -> Result<(), ()> {
        // enable radio to perform cca
        self.put_in_rx_mode();

        // clear related events
        self.radio.events_phyend.reset();
        self.radio.events_end.reset();

        // NOTE(unsafe) DMA transfer has not yet started
        unsafe {
            self.radio
                .packetptr
                .write(|w| w.packetptr().bits(packet.buffer.as_ptr() as u32));
        }

        // configure radio to immediately start transmission if the channel is idle
        self.radio.shorts.modify(|_, w| {
            w.ccaidle_txen()
                .set_bit()
                .txready_start()
                .set_bit()
                .end_disable()
                .set_bit()
        });

        // the DMA transfer will start at some point after the following write operation so
        // we place the compiler fence here
        dma_start_fence();
        // start CCA. In case the channel is clear, the data at packetptr will be sent automatically
        self.radio
            .tasks_ccastart
            .write(|w| w.tasks_ccastart().set_bit());

        loop {
            if self.radio.events_phyend.read().events_phyend().bit_is_set() {
                // transmission completed
                dma_end_fence();
                self.radio.events_phyend.reset();
                self.radio.shorts.reset();
                return Ok(());
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
                self.radio.shorts.reset();
                return Err(());
            }
        }
    }

    /// Sends the given `packet`
    ///
    /// This is utility method that *consecutively* calls the `try_send` method until it succeeds.
    /// Note that this approach is *not* IEEE spec compliant -- there must be delay between failed
    /// CCA attempts to be spec compliant
    ///
    /// NOTE this method will *not* modify the `packet` argument. The mutable reference is used to
    /// ensure the `packet` buffer is allocated in RAM, which is required by the RADIO peripheral
    // NOTE we do NOT check the address of `packet` because the mutable reference ensures it's
    // allocated in RAM
    pub fn send(&mut self, packet: &mut Packet) {
        // enable radio to perform cca
        self.put_in_rx_mode();

        // clear related events
        self.radio.events_phyend.reset();
        self.radio.events_end.reset();

        // immediately start transmission if the channel is idle
        self.radio.shorts.modify(|_, w| {
            w.ccaidle_txen()
                .set_bit()
                .txready_start()
                .set_bit()
                .end_disable()
                .set_bit()
        });

        // the DMA transfer will start at some point after the following write operation so
        // we place the compiler fence here
        dma_start_fence();
        // NOTE(unsafe) DMA transfer has not yet started
        unsafe {
            self.radio
                .packetptr
                .write(|w| w.packetptr().bits(packet.buffer.as_ptr() as u32));
        }

        'cca: loop {
            // start CCA (+ sending if channel is clear)
            self.radio
                .tasks_ccastart
                .write(|w| w.tasks_ccastart().set_bit());

            loop {
                if self.radio.events_phyend.read().events_phyend().bit_is_set() {
                    dma_end_fence();
                    // transmission is complete
                    self.radio.events_phyend.reset();
                    break 'cca;
                }

                if self
                    .radio
                    .events_ccabusy
                    .read()
                    .events_ccabusy()
                    .bit_is_set()
                {
                    // channel is busy; try another CCA
                    self.radio.events_ccabusy.reset();
                    continue 'cca;
                }
            }
        }

        self.radio.shorts.reset();
    }

    /// Sends the specified `packet` without first performing CCA
    ///
    /// Acknowledgment packets must be sent using this method
    ///
    /// NOTE this method will *not* modify the `packet` argument. The mutable reference is used to
    /// ensure the `packet` buffer is allocated in RAM, which is required by the RADIO peripheral
    // NOTE we do NOT check the address of `packet` because the mutable reference ensures it's
    // allocated in RAM
    pub fn send_no_cca(&mut self, packet: &mut Packet) {
        self.put_in_tx_mode();

        // clear related events
        self.radio.events_phyend.reset();
        self.radio.events_end.reset();

        // NOTE(unsafe) DMA transfer has not yet started
        unsafe {
            self.radio
                .packetptr
                .write(|w| w.packetptr().bits(packet.buffer.as_ptr() as u32));
        }

        // configure radio to disable transmitter once packet is sent
        self.radio.shorts.modify(|_, w| w.end_disable().set_bit());

        // start DMA transfer
        dma_start_fence();
        self.radio.tasks_start.write(|w| w.tasks_start().set_bit());

        self.wait_for_event(Event::PhyEnd);
        self.radio.shorts.reset();
    }

    /// Moves the radio from any state to the DISABLED state
    fn disable(&mut self) {
        // See figure 110 in nRF52840-PS
        loop {
            match self.radio.state.read().state().variant().unwrap() {
                STATE_A::DISABLED => return,

                STATE_A::RX_RU | STATE_A::RX_IDLE | STATE_A::TX_RU | STATE_A::TX_IDLE => {
                    self.radio
                        .tasks_disable
                        .write(|w| w.tasks_disable().set_bit());

                    self.wait_for_state_a(STATE_A::DISABLED);
                    return;
                }

                // ramping down
                STATE_A::RX_DISABLE | STATE_A::TX_DISABLE => {
                    self.wait_for_state_a(STATE_A::DISABLED);
                    return;
                }

                // cancel ongoing transfer or ongoing CCA
                STATE_A::RX => {
                    self.radio
                        .tasks_ccastop
                        .write(|w| w.tasks_ccastop().set_bit());
                    self.radio.tasks_stop.write(|w| w.tasks_stop().set_bit());
                    self.wait_for_state_a(STATE_A::RX_IDLE);
                }
                STATE_A::TX => {
                    self.radio.tasks_stop.write(|w| w.tasks_stop().set_bit());
                    self.wait_for_state_a(STATE_A::TX_IDLE);
                }
            }
        }
    }

    /// Moves the radio to the RXIDLE state
    fn put_in_rx_mode(&mut self) {
        let state = self.state();

        let (disable, enable) = match state {
            State::Disabled => (false, true),
            State::RxIdle => (false, self.needs_enable),
            // NOTE to avoid errata 204 (see rev1 v1.4) we do TXIDLE -> DISABLED -> RXIDLE
            State::TxIdle => (true, true),
        };

        if disable {
            self.radio
                .tasks_disable
                .write(|w| w.tasks_disable().set_bit());
            self.wait_for_state_a(STATE_A::DISABLED);
        }

        if enable {
            self.needs_enable = false;
            self.radio.tasks_rxen.write(|w| w.tasks_rxen().set_bit());
            self.wait_for_state_a(STATE_A::RX_IDLE);
        }
    }

    /// Moves the radio to the TXIDLE state
    fn put_in_tx_mode(&mut self) {
        let state = self.state();

        if state != State::TxIdle || self.needs_enable {
            self.needs_enable = false;
            self.radio.tasks_txen.write(|w| w.tasks_txen().set_bit());
            self.wait_for_state_a(STATE_A::TX_IDLE);
        }
    }

    fn state(&self) -> State {
        match self.radio.state.read().state().variant().unwrap() {
            // final states
            STATE_A::DISABLED => State::Disabled,
            STATE_A::TX_IDLE => State::TxIdle,
            STATE_A::RX_IDLE => State::RxIdle,

            // transitory states
            STATE_A::TX_DISABLE => {
                self.wait_for_state_a(STATE_A::DISABLED);
                State::Disabled
            }

            _ => unreachable!(),
        }
    }

    fn wait_for_event(&self, event: Event) {
        match event {
            Event::End => {
                while self.radio.events_end.read().events_end().bit_is_clear() {}
                self.radio.events_end.reset();
            }
            Event::PhyEnd => {
                while self
                    .radio
                    .events_phyend
                    .read()
                    .events_phyend()
                    .bit_is_clear()
                {}
                self.radio.events_phyend.reset();
            }
        }
    }

    /// Waits until the radio state matches the given `state`
    fn wait_for_state_a(&self, state: STATE_A) {
        while self.radio.state.read().state().variant().unwrap() != state {}
    }
}

/// Error
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Error {
    /// Incorrect CRC
    Crc(u16),
    /// Timeout
    Timeout,
}

/// Driver state
///
/// After, or at the start of, any method call the RADIO will be in one of these states
// This is a subset of the STATE_A enum
#[derive(Copy, Clone, PartialEq)]
enum State {
    Disabled,
    RxIdle,
    TxIdle,
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
/// This `Packet` is a PHY layer packet. It's made up of the physical header (PHR) and the PSDU
/// (PHY service data unit). The PSDU of this `Packet` will always include the MAC level CRC, AKA
/// the FCS (Frame Control Sequence) -- the CRC is fully computed in hardware and automatically
/// appended on transmission and verified on reception.
///
/// The API lets users modify the usable part (not the CRC) of the PSDU via the `deref` and
/// `copy_from_slice` methods. These methods will automatically update the PHR.
///
/// See figure 119 in the Product Specification of the nRF52840 for more details
pub struct Packet {
    buffer: [u8; Self::SIZE],
}

// See figure 124 in nRF52840-PS
impl Packet {
    // for indexing purposes
    const PHY_HDR: usize = 0;
    const DATA: RangeFrom<usize> = 1..;

    /// Maximum amount of usable payload (CRC excluded) a single packet can contain, in bytes
    pub const CAPACITY: u8 = 125;
    const CRC: u8 = 2; // size of the CRC, which is *never* copied to / from RAM
    const MAX_PSDU_LEN: u8 = Self::CAPACITY + Self::CRC;
    const SIZE: usize = 1 /* PHR */ + Self::MAX_PSDU_LEN as usize;

    /// Returns an empty packet (length = 0)
    pub fn new() -> Self {
        let mut packet = Self {
            buffer: [0; Self::SIZE],
        };
        packet.set_len(0);
        packet
    }

    /// Fills the packet payload with given `src` data
    ///
    /// # Panics
    ///
    /// This function panics if `src` is larger than `Self::CAPACITY`
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        assert!(src.len() <= Self::CAPACITY as usize);
        let len = src.len() as u8;
        self.buffer[Self::DATA][..len as usize].copy_from_slice(&src[..len.into()]);
        self.set_len(len);
    }

    /// Returns the size of this packet's payload
    pub fn len(&self) -> u8 {
        self.buffer[Self::PHY_HDR] - Self::CRC
    }

    /// Changes the size of the packet's payload
    ///
    /// # Panics
    ///
    /// This function panics if `len` is larger than `Self::CAPACITY`
    pub fn set_len(&mut self, len: u8) {
        assert!(len <= Self::CAPACITY);
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
    /// this method will return an invalid value for those packets.
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
