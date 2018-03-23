use gpio;
use hal::spi::{Mode, Phase, Polarity, FullDuplex};
use nrf52::{SPIM0, SPIM1, SPIM2};
use time::Hertz;
use nb;

// pub const DEFAULT_SPI_MODE: Mode = Mode {
//     polarity: Polarity::IdleLow,
//     phase: Phase::CaptureOnFirstTransition,
// };


// TODO, theoretically any set of three pins can be a SPI master. How do I handle this generically?

// For now, use the following hardcoded pins to SPI1:
// SPI1_MISO:  P0.18
// SPI1_MOSI:  P0.20
// SPI1_CLK:   P0.16
// DW_IRQ:     P0.19
// DW_CS:      P0.17
// DW_RST:     P0.24

use gpio::p0::P0_Pin;
use gpio::{Input, Output, Floating, PushPull};

pub enum Error {
    Bad,
}

impl FullDuplex<u8> for Spi1 {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        // TODO, verify events_started != 0
        Ok(unsafe {
            ::core::ptr::read_volatile(&self.rx_buf as *const _ as *const u8)
        })
    }
    fn send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        let ptr = &self.tx_buf as *const _ as u32;
        self._spi.txd.ptr.write(|w| unsafe{
            w.bits(ptr)
        });
        assert_eq!(ptr, self._spi.txd.ptr.read().bits());

        let ptr = &self.rx_buf as *const _ as u32;
        self._spi.rxd.ptr.write(|w| unsafe{
            w.bits(ptr)
        });
        assert_eq!(ptr, self._spi.rxd.ptr.read().bits());

        self._spi.txd.maxcnt.write(|w| unsafe {w.bits(1) });
        self._spi.rxd.maxcnt.write(|w| unsafe {w.bits(0) });

        self._spi.events_started.write(|w| unsafe { w.bits(0) });
        self._spi.events_end.write(|w| unsafe {w.bits(0)});

        self.tx_buf = word;

        self._spi.tasks_start.write(|w| unsafe {w.bits(1) });

        while self._spi.events_end.read().bits() == 0 {

        }

        Ok(())
    }
}

pub struct Spi1 { // TODO generic across Spi[0...2]
    miso_pin: P0_Pin<Input<Floating>>,
    mosi_pin: P0_Pin<Output<PushPull>>,
    clk_pin: P0_Pin<Output<PushPull>>,
    _spi: &'static ::nrf52::spim0::RegisterBlock, // TODO
    rx_buf: u8,
    tx_buf: u8,
}

impl Spi1 {
    pub fn new(
        mode: Mode,
        miso_pin: P0_Pin<Input<Floating>>,
        mosi_pin: P0_Pin<Output<PushPull>>,
        clk_pin: P0_Pin<Output<PushPull>>,
        freq: Hertz,
    ) -> Self {

        // TODO - obtain something that prevents shared overlapping peripherals?
        // See Product Spec Section 29.1 "Shared Resources"

        let mut spi1 = unsafe {
            &(*SPIM1::ptr())
        };

        // spi1.shorts; // ?
        // spi1.intenset; // ? enable interrupts?
        // spi1.intenclr; // ? disable interrupts?

        spi1.enable.write(|w| w.enable().disabled());

        // configure pins
        // NOTE: Unsafe because svd2rust can't figure out the pins bits
        spi1.psel.sck.write(|w| {
            let pin = clk_pin.pin;
            unsafe { w.pin().bits(pin); }
            w.connect().connected()
        });
        spi1.psel.mosi.write(|w| {
            let pin = mosi_pin.pin;
            unsafe { w.pin().bits(pin); }
            w.connect().connected()
        });
        spi1.psel.miso.write(|w| {
            let pin = miso_pin.pin;
            unsafe { w.pin().bits(pin); }
            w.connect().connected()
        });

        // Configure SPI
        spi1.frequency.write(|w| {
            match freq.0 {
                125_000   => w.frequency().k125(),
                250_000   => w.frequency().k250(),
                500_000   => w.frequency().k500(),
                1_000_000 => w.frequency().m1(),
                2_000_000 => w.frequency().m2(),
                4_000_000 => w.frequency().m4(),
                8_000_000 => w.frequency().m8(),
                _ => unreachable!(),
            }
        });

        spi1.config.write(|w| {
            match mode.polarity {
                Polarity::IdleLow => w.cpol().active_high(),
                Polarity::IdleHigh => w.cpol().active_low(),
            };

            match mode.phase {
                Phase::CaptureOnFirstTransition => w.cpha().leading(),
                Phase::CaptureOnSecondTransition => w.cpha().trailing(),
            };

            // TODO set MSB/LSB in config?
            w.order().msb_first()
        });

        // TODO - DMA? Disable for now
        spi1.rxd.list.reset();
        spi1.rxd.maxcnt.reset();
        spi1.txd.list.reset();
        spi1.txd.maxcnt.reset();

        spi1.enable.write(|w| w.enable().enabled());

        // TODO - datasheet says high speed might need higher drive strength?
        Spi1 {
            miso_pin,
            mosi_pin,
            clk_pin,
            _spi: spi1,
            rx_buf: 0,
            tx_buf: 0,
        }
    }
}

