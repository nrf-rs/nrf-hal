#![no_std]

pub mod prelude {
    pub use nrf52832_hal::prelude::*;
}

use nrf52832_hal::{
    gpio::{p0, Floating, Input, Level, Output, Pin, PushPull},
    prelude::*,
    target::{self as pac, CorePeripherals, Peripherals},
    uarte, Uarte,
};

#[allow(non_snake_case)]
pub struct Board {
    pub pins: Pins,

    pub cdc: Uarte<pac::UARTE0>,

    pub leds: Leds,

    pub nfc: NFC,

    /// Core peripheral: Cache and branch predictor maintenance operations
    pub CBP: pac::CBP,

    /// Core peripheral: CPUID
    pub CPUID: pac::CPUID,

    /// Core peripheral: Debug Control Block
    pub DCB: pac::DCB,

    /// Core peripheral: Data Watchpoint and Trace unit
    pub DWT: pac::DWT,

    /// Core peripheral: Flash Patch and Breakpoint unit
    pub FPB: pac::FPB,

    /// Core peripheral: Floating Point Unit
    pub FPU: pac::FPU,

    /// Core peripheral: Instrumentation Trace Macrocell
    pub ITM: pac::ITM,

    /// Core peripheral: Memory Protection Unit
    pub MPU: pac::MPU,

    /// Core peripheral: Nested Vector Interrupt Controller
    pub NVIC: pac::NVIC,

    /// Core peripheral: System Control Block
    pub SCB: pac::SCB,

    /// Core peripheral: SysTick Timer
    pub SYST: pac::SYST,

    /// Core peripheral: Trace Port Interface Unit
    pub TPIU: pac::TPIU,

    /// nRF52 peripheral: FICR
    pub FICR: pac::FICR,

    /// nRF52 peripheral: UICR
    pub UICR: pac::UICR,

    /// nRF52 peripheral: POWER
    pub POWER: pac::POWER,

    /// nRF52 peripheral: CLOCK
    pub CLOCK: pac::CLOCK,

    /// nRF52 peripheral: RADIO
    pub RADIO: pac::RADIO,

    /// nRF52 peripheral: UART0
    pub UART0: pac::UART0,

    /// nRF52 peripheral: SPIM0
    pub SPIM0: pac::SPIM0,

    /// nRF52 peripheral: SPIS0
    pub SPIS0: pac::SPIS0,

    /// nRF52 peripheral: TWIM0
    pub TWIM0: pac::TWIM0,

    /// nRF52 peripheral: TWIS0
    pub TWIS0: pac::TWIS0,

    /// nRF52 peripheral: SPI0
    pub SPI0: pac::SPI0,

    /// nRF52 peripheral: TWI0
    pub TWI0: pac::TWI0,

    /// nRF52 peripheral: SPIM1
    pub SPIM1: pac::SPIM1,

    /// nRF52 peripheral: SPIS1
    pub SPIS1: pac::SPIS1,

    /// nRF52 peripheral: TWIS1
    pub TWIS1: pac::TWIS1,

    /// nRF52 peripheral: SPI1
    pub SPI1: pac::SPI1,

    /// nRF52 peripheral: TWI1
    pub TWI1: pac::TWI1,

    /// nRF52 peripheral: NFCT
    pub NFCT: pac::NFCT,

    /// nRF52 peripheral: GPIOTE
    pub GPIOTE: pac::GPIOTE,

    /// nRF52 peripheral: SAADC
    pub SAADC: pac::SAADC,

    /// nRF52 peripheral: TIMER0
    pub TIMER0: pac::TIMER0,

    /// nRF52 peripheral: TIMER1
    pub TIMER1: pac::TIMER1,

    /// nRF52 peripheral: TIMER2
    pub TIMER2: pac::TIMER2,

    /// nRF52 peripheral: RTC0
    pub RTC0: pac::RTC0,

