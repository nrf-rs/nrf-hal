//! Board support crate for the Nordic nRF52840-DK
//!
//! This crate is in early development. UARTE, SPIM and TWI should be functiona,
//! but might miss some features.
#![no_std]

pub use cortex_m;
pub use cortex_m_rt;
pub use embedded_hal;
pub use nrf52840_hal as hal;

/// Exports traits that are usually needed when using this crate
pub mod prelude {
    pub use nrf52840_hal::prelude::*;
}

// TODO: Maybe we want a debug module like in the DWM1001-Dev implementation.
// pub mod debug;

use nrf52840_hal::{
    prelude::*,
    gpio::{
        p0,
        p1,
        Pin,
        Floating,
        Input,
        Output,
        PushPull,
        PullUp,
        Level,
    },
    nrf52840_pac::{
        self as nrf52,
        CorePeripherals,
        Peripherals,
    },
    spim::{
        self,
        Frequency,
        MODE_0,
        Spim
    },
    uarte::{
        self,
        Uarte,
        Parity as UartParity,
        Baudrate as UartBaudrate,
    },
};

/// Provides access to all features of the nRF52840-DK board
#[allow(non_snake_case)]
pub struct Board {
    /// The nRF52's pins which are not otherwise occupied on the nRF52840-DK
    pub pins: Pins,

    /// The nRF52840-DK UART which is wired to the virtual USB CDC port
    pub cdc: Uarte<nrf52::UARTE0>,

    /// The nRF52840-DK SPI which is wired to the SPI flash
    pub flash: Spim<nrf52::SPIM2>,
    pub flash_cs: Pin<Output<PushPull>>,

    /// The LEDs on the nRF52840-DK board
    pub leds: Leds,

    /// The buttons on the nRF52840-DK board
    pub buttons: Buttons,

    pub nfc: NFC,

    /// Core peripheral: Cache and branch predictor maintenance operations
    pub CBP: nrf52::CBP,

    /// Core peripheral: CPUID
    pub CPUID: nrf52::CPUID,

    /// Core peripheral: Debug Control Block
    pub DCB: nrf52::DCB,

    /// Core peripheral: Data Watchpoint and Trace unit
    pub DWT: nrf52::DWT,

    /// Core peripheral: Flash Patch and Breakpoint unit
    pub FPB: nrf52::FPB,

    /// Core peripheral: Floating Point Unit
    pub FPU: nrf52::FPU,

    /// Core peripheral: Instrumentation Trace Macrocell
    pub ITM: nrf52::ITM,

    /// Core peripheral: Memory Protection Unit
    pub MPU: nrf52::MPU,

    /// Core peripheral: Nested Vector Interrupt Controller
    pub NVIC: nrf52::NVIC,

    /// Core peripheral: System Control Block
    pub SCB: nrf52::SCB,

    /// Core peripheral: SysTick Timer
    pub SYST: nrf52::SYST,

    /// Core peripheral: Trace Port Interface Unit
    pub TPIU: nrf52::TPIU,

    /// nRF52 peripheral: FICR
    pub FICR: nrf52::FICR,

    /// nRF52 peripheral: UICR
    pub UICR: nrf52::UICR,

    /// nRF52 peripheral: ACL
    pub ACL: nrf52::ACL,

    /// nRF52 peripheral: POWER
    pub POWER: nrf52::POWER,

    /// nRF52 peripheral: CLOCK
    pub CLOCK: nrf52::CLOCK,

    /// nRF52 peripheral: RADIO
    pub RADIO: nrf52::RADIO,

    /// nRF52 peripheral: UART0
    pub UART0: nrf52::UART0,

    /// nRF52 peripheral: SPIM0
    pub SPIM0: nrf52::SPIM0,

    /// nRF52 peripheral: SPIS0
    pub SPIS0: nrf52::SPIS0,

    /// nRF52 peripheral: TWIM0
    pub TWIM0: nrf52::TWIM0,

    /// nRF52 peripheral: TWIS0
    pub TWIS0: nrf52::TWIS0,

    /// nRF52 peripheral: SPI0
    pub SPI0: nrf52::SPI0,

    /// nRF52 peripheral: TWI0
    pub TWI0: nrf52::TWI0,

    /// nRF52 peripheral: SPIM1
    pub SPIM1: nrf52::SPIM1,

    /// nRF52 peripheral: SPIS1
    pub SPIS1: nrf52::SPIS1,

    /// nRF52 peripheral: TWIS1
    pub TWIS1: nrf52::TWIS1,

    /// nRF52 peripheral: SPI1
    pub SPI1: nrf52::SPI1,

    /// nRF52 peripheral: TWI1
    pub TWI1: nrf52::TWI1,

    /// nRF52 peripheral: NFCT
    pub NFCT: nrf52::NFCT,