// ================================> STM32F103xx below

// use core::ptr;

// use hal::spi::{Mode, Phase, Polarity};
// use hal;
// use nb;
// use stm32f103xx::{SPI1, SPI2};

// use afio::MAPR;
// use gpio::gpioa::{PA5, PA6, PA7};
// use gpio::gpiob::{PB13, PB14, PB15, PB3, PB4, PB5};
// use gpio::{Alternate, Floating, Input, PushPull};
// use rcc::{APB1, APB2, Clocks};


// /// SPI error
// #[derive(Debug)]
// pub enum Error {
//     /// Overrun occurred
//     Overrun,
//     /// Mode fault occurred
//     ModeFault,
//     /// CRC error
//     Crc,
//     #[doc(hidden)] _Extensible,
// }

// pub trait Pins<SPI> {
//     const REMAP: bool;
// }

// impl Pins<SPI1>
//     for (
//         PA5<Alternate<PushPull>>,
//         PA6<Input<Floating>>,
//         PA7<Alternate<PushPull>>,
//     ) {
//     const REMAP: bool = false;
// }

// impl Pins<SPI1>
//     for (
//         PB3<Alternate<PushPull>>,
//         PB4<Input<Floating>>,
//         PB5<Alternate<PushPull>>,
//     ) {
//     const REMAP: bool = true;
// }

// impl Pins<SPI2>
//     for (
//         PB13<Alternate<PushPull>>,
//         PB14<Input<Floating>>,
//         PB15<Alternate<PushPull>>,
//     ) {
//     const REMAP: bool = false;
// }

// pub struct Spi<SPI, PINS> {
//     spi: SPI,
//     pins: PINS,
// }

// impl<PINS> Spi<SPI1, PINS> {
//     pub fn spi1<F>(
//         spi: SPI1,
//         pins: PINS,
//         mapr: &mut MAPR,
//         mode: Mode,
//         freq: F,
//         clocks: Clocks,
//         apb: &mut APB2,
//     ) -> Self
//     where
//         F: Into<Hertz>,
//         PINS: Pins<SPI1>,
//     {
//         mapr.mapr().modify(|_, w| w.spi1_remap().bit(PINS::REMAP));
//         Spi::_spi1(spi, pins, mode, freq.into(), clocks, apb)
//     }
// }

// impl<PINS> Spi<SPI2, PINS> {
//     pub fn spi2<F>(
//         spi: SPI2,
//         pins: PINS,
//         mode: Mode,
//         freq: F,
//         clocks: Clocks,
//         apb: &mut APB1,
//     ) -> Self
//     where
//         F: Into<Hertz>,
//         PINS: Pins<SPI2>,
//     {
//         Spi::_spi2(spi, pins, mode, freq.into(), clocks, apb)
//     }
// }

// macro_rules! hal {
//     ($($SPIX:ident: ($spiX:ident, $spiXen:ident, $spiXrst:ident, $APB:ident),)+) => {
//         $(
//             impl<PINS> Spi<$SPIX, PINS> {
//                 fn $spiX(
//                     spi: $SPIX,
//                     pins: PINS,
//                     mode: Mode,
//                     freq: Hertz,
//                     clocks: Clocks,
//                     apb: &mut $APB,
//                 ) -> Self {
//                     // enable or reset $SPIX
//                     apb.enr().modify(|_, w| w.$spiXen().enabled());
//                     apb.rstr().modify(|_, w| w.$spiXrst().set_bit());
//                     apb.rstr().modify(|_, w| w.$spiXrst().clear_bit());