    /// nRF52 peripheral: TEMP
    pub TEMP: pac::TEMP,

    /// nRF52 peripheral: RNG
    pub RNG: pac::RNG,

    /// nRF52 peripheral: ECB
    pub ECB: pac::ECB,

    /// nRF52 peripheral: CCM
    pub CCM: pac::CCM,

    /// nRF52 peripheral: AAR
    pub AAR: pac::AAR,

    /// nRF52 peripheral: WDT
    pub WDT: pac::WDT,

    /// nRF52 peripheral: RTC1
    pub RTC1: pac::RTC1,

    /// nRF52 peripheral: QDEC
    pub QDEC: pac::QDEC,

    /// nRF52 peripheral: COMP
    pub COMP: pac::COMP,

    /// nRF52 peripheral: LPCOMP
    pub LPCOMP: pac::LPCOMP,

    /// nRF52 peripheral: SWI0
    pub SWI0: pac::SWI0,

    /// nRF52 peripheral: EGU0
    pub EGU0: pac::EGU0,

    /// nRF52 peripheral: SWI1
    pub SWI1: pac::SWI1,

    /// nRF52 peripheral: EGU1
    pub EGU1: pac::EGU1,

    /// nRF52 peripheral: SWI2
    pub SWI2: pac::SWI2,

    /// nRF52 peripheral: EGU2
    pub EGU2: pac::EGU2,

    /// nRF52 peripheral: SWI3
    pub SWI3: pac::SWI3,

    /// nRF52 peripheral: EGU3
    pub EGU3: pac::EGU3,

    /// nRF52 peripheral: SWI4
    pub SWI4: pac::SWI4,

    /// nRF52 peripheral: EGU4
    pub EGU4: pac::EGU4,

    /// nRF52 peripheral: SWI5
    pub SWI5: pac::SWI5,

    /// nRF52 peripheral: EGU5
    pub EGU5: pac::EGU5,

    /// nRF52 peripheral: TIMER3
    pub TIMER3: pac::TIMER3,

    /// nRF52 peripheral: TIMER4
    pub TIMER4: pac::TIMER4,

    /// nRF52 peripheral: PWM0
    pub PWM0: pac::PWM0,

    /// nRF52 peripheral: PDM
    pub PDM: pac::PDM,

    /// nRF52 peripheral: NVMC
    pub NVMC: pac::NVMC,

    /// nRF52 peripheral: PPI
    pub PPI: pac::PPI,

    /// nRF52 peripheral: MWU
    pub MWU: pac::MWU,

    /// nRF52 peripheral: PWM1
    pub PWM1: pac::PWM1,

    /// nRF52 peripheral: PWM2
    pub PWM2: pac::PWM2,

    /// nRF52 peripheral: RTC2
    pub RTC2: pac::RTC2,

    /// nRF52 peripheral: I2S
    pub I2S: pac::I2S,
}

impl Board {
    pub fn take() -> Option<Self> {
        Some(Self::new(CorePeripherals::take()?, Peripherals::take()?))
    }

    pub unsafe fn steal() -> Self {
        Self::new(CorePeripherals::steal(), Peripherals::steal())
    }

