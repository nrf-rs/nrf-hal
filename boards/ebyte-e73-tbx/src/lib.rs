//! Board support crate for the EBYTE E73-TBX boards
//! E73-TBA: http://ebyte.com/en/product-view-news.aspx?id=888
//! E73-TBB: http://ebyte.com/en/product-view-news.aspx?id=889
//!
#![no_std]

extern crate cortex_m;

extern crate cortex_m_rt;

#[cfg(feature = "tba")]
pub extern crate nrf52810_hal as hal;

#[cfg(feature = "tbb")]
pub extern crate nrf52832_hal as hal;

/// Exports traits that are usually needed when using this crate
pub mod prelude {
    pub use hal::prelude::*;
}

use hal::{
    gpio::{p0, Floating, Input, Level, Output, Pin, PullUp, PushPull},
    pac::{self as nrf52, CorePeripherals, Peripherals},
    uarte::{self, Baudrate as UartBaudrate, Parity as UartParity, Uarte},
};

use hal::prelude::{OutputPin, InputPin};

/// Provides access to all features of the E73-TBX boards
#[allow(non_snake_case)]
pub struct Board {
    /// The nRF52's pins which are not otherwise occupied on the E73-TBX
    pub pins: Pins,

    /// The E73-TBX UART which is wired to the virtual USB CDC port
    pub cdc_uart: Uarte<nrf52::UARTE0>,

    /// The LEDs on the E73-TBX boards
    pub leds: Leds,

    /// The buttons on the E73-TBX boards
    pub buttons: Buttons,

    #[cfg(feature = "tbb")]
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
    #[cfg(feature = "tbb")]
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
    #[cfg(feature = "tbb")]
    pub SPIM1: nrf52::SPIM1,

    /// nRF52 peripheral: SPIS1
    #[cfg(feature = "tbb")]
    pub SPIS1: nrf52::SPIS1,

    /// nRF52 peripheral: TWIS1
    #[cfg(feature = "tbb")]
    pub TWIS1: nrf52::TWIS1,

    /// nRF52 peripheral: SPI1
    #[cfg(feature = "tbb")]
    pub SPI1: nrf52::SPI1,

    /// nRF52 peripheral: TWI1
    #[cfg(feature = "tbb")]
    pub TWI1: nrf52::TWI1,

    /// nRF52 peripheral: NFCT
    #[cfg(feature = "tbb")]
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
    #[cfg(feature = "tbb")]
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
    #[cfg(feature = "tbb")]
    pub EGU2: nrf52::EGU2,

    /// nRF52 peripheral: SWI3
    pub SWI3: nrf52::SWI3,

    /// nRF52 peripheral: EGU3
    #[cfg(feature = "tbb")]
    pub EGU3: nrf52::EGU3,

    /// nRF52 peripheral: SWI4
    pub SWI4: nrf52::SWI4,

    /// nRF52 peripheral: EGU4
    #[cfg(feature = "tbb")]
    pub EGU4: nrf52::EGU4,

    /// nRF52 peripheral: SWI5
    pub SWI5: nrf52::SWI5,

    /// nRF52 peripheral: EGU5
    #[cfg(feature = "tbb")]
    pub EGU5: nrf52::EGU5,

    /// nRF52 peripheral: TIMER3
    #[cfg(feature = "tbb")]
    pub TIMER3: nrf52::TIMER3,

    /// nRF52 peripheral: TIMER4
    #[cfg(feature = "tbb")]
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
    #[cfg(feature = "tbb")]
    pub MWU: nrf52::MWU,

    /// nRF52 peripheral: PWM1
    #[cfg(feature = "tbb")]
    pub PWM1: nrf52::PWM1,

    /// nRF52 peripheral: PWM2
    #[cfg(feature = "tbb")]
    pub PWM2: nrf52::PWM2,

    /// nRF52 peripheral: RTC2
    #[cfg(feature = "tbb")]
    pub RTC2: nrf52::RTC2,

    /// nRF52 peripheral: I2S
    #[cfg(feature = "tbb")]
    pub I2S: nrf52::I2S,
}

impl Board {
    /// Take the peripherals safely
    ///
    /// This method will return an instance of `E73TBX` the first time it is
    /// called. It will return only `None` on subsequent calls.
    pub fn take() -> Option<Self> {
        Some(Self::new(CorePeripherals::take()?, Peripherals::take()?))
    }