//                     // disable SS output
//                     spi.cr2.write(|w| w.ssoe().clear_bit());

//                     let br = match clocks.pclk2().0 / freq.0 {
//                         0 => unreachable!(),
//                         1...2 => 0b000,
//                         3...5 => 0b001,
//                         6...11 => 0b010,
//                         12...23 => 0b011,
//                         24...47 => 0b100,
//                         48...95 => 0b101,
//                         96...191 => 0b110,
//                         _ => 0b111,
//                     };

//                     // mstr: master configuration
//                     // lsbfirst: MSB first
//                     // ssm: enable software slave management (NSS pin free for other uses)
//                     // ssi: set nss high = master mode
//                     // dff: 8 bit frames
//                     // bidimode: 2-line unidirectional
//                     // spe: enable the SPI bus
//                     spi.cr1.write(|w| {
//                         w.cpha()
//                             .bit(mode.phase == Phase::CaptureOnSecondTransition)
//                             .cpol()
//                             .bit(mode.polarity == Polarity::IdleHigh)
//                             .mstr()
//                             .set_bit()
//                             .br()
//                             .bits(br)
//                             .lsbfirst()
//                             .clear_bit()
//                             .ssm()
//                             .set_bit()
//                             .ssi()
//                             .set_bit()
//                             .rxonly()
//                             .clear_bit()
//                             .dff()
//                             .clear_bit()
//                             .bidimode()
//                             .clear_bit()
//                             .spe()
//                             .set_bit()
//                     });

//                     Spi { spi, pins }
//                 }

//                 pub fn free(self) -> ($SPIX, PINS) {
//                     (self.spi, self.pins)
//                 }
//             }

//             impl<PINS> hal::spi::FullDuplex<u8> for Spi<$SPIX, PINS> {
//                 type Error = Error;

//                 fn read(&mut self) -> nb::Result<u8, Error> {
//                     let sr = self.spi.sr.read();

//                     Err(if sr.ovr().bit_is_set() {
//                         nb::Error::Other(Error::Overrun)
//                     } else if sr.modf().bit_is_set() {
//                         nb::Error::Other(Error::ModeFault)
//                     } else if sr.crcerr().bit_is_set() {
//                         nb::Error::Other(Error::Crc)
//                     } else if sr.rxne().bit_is_set() {
//                         // NOTE(read_volatile) read only 1 byte (the svd2rust API only allows
//                         // reading a half-word)
//                         return Ok(unsafe {
//                             ptr::read_volatile(&self.spi.dr as *const _ as *const u8)
//                         });
//                     } else {
//                         nb::Error::WouldBlock
//                     })
//                 }

//                 fn send(&mut self, byte: u8) -> nb::Result<(), Error> {
//                     let sr = self.spi.sr.read();

//                     Err(if sr.ovr().bit_is_set() {
//                         nb::Error::Other(Error::Overrun)
//                     } else if sr.modf().bit_is_set() {
//                         nb::Error::Other(Error::ModeFault)
//                     } else if sr.crcerr().bit_is_set() {
//                         nb::Error::Other(Error::Crc)
//                     } else if sr.txe().bit_is_set() {
//                         // NOTE(write_volatile) see note above
//                         unsafe { ptr::write_volatile(&self.spi.dr as *const _ as *mut u8, byte) }
//                         return Ok(());
//                     } else {
//                         nb::Error::WouldBlock
//                     })
//                 }

//             }

//             impl<PINS> ::hal::blocking::spi::transfer::Default<u8> for Spi<$SPIX, PINS> {}

//             impl<PINS> ::hal::blocking::spi::write::Default<u8> for Spi<$SPIX, PINS> {}
//         )+
//     }
// }

// hal! {
//     SPI1: (_spi1, spi1en, spi1rst, APB2),
//     SPI2: (_spi2, spi2en, spi2rst, APB1),
// }