    /// nRF52 peripheral: GPIOTE
    pub GPIOTE: nrf52::GPIOTE,

    /// nRF52 peripheral: SAADC
    pub SAADC: nrf52::SAADC,

    /// nRF52 peripheral: TIMER0
    pub TIMER0: nrf52::TIMER0,

    /// nRF52 peripheral: TIMER1
    pub TIMER1: nrf52::TIMER1,

    /// nRF52 peripheral: TIMER2
    pub TIMER2: nrf52::TIMER2,

    /// nRF52 peripheral: RTC0
    pub RTC0: nrf52::RTC0,

    /// nRF52 peripheral: TEMP
    pub TEMP: nrf52::TEMP,

    /// nRF52 peripheral: RNG
    pub RNG: nrf52::RNG,

    /// nRF52 peripheral: ECB
    pub ECB: nrf52::ECB,

    /// nRF52 peripheral: CCM
    pub CCM: nrf52::CCM,

    /// nRF52 peripheral: AAR
    pub AAR: nrf52::AAR,

    /// nRF52 peripheral: WDT
    pub WDT: nrf52::WDT,

    /// nRF52 peripheral: RTC1
    pub RTC1: nrf52::RTC1,

    /// nRF52 peripheral: QDEC
    pub QDEC: nrf52::QDEC,

    /// nRF52 peripheral: COMP
    pub COMP: nrf52::COMP,

    /// nRF52 peripheral: LPCOMP
    pub LPCOMP: nrf52::LPCOMP,

    /// nRF52 peripheral: SWI0
    pub SWI0: nrf52::SWI0,

    /// nRF52 peripheral: EGU0
    pub EGU0: nrf52::EGU0,

    /// nRF52 peripheral: SWI1
    pub SWI1: nrf52::SWI1,

    /// nRF52 peripheral: EGU1
    pub EGU1: nrf52::EGU1,

    /// nRF52 peripheral: SWI2
    pub SWI2: nrf52::SWI2,

    /// nRF52 peripheral: EGU2
    pub EGU2: nrf52::EGU2,

    /// nRF52 peripheral: SWI3
    pub SWI3: nrf52::SWI3,

    /// nRF52 peripheral: EGU3
    pub EGU3: nrf52::EGU3,

    /// nRF52 peripheral: SWI4
    pub SWI4: nrf52::SWI4,

    /// nRF52 peripheral: EGU4
    pub EGU4: nrf52::EGU4,

    /// nRF52 peripheral: SWI5
    pub SWI5: nrf52::SWI5,

    /// nRF52 peripheral: EGU5
    pub EGU5: nrf52::EGU5,

    /// nRF52 peripheral: TIMER3
    pub TIMER3: nrf52::TIMER3,

    /// nRF52 peripheral: TIMER4
    pub TIMER4: nrf52::TIMER4,

    /// nRF52 peripheral: PWM0
    pub PWM0: nrf52::PWM0,

    /// nRF52 peripheral: PDM
    pub PDM: nrf52::PDM,

    /// nRF52 peripheral: NVMC
    pub NVMC: nrf52::NVMC,

    /// nRF52 peripheral: PPI
    pub PPI: nrf52::PPI,

    /// nRF52 peripheral: MWU
    pub MWU: nrf52::MWU,

    /// nRF52 peripheral: PWM1
    pub PWM1: nrf52::PWM1,

    /// nRF52 peripheral: PWM2
    pub PWM2: nrf52::PWM2,

    /// nRF52 peripheral: RTC2
    pub RTC2: nrf52::RTC2,

    /// nRF52 peripheral: I2S
    pub I2S: nrf52::I2S,
}

impl Board {
    /// Take the peripherals safely
    ///
    /// This method will return an instance of `nRF52840DK` the first time it is
    /// called. It will return only `None` on subsequent calls.
    pub fn take() -> Option<Self> {
        Some(Self::new(
            CorePeripherals::take()?,
            Peripherals::take()?,
        ))
    }

    /// Steal the peripherals
    ///
    /// This method produces an instance of `nRF52840DK`, regardless of whether
    /// another instance was create previously.
    ///
    /// # Safety
    ///
    /// This method can be used to create multiple instances of `nRF52840DK`. Those
    /// instances can interfere with each other, causing all kinds of unexpected
    /// behavior and circumventing safety guarantees in many ways.
    ///
    /// Always use `nRF52840DK::take`, unless you really know what you're doing.
    pub unsafe fn steal() -> Self {
        Self::new(
            CorePeripherals::steal(),
            Peripherals::steal(),
        )
    }