    /// Steal the peripherals
    ///
    /// This method produces an instance of `E73TBX`, regardless of whether
    /// another instance was create previously.
    ///
    /// # Safety
    ///
    /// This method can be used to create multiple instances of `E73TBX`. Those
    /// instances can interfere with each other, causing all kinds of unexpected
    /// behavior and circumventing safety guarantees in many ways.
    ///
    /// Always use `E73TBX::take`, unless you really know what you're doing.
    pub unsafe fn steal() -> Self {
        Self::new(CorePeripherals::steal(), Peripherals::steal())
    }

    fn new(cp: CorePeripherals, p: Peripherals) -> Self {
        let pins0 = p0::Parts::new(p.P0);

        // The E73-TBX features an USB CDC port.
        let cdc_uart = Uarte::new(
            p.UARTE0,
            uarte::Pins {
                txd: pins0.p0_06.into_push_pull_output(Level::High).degrade(),
                rxd: pins0.p0_08.into_floating_input().degrade(),
                cts: Some(pins0.p0_07.into_floating_input().degrade()),
                rts: Some(pins0.p0_05.into_push_pull_output(Level::High).degrade()),
            },
            UartParity::EXCLUDED,
            UartBaudrate::BAUD115200,
        );

        Board {
            cdc_uart: cdc_uart,

            pins: Pins {
                // P1
                P0_11: pins0.p0_11,
                P0_12: pins0.p0_12,
                P0_15: pins0.p0_15,
                P0_16: pins0.p0_16,
                P0_19: pins0.p0_19,
                P0_20: pins0.p0_20,
                P0_21: pins0.p0_21,
                P0_22: pins0.p0_22,
                P0_23: pins0.p0_23,
                P0_24: pins0.p0_24,

                // P2
                P0_04: pins0.p0_04,
                P0_03: pins0.p0_03,
                P0_02: pins0.p0_02,
                P0_31: pins0.p0_31,
                P0_30: pins0.p0_30,
                P0_29: pins0.p0_29,
                P0_28: pins0.p0_28,
                P0_27: pins0.p0_27,
                P0_26: pins0.p0_26,
                P0_25: pins0.p0_25,
            },

            leds: Leds {
                led_1: Led::new(pins0.p0_17.degrade()),
                led_2: Led::new(pins0.p0_18.degrade()),
            },

            buttons: Buttons {
                button_1: Button::new(pins0.p0_14.degrade()),
                button_2: Button::new(pins0.p0_13.degrade()),
            },

            #[cfg(feature = "tbb")]
            nfc: NFC {
                nfc_1: pins0.p0_09,
                nfc_2: pins0.p0_10,
            },

            // Core peripherals
            CBP: cp.CBP,
            CPUID: cp.CPUID,
            DCB: cp.DCB,
            DWT: cp.DWT,
            FPB: cp.FPB,
            #[cfg(feature = "tbb")]
            FPU: cp.FPU,
            ITM: cp.ITM,
            MPU: cp.MPU,
            NVIC: cp.NVIC,
            SCB: cp.SCB,
            SYST: cp.SYST,
            TPIU: cp.TPIU,

            // nRF52 peripherals
            FICR: p.FICR,
            UICR: p.UICR,
            POWER: p.POWER,
            CLOCK: p.CLOCK,
            RADIO: p.RADIO,

            UART0: p.UART0,
            SPIM0: p.SPIM0,
            SPIS0: p.SPIS0,
            TWIM0: p.TWIM0,
            TWIS0: p.TWIS0,
            SPI0: p.SPI0,
            TWI0: p.TWI0,
            #[cfg(feature = "tbb")]
            SPIM1: p.SPIM1,
            #[cfg(feature = "tbb")]
            SPIS1: p.SPIS1,
            #[cfg(feature = "tbb")]
            TWIS1: p.TWIS1,
            #[cfg(feature = "tbb")]
            SPI1: p.SPI1,
            #[cfg(feature = "tbb")]
            TWI1: p.TWI1,
            #[cfg(feature = "tbb")]
            NFCT: p.NFCT,
            GPIOTE: p.GPIOTE,
            SAADC: p.SAADC,
            TIMER0: p.TIMER0,
            TIMER1: p.TIMER1,
            TIMER2: p.TIMER2,
            RTC0: p.RTC0,
            TEMP: p.TEMP,
            RNG: p.RNG,
            ECB: p.ECB,
            CCM: p.CCM,
            AAR: p.AAR,
            WDT: p.WDT,
            RTC1: p.RTC1,
            QDEC: p.QDEC,
            COMP: p.COMP,
            #[cfg(feature = "tbb")]
            LPCOMP: p.LPCOMP,
            SWI0: p.SWI0,
            EGU0: p.EGU0,
            SWI1: p.SWI1,
            EGU1: p.EGU1,
            SWI2: p.SWI2,
            #[cfg(feature = "tbb")]
            EGU2: p.EGU2,
            SWI3: p.SWI3,
            #[cfg(feature = "tbb")]
            EGU3: p.EGU3,
            SWI4: p.SWI4,
            #[cfg(feature = "tbb")]
            EGU4: p.EGU4,
            SWI5: p.SWI5,
            #[cfg(feature = "tbb")]
            EGU5: p.EGU5,
            #[cfg(feature = "tbb")]
            TIMER3: p.TIMER3,
            #[cfg(feature = "tbb")]
            TIMER4: p.TIMER4,
            PWM0: p.PWM0,
            PDM: p.PDM,
            NVMC: p.NVMC,
            PPI: p.PPI,
            #[cfg(feature = "tbb")]
            MWU: p.MWU,
            #[cfg(feature = "tbb")]
            PWM1: p.PWM1,
            #[cfg(feature = "tbb")]
            PWM2: p.PWM2,
            #[cfg(feature = "tbb")]
            RTC2: p.RTC2,
            #[cfg(feature = "tbb")]
            I2S: p.I2S,
        }
    }
}