    pub fn new(cp: CorePeripherals, p: Peripherals) -> Self {
        let pins = p0::Parts::new(p.P0);

        let cdc_uarte = Uarte::new(
            p.UARTE0,
            uarte::Pins {
                txd: pins.p0_06.into_push_pull_output(Level::High).degrade(),
                rxd: pins.p0_08.into_floating_input().degrade(),
                cts: None,
                rts: None,
            },
            uarte::Parity::EXCLUDED,
            uarte::Baudrate::BAUD115200,
        );

        Self {
            cdc: cdc_uarte,

            nfc: NFC {
                nfc1: pins.p0_09,
                nfc2: pins.p0_10,
            },

            pins: Pins {
                a0: pins.p0_02,
                a1: pins.p0_03,
                a2: pins.p0_04,
                a3: pins.p0_05,
                a4: pins.p0_28,
                a5: pins.p0_29,
                sck: pins.p0_12,
                mosi: pins.p0_13,
                miso: pins.p0_14,
                dfu: pins.p0_20,
                frst: pins.p0_22,
                d16: pins.p0_16,
                d15: pins.p0_15,
                d7: pins.p0_07,
                d11: pins.p0_11,
                a7: pins.p0_31,
                a6: pins.p0_30,
                d27: pins.p0_27,
                scl: pins.p0_26,
                sda: pins.p0_25,
            },
            leds: Leds {
                red: Led::new(pins.p0_17.degrade()),
                blue: Led::new(pins.p0_19.degrade()),
            },

            // Core peripherals
            CBP: cp.CBP,
            CPUID: cp.CPUID,
            DCB: cp.DCB,
            DWT: cp.DWT,
            FPB: cp.FPB,
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
            SPIM1: p.SPIM1,
            SPIS1: p.SPIS1,
            TWIS1: p.TWIS1,
            SPI1: p.SPI1,
            TWI1: p.TWI1,
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
            LPCOMP: p.LPCOMP,
            SWI0: p.SWI0,
            EGU0: p.EGU0,
            SWI1: p.SWI1,
            EGU1: p.EGU1,
            SWI2: p.SWI2,
            EGU2: p.EGU2,
            SWI3: p.SWI3,
            EGU3: p.EGU3,
            SWI4: p.SWI4,
            EGU4: p.EGU4,
            SWI5: p.SWI5,
            EGU5: p.EGU5,
            TIMER3: p.TIMER3,
            TIMER4: p.TIMER4,
            PWM0: p.PWM0,
            PDM: p.PDM,
            NVMC: p.NVMC,
            PPI: p.PPI,
            MWU: p.MWU,
            PWM1: p.PWM1,
            PWM2: p.PWM2,
            RTC2: p.RTC2,
            I2S: p.I2S,
        }
    }
}

pub struct Leds {
    pub red: Led,
    pub blue: Led,
}

pub struct Led(Pin<Output<PushPull>>);

impl Led {
    fn new<Mode>(pin: Pin<Mode>) -> Self {
        Led(pin.into_push_pull_output(Level::High))
    }

    pub fn enable(&mut self) {
        self.0.set_high();
    }

    pub fn disable(&mut self) {
        self.0.set_low();
    }
}

/// Maps the pins to the names printed on the device
pub struct Pins {
    pub a0: p0::P0_02<Input<Floating>>,
    pub a1: p0::P0_03<Input<Floating>>,
    pub a2: p0::P0_04<Input<Floating>>,
    pub a3: p0::P0_05<Input<Floating>>,
    pub a4: p0::P0_28<Input<Floating>>,
    pub a5: p0::P0_29<Input<Floating>>,
    pub sck: p0::P0_12<Input<Floating>>,
    pub mosi: p0::P0_13<Input<Floating>>,
    pub miso: p0::P0_14<Input<Floating>>,
    pub dfu: p0::P0_20<Input<Floating>>,
    pub frst: p0::P0_22<Input<Floating>>,
    pub d16: p0::P0_16<Input<Floating>>,
    pub d15: p0::P0_15<Input<Floating>>,
    pub d7: p0::P0_07<Input<Floating>>,
    pub d11: p0::P0_11<Input<Floating>>,
    pub a7: p0::P0_31<Input<Floating>>,
    pub a6: p0::P0_30<Input<Floating>>,
    pub d27: p0::P0_27<Input<Floating>>,
    pub scl: p0::P0_26<Input<Floating>>,
    pub sda: p0::P0_25<Input<Floating>>,
}

pub struct NFC {
    pub nfc1: p0::P0_09<Input<Floating>>,
    pub nfc2: p0::P0_10<Input<Floating>>,
}