    fn new(cp: CorePeripherals, p: Peripherals) -> Self {
        let pins0 = p.P0.split();
        let pins1 = p.P1.split();

        // The nRF52840-DK has an 64MB SPI flash on board which can be interfaced through SPI or Quad SPI.
        // As for now, only the normal SPI mode is available, so we are using this for the interface.
        let flash_spim = p.SPIM2.constrain(spim::Pins {
            sck : pins0.p0_19.into_push_pull_output(Level::Low).degrade(),
            mosi: Some(pins0.p0_20.into_push_pull_output(Level::Low).degrade()),
            miso: Some(pins0.p0_21.into_floating_input().degrade()),
        }, Frequency::K500, MODE_0, 0);

        let flash_cs = pins0.p0_17.into_push_pull_output(Level::High).degrade();

        // The nRF52840-DK features an USB CDC port.
        // It features HWFC but does not have to use it.
        // It can transmit a flexible baudrate of up to 1Mbps.
        let cdc_uart = p.UARTE0.constrain(uarte::Pins {
                txd: pins0.p0_06.into_push_pull_output(Level::High).degrade(),
                rxd: pins0.p0_08.into_floating_input().degrade(),
                cts: Some(pins0.p0_07.into_floating_input().degrade()),
                rts: Some(pins0.p0_05.into_push_pull_output(Level::High).degrade()),
            },
            UartParity::EXCLUDED,
            UartBaudrate::BAUD115200
        );

        Board {
            cdc: cdc_uart,
            flash: flash_spim,
            flash_cs: flash_cs,

            pins: Pins {
                P0_03 : pins0.p0_03,
                P0_04 : pins0.p0_04,
                _RESET : pins0.p0_18,
                P0_22 : pins0.p0_22,
                P0_23 : pins0.p0_23,
                P0_26 : pins0.p0_26,
                P0_27 : pins0.p0_27,
                P0_28 : pins0.p0_28,
                P0_29 : pins0.p0_29,
                P0_30 : pins0.p0_30,
                P0_31 : pins0.p0_31,
                P1_00 : pins1.p1_00,
                P1_01 : pins1.p1_01,
                P1_02 : pins1.p1_02,
                P1_03 : pins1.p1_03,
                P1_04 : pins1.p1_04,
                P1_05 : pins1.p1_05,
                P1_06 : pins1.p1_06,
                P1_07 : pins1.p1_07,
                P1_08 : pins1.p1_08,
                P1_09 : pins1.p1_09,
                P1_10 : pins1.p1_10,
                P1_11 : pins1.p1_11,
                P1_12 : pins1.p1_12,
                P1_13 : pins1.p1_13,
                P1_14 : pins1.p1_14,
                P1_15 : pins1.p1_15,
            },

            leds: Leds {
                led_1: Led::new(pins0.p0_13.degrade()),
                led_2: Led::new(pins0.p0_14.degrade()),
                led_3: Led::new(pins0.p0_15.degrade()),
                led_4: Led::new(pins0.p0_16.degrade()),
            },

            buttons: Buttons {
                button_1: Button::new(pins0.p0_11.degrade()),
                button_2: Button::new(pins0.p0_12.degrade()),
                button_3: Button::new(pins0.p0_24.degrade()),
                button_4: Button::new(pins0.p0_25.degrade()),
            },

            nfc: NFC {
                nfc_1: pins0.p0_09,
                nfc_2: pins0.p0_10,
            },

            // Core peripherals
            CBP  : cp.CBP,
            CPUID: cp.CPUID,
            DCB  : cp.DCB,
            DWT  : cp.DWT,
            FPB  : cp.FPB,
            FPU  : cp.FPU,
            ITM  : cp.ITM,
            MPU  : cp.MPU,
            NVIC : cp.NVIC,
            SCB  : cp.SCB,
            SYST : cp.SYST,
            TPIU : cp.TPIU,

            // nRF52 peripherals
            FICR  : p.FICR,
            UICR  : p.UICR,
            ACL   : p.ACL,
            POWER : p.POWER,
            CLOCK : p.CLOCK,
            RADIO : p.RADIO,

            UART0 : p.UART0,
            SPIM0 : p.SPIM0,
            SPIS0 : p.SPIS0,
            TWIM0 : p.TWIM0,
            TWIS0 : p.TWIS0,
            SPI0  : p.SPI0,
            TWI0  : p.TWI0,
            SPIM1 : p.SPIM1,
            SPIS1 : p.SPIS1,
            TWIS1 : p.TWIS1,
            SPI1  : p.SPI1,
            TWI1  : p.TWI1,
            NFCT  : p.NFCT,
            GPIOTE: p.GPIOTE,
            SAADC : p.SAADC,
            TIMER0: p.TIMER0,
            TIMER1: p.TIMER1,
            TIMER2: p.TIMER2,
            RTC0  : p.RTC0,
            TEMP  : p.TEMP,
            RNG   : p.RNG,
            ECB   : p.ECB,
            CCM   : p.CCM,
            AAR   : p.AAR,
            WDT   : p.WDT,
            RTC1  : p.RTC1,
            QDEC  : p.QDEC,
            COMP  : p.COMP,
            LPCOMP: p.LPCOMP,
            SWI0  : p.SWI0,
            EGU0  : p.EGU0,
            SWI1  : p.SWI1,
            EGU1  : p.EGU1,
            SWI2  : p.SWI2,
            EGU2  : p.EGU2,
            SWI3  : p.SWI3,
            EGU3  : p.EGU3,
            SWI4  : p.SWI4,
            EGU4  : p.EGU4,
            SWI5  : p.SWI5,
            EGU5  : p.EGU5,
            TIMER3: p.TIMER3,
            TIMER4: p.TIMER4,
            PWM0  : p.PWM0,
            PDM   : p.PDM,
            NVMC  : p.NVMC,
            PPI   : p.PPI,
            MWU   : p.MWU,
            PWM1  : p.PWM1,
            PWM2  : p.PWM2,
            RTC2  : p.RTC2,
            I2S   : p.I2S,
        }
    }
}