/// The nRF52 pins that are available on the nRF52DK
#[allow(non_snake_case)]
pub struct Pins {
    // P1
    pub P0_11: p0::P0_11<Input<Floating>>,
    pub P0_12: p0::P0_12<Input<Floating>>,
    pub P0_15: p0::P0_15<Input<Floating>>,
    pub P0_16: p0::P0_16<Input<Floating>>,
    pub P0_19: p0::P0_19<Input<Floating>>,
    pub P0_20: p0::P0_20<Input<Floating>>,
    pub P0_21: p0::P0_21<Input<Floating>>,
    pub P0_22: p0::P0_22<Input<Floating>>,
    pub P0_23: p0::P0_23<Input<Floating>>,
    pub P0_24: p0::P0_24<Input<Floating>>,

    // P2
    pub P0_04: p0::P0_04<Input<Floating>>,
    pub P0_03: p0::P0_03<Input<Floating>>,
    pub P0_02: p0::P0_02<Input<Floating>>,
    pub P0_31: p0::P0_31<Input<Floating>>,
    pub P0_30: p0::P0_30<Input<Floating>>,
    pub P0_29: p0::P0_29<Input<Floating>>,
    pub P0_28: p0::P0_28<Input<Floating>>,
    pub P0_27: p0::P0_27<Input<Floating>>,
    pub P0_26: p0::P0_26<Input<Floating>>,
    pub P0_25: p0::P0_25<Input<Floating>>,
}

/// The LEDs on the E73-TBX boards
pub struct Leds {
    /// E73-TBX: LED1, nRF52: P0.17
    pub led_1: Led,

    /// E73-TBX: LED2, nRF52: P0.18
    pub led_2: Led,
}

/// An LED on the E73-TBX boards
pub struct Led(Pin<Output<PushPull>>);

impl Led {
    fn new<Mode>(pin: Pin<Mode>) -> Self {
        Led(pin.into_push_pull_output(Level::High))
    }

    /// Enable the LED
    pub fn enable(&mut self) {
        self.0.set_low().unwrap()
    }

    /// Disable the LED
    pub fn disable(&mut self) {
        self.0.set_high().unwrap()
    }
}

/// The Buttons on the E73-TBX boards
pub struct Buttons {
    /// E73-TBX: Button 1, nRF52: P0.14
    pub button_1: Button,

    /// E73-TBX: Button 2, nRF52: P0.13
    pub button_2: Button,
}

/// A Button on the E73-TBX boards
pub struct Button(Pin<Input<PullUp>>);

impl Button {
    fn new<Mode>(pin: Pin<Mode>) -> Self {
        Button(pin.into_pullup_input())
    }

    pub fn is_active(&self) -> bool {
        self.0.is_low().unwrap()
    }
}

/// The NFC pins on the E73-TBX boards
#[cfg(feature = "tbb")]
pub struct NFC {
    /// E73-TBB: NFC1, nRF52: P0.09
    pub nfc_1: p0::P0_09<Input<Floating>>,

    /// E73-TBB: NFC2, nRF52: P0.10
    pub nfc_2: p0::P0_10<Input<Floating>>,
}