/// The nRF52 pins that are available on the nRF52840DK
#[allow(non_snake_case)]
pub struct Pins {
    pub P0_03: p0::P0_03<Input<Floating>>,
    pub P0_04: p0::P0_04<Input<Floating>>,
       _RESET: p0::P0_18<Input<Floating>>,
    pub P0_22: p0::P0_22<Input<Floating>>,
    pub P0_23: p0::P0_23<Input<Floating>>,
    pub P0_26: p0::P0_26<Input<Floating>>,
    pub P0_27: p0::P0_27<Input<Floating>>,
    pub P0_28: p0::P0_28<Input<Floating>>,
    pub P0_29: p0::P0_29<Input<Floating>>,
    pub P0_30: p0::P0_30<Input<Floating>>,
    pub P0_31: p0::P0_31<Input<Floating>>,
    pub P1_00: p1::P1_00<Input<Floating>>,
    pub P1_01: p1::P1_01<Input<Floating>>,
    pub P1_02: p1::P1_02<Input<Floating>>,
    pub P1_03: p1::P1_03<Input<Floating>>,
    pub P1_04: p1::P1_04<Input<Floating>>,
    pub P1_05: p1::P1_05<Input<Floating>>,
    pub P1_06: p1::P1_06<Input<Floating>>,
    pub P1_07: p1::P1_07<Input<Floating>>,
    pub P1_08: p1::P1_08<Input<Floating>>,
    pub P1_09: p1::P1_09<Input<Floating>>,
    pub P1_10: p1::P1_10<Input<Floating>>,
    pub P1_11: p1::P1_11<Input<Floating>>,
    pub P1_12: p1::P1_12<Input<Floating>>,
    pub P1_13: p1::P1_13<Input<Floating>>,
    pub P1_14: p1::P1_14<Input<Floating>>,
    pub P1_15: p1::P1_15<Input<Floating>>,
}


/// The LEDs on the nRF52840-DK board
pub struct Leds {
    /// nRF52840-DK: LED1, nRF52: P0.30
    pub led_1: Led,

    /// nRF52840-DK: LED2, nRF52: P0.31
    pub led_2: Led,

    /// nRF52840-DK: LED3, nRF52: P0.22
    pub led_3: Led,

    /// nRF52840-DK: LED4, nRF52: P0.14
    pub led_4: Led,
}

/// An LED on the nRF52840-DK board
pub struct Led(Pin<Output<PushPull>>);

impl Led {
    fn new<Mode>(pin: Pin<Mode>) -> Self {
        Led(pin.into_push_pull_output(Level::High))
    }

    /// Enable the LED
    pub fn enable(&mut self) {
        self.0.set_low()
    }

    /// Disable the LED
    pub fn disable(&mut self) {
        self.0.set_high()
    }
}

/// The LEDs on the nRF52840-DK board
pub struct Buttons {
    /// nRF52840-DK: Button 1, nRF52: P0.30
    pub button_1: Button,

    /// nRF52840-DK: Button 2, nRF52: P0.31
    pub button_2: Button,

    /// nRF52840-DK: Button 3, nRF52: P0.22
    pub button_3: Button,

    /// nRF52840-DK: Button 4, nRF52: P0.14
    pub button_4: Button,
}

/// An LED on the nRF52840-DK board
pub struct Button(Pin<Input<PullUp>>);

impl Button {
    fn new<Mode>(pin: Pin<Mode>) -> Self {
        Button(pin.into_pullup_input())
    }
}

/// The LEDs on the nRF52840-DK board
pub struct NFC {
    /// nRF52840-DK: LED1, nRF52: P0.30
    pub nfc_1: p0::P0_09<Input<Floating>>,

    /// nRF52840-DK: LED2, nRF52: P0.31
    pub nfc_2: p0::P0_10<Input<Floating>>,
}
