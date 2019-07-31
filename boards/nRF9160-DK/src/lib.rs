//! Board support crate for the Nordic nRF9160-DK
//! https://www.nordicsemi.com/Software-and-Tools/Development-Kits/nRF9160-DK
//!
#![no_std]

extern crate cortex_m;

extern crate cortex_m_rt;
pub extern crate nrf9160_hal as hal;

/// Exports traits that are usually needed when using this crate
pub mod prelude {
    pub use hal::prelude::*;
}

use hal::{
    gpio::{p0, Floating, Input, Level, Output, Pin, PullUp, PushPull},
    pac::{CorePeripherals, Peripherals},
    prelude::*,
    uarte::{self, Baudrate as UartBaudrate, Parity as UartParity, Uarte},
};

pub use hal::pac;

/// Provides access to all features of the nRF9160-DK board
#[allow(non_snake_case)]
pub struct Board {
    /// The nRF9160's pins which are not otherwise occupied on the nRF9160-DK
    pub pins: Pins,

    /// The nRF9160-DK UART which is wired to the virtual USB CDC port
    pub cdc_uart: Uarte<pac::UARTE0_NS>,

    /// The LEDs on the nRF9160-DK board
    pub leds: Leds,

    /// The buttons on the nRF9160-DK board
    pub buttons: Buttons,

    /// Cortex-M33 Core peripheral: Cache and branch predictor maintenance operations
    pub CBP: pac::CBP,

    /// Cortex-M33 Core peripheral: CPUID
    pub CPUID: pac::CPUID,

    /// Cortex-M33 Core peripheral: Debug Control Block
    pub DCB: pac::DCB,

    /// Cortex-M33 Core peripheral: Data Watchpoint and Trace unit
    pub DWT: pac::DWT,

    /// Cortex-M33 Core peripheral: Flash Patch and Breakpoint unit
    pub FPB: pac::FPB,

    /// Cortex-M33 Core peripheral: Floating Point Unit
    pub FPU: pac::FPU,

    /// Cortex-M33 Core peripheral: Instrumentation Trace Macrocell
    pub ITM: pac::ITM,

    /// Cortex-M33 Core peripheral: Memory Protection Unit
    pub MPU: pac::MPU,

    /// Cortex-M33 Core peripheral: Nested Vector Interrupt Controller
    pub NVIC: pac::NVIC,

    /// Cortex-M33 Core peripheral: System Control Block
    pub SCB: pac::SCB,

    /// Cortex-M33 Core peripheral: SysTick Timer
    pub SYST: pac::SYST,

    /// Cortex-M33 Core peripheral: Trace Port Interface Unit
    pub TPIU: pac::TPIU,

    /// nRF9160 Non-secure peripheral: Clock management 0
    pub CLOCK_NS: pac::CLOCK_NS,

    /// nRF9160 Non-secure peripheral: Distributed Programmable Peripheral Interconnect Controller 0
    pub DPPIC_NS: pac::DPPIC_NS,

    /// nRF9160 Non-secure peripheral: Event Generator Unit 0
    pub EGU0_NS: pac::EGU0_NS,

    /// nRF9160 Non-secure peripheral: Event Generator Unit 1
    pub EGU1_NS: pac::EGU1_NS,

    /// nRF9160 Non-secure peripheral: Event Generator Unit 2
    pub EGU2_NS: pac::EGU2_NS,

    /// nRF9160 Non-secure peripheral: Event Generator Unit 3
    pub EGU3_NS: pac::EGU3_NS,

    /// nRF9160 Non-secure peripheral: Event Generator Unit 3
    pub EGU4_NS: pac::EGU4_NS,

    /// nRF9160 Non-secure peripheral: Event Generator Unit 5
    pub EGU5_NS: pac::EGU5_NS,

    /// nRF9160 Non-secure peripheral: FPU 0
    pub FPU_NS: pac::FPU_NS,

    /// nRF9160 Non-secure peripheral: GPIO Tasks and Events 1
    pub GPIOTE1_NS: pac::GPIOTE1_NS,

    /// nRF9160 Non-secure peripheral: Inter-IC Sound 0
    pub I2S_NS: pac::I2S_NS,

    /// nRF9160 Non-secure peripheral: Inter Processor Communication 0
    pub IPC_NS: pac::IPC_NS,

    /// nRF9160 Non-secure peripheral: Key management unit 0
    pub KMU_NS: pac::KMU_NS,

    /// nRF9160 Non-secure peripheral: Non-volatile memory controller 0
    pub NVMC_NS: pac::NVMC_NS,

    /// nRF9160 Non-secure peripheral: Pulse Density Modulation (Digital Microphone) Interface 0
    pub PDM_NS: pac::PDM_NS,

    /// nRF9160 Non-secure peripheral: Power control 0
    pub POWER_NS: pac::POWER_NS,

    /// nRF9160 Non-secure peripheral: Pulse width modulation unit 0
    pub PWM0_NS: pac::PWM0_NS,

    /// nRF9160 Non-secure peripheral: Pulse width modulation unit 1
    pub PWM1_NS: pac::PWM1_NS,

    /// nRF9160 Non-secure peripheral: Pulse width modulation unit 2
    pub PWM2_NS: pac::PWM2_NS,

    /// nRF9160 Non-secure peripheral: Pulse width modulation unit 3
    pub PWM3_NS: pac::PWM3_NS,

    /// nRF9160 Non-secure peripheral: Voltage regulators control 0
    pub REGULATORS_NS: pac::REGULATORS_NS,

    /// nRF9160 Non-secure peripheral: Real-time counter 0
    pub RTC0_NS: pac::RTC0_NS,

    /// nRF9160 Non-secure peripheral: Real-time counter 1
    pub RTC1_NS: pac::RTC1_NS,

    /// nRF9160 Non-secure peripheral: Analog to Digital Converter 0
    pub SAADC_NS: pac::SAADC_NS,

    /// nRF9160 Non-secure peripheral: Serial Peripheral Interface Master with EasyDMA 0
    pub SPIM0_NS: pac::SPIM0_NS,

    /// nRF9160 Non-secure peripheral: Serial Peripheral Interface Master with EasyDMA 1
    pub SPIM1_NS: pac::SPIM1_NS,

    /// nRF9160 Non-secure peripheral: Serial Peripheral Interface Master with EasyDMA 2
    pub SPIM2_NS: pac::SPIM2_NS,

    /// nRF9160 Non-secure peripheral: Serial Peripheral Interface Master with EasyDMA 3
    pub SPIM3_NS: pac::SPIM3_NS,

    /// nRF9160 Non-secure peripheral: SPI Slave 0
    pub SPIS0_NS: pac::SPIS0_NS,

    /// nRF9160 Non-secure peripheral: SPI Slave 1
    pub SPIS1_NS: pac::SPIS1_NS,

    /// nRF9160 Non-secure peripheral: SPI Slave 2
    pub SPIS2_NS: pac::SPIS2_NS,

    /// nRF9160 Non-secure peripheral: SPI Slave 3
    pub SPIS3_NS: pac::SPIS3_NS,

    /// nRF9160 Non-secure peripheral: Timer/Counter 0
    pub TIMER0_NS: pac::TIMER0_NS,

    /// nRF9160 Non-secure peripheral: Timer/Counter 1
    pub TIMER1_NS: pac::TIMER1_NS,

    /// nRF9160 Non-secure peripheral: Timer/Counter 2
    pub TIMER2_NS: pac::TIMER2_NS,

    /// nRF9160 Non-secure peripheral: I2C compatible Two-Wire Master Interface with EasyDMA 0
    pub TWIM0_NS: pac::TWIM0_NS,

    /// nRF9160 Non-secure peripheral: I2C compatible Two-Wire Master Interface with EasyDMA 1
    pub TWIM1_NS: pac::TWIM1_NS,

    /// nRF9160 Non-secure peripheral: I2C compatible Two-Wire Master Interface with EasyDMA 2
    pub TWIM2_NS: pac::TWIM2_NS,

    /// nRF9160 Non-secure peripheral: I2C compatible Two-Wire Master Interface with EasyDMA 3
    pub TWIM3_NS: pac::TWIM3_NS,

    /// nRF9160 Non-secure peripheral: I2C compatible Two-Wire Slave Interface with EasyDMA 0
    pub TWIS0_NS: pac::TWIS0_NS,

    /// nRF9160 Non-secure peripheral: I2C compatible Two-Wire Slave Interface with EasyDMA 1
    pub TWIS1_NS: pac::TWIS1_NS,

    /// nRF9160 Non-secure peripheral: I2C compatible Two-Wire Slave Interface with EasyDMA 2
    pub TWIS2_NS: pac::TWIS2_NS,

    /// nRF9160 Non-secure peripheral: I2C compatible Two-Wire Slave Interface with EasyDMA 3
    pub TWIS3_NS: pac::TWIS3_NS,

    /// nRF9160 Non-secure peripheral: UART with EasyDMA 1
    pub UARTE1_NS: pac::UARTE1_NS,

    /// nRF9160 Non-secure peripheral: UART with EasyDMA 2
    pub UARTE2_NS: pac::UARTE2_NS,

    /// nRF9160 Non-secure peripheral: UART with EasyDMA 3
    pub UARTE3_NS: pac::UARTE3_NS,

    /// nRF9160 Non-secure peripheral: Volatile Memory controller 0
    pub VMC_NS: pac::VMC_NS,

    /// nRF9160 Non-secure peripheral: Watchdog Timer 0
    pub WDT_NS: pac::WDT_NS,
}

/// Contains all the 'secure' mode peripherals. The HAL doesn't support these
/// yet but at least they're all together.
#[allow(non_snake_case)]
pub struct SecurePeripherals {
    /// nRF9160 Secure peripheral: Clock management 1
    pub CLOCK_S: pac::CLOCK_S,

    /// nRF9160 Secure peripheral: ARM TrustZone CryptoCell register interface
    pub CRYPTOCELL_S: pac::CRYPTOCELL_S,

    /// nRF9160 Secure peripheral: Control access port
    pub CTRL_AP_PERI_S: pac::CTRL_AP_PERI_S,

    /// nRF9160 Secure peripheral: Distributed Programmable Peripheral Interconnect Controller 1
    pub DPPIC_S: pac::DPPIC_S,

    /// nRF9160 Secure peripheral: Event Generator Unit 1
    pub EGU0_S: pac::EGU0_S,

    /// nRF9160 Secure peripheral: Event Generator Unit 3
    pub EGU1_S: pac::EGU1_S,

    /// nRF9160 Secure peripheral: Event Generator Unit 5
    pub EGU2_S: pac::EGU2_S,

    /// nRF9160 Secure peripheral: Event Generator Unit 7
    pub EGU3_S: pac::EGU3_S,

    /// nRF9160 Secure peripheral: Event Generator Unit 9
    pub EGU4_S: pac::EGU4_S,

    /// nRF9160 Secure peripheral: Event Generator Unit 11
    pub EGU5_S: pac::EGU5_S,

    /// nRF9160 Secure peripheral: Factory Information Configuration Registers
    pub FICR_S: pac::FICR_S,

    /// nRF9160 Secure peripheral: FPU 1
    pub FPU_S: pac::FPU_S,

    /// nRF9160 Secure peripheral: GPIO Tasks and Events 0
    pub GPIOTE0_S: pac::GPIOTE0_S,

    /// nRF9160 Secure peripheral: Inter-IC Sound 1
    pub I2S_S: pac::I2S_S,

    /// nRF9160 Secure peripheral: Inter Processor Communication 1
    pub IPC_S: pac::IPC_S,

    /// nRF9160 Secure peripheral: Key management unit 1
    pub KMU_S: pac::KMU_S,

    /// nRF9160 Secure peripheral: Non-volatile memory controller 1
    pub NVMC_S: pac::NVMC_S,

    /// nRF9160 Secure peripheral: GPIO Port 1
    pub P0_S: pac::P0_S,

    /// nRF9160 Secure peripheral: Pulse Density Modulation (Digital Microphone) Interface 1
    pub PDM_S: pac::PDM_S,

    /// nRF9160 Secure peripheral: Power control 1
    pub POWER_S: pac::POWER_S,

    /// nRF9160 Secure peripheral: Pulse width modulation unit 1
    pub PWM0_S: pac::PWM0_S,

    /// nRF9160 Secure peripheral: Pulse width modulation unit 3
    pub PWM1_S: pac::PWM1_S,

    /// nRF9160 Secure peripheral: Pulse width modulation unit 5
    pub PWM2_S: pac::PWM2_S,

    /// nRF9160 Secure peripheral: Pulse width modulation unit 7
    pub PWM3_S: pac::PWM3_S,

    /// nRF9160 Secure peripheral: Voltage regulators control 1
    pub REGULATORS_S: pac::REGULATORS_S,

    /// nRF9160 Secure peripheral: Real-time counter 1
    pub RTC0_S: pac::RTC0_S,

    /// nRF9160 Secure peripheral: Real-time counter 3
    pub RTC1_S: pac::RTC1_S,

    /// nRF9160 Secure peripheral: Analog to Digital Converter 1
    pub SAADC_S: pac::SAADC_S,

    /// nRF9160 Secure peripheral: Serial Peripheral Interface Master with EasyDMA 1
    pub SPIM0_S: pac::SPIM0_S,

    /// nRF9160 Secure peripheral: Serial Peripheral Interface Master with EasyDMA 3
    pub SPIM1_S: pac::SPIM1_S,

    /// nRF9160 Secure peripheral: Serial Peripheral Interface Master with EasyDMA 5
    pub SPIM2_S: pac::SPIM2_S,

    /// nRF9160 Secure peripheral: Serial Peripheral Interface Master with EasyDMA 7
    pub SPIM3_S: pac::SPIM3_S,

    /// nRF9160 Secure peripheral: SPI Slave 1
    pub SPIS0_S: pac::SPIS0_S,

    /// nRF9160 Secure peripheral: SPI Slave 3
    pub SPIS1_S: pac::SPIS1_S,

    /// nRF9160 Secure peripheral: SPI Slave 5
    pub SPIS2_S: pac::SPIS2_S,

    /// nRF9160 Secure peripheral: SPI Slave 7
    pub SPIS3_S: pac::SPIS3_S,

    /// nRF9160 Secure peripheral: System protection unit
    pub SPU_S: pac::SPU_S,

    /// nRF9160 Secure peripheral: Trace and debug control
    pub TAD_S: pac::TAD_S,

    /// nRF9160 Secure peripheral: Timer/Counter 1
    pub TIMER0_S: pac::TIMER0_S,

    /// nRF9160 Secure peripheral: Timer/Counter 3
    pub TIMER1_S: pac::TIMER1_S,

    /// nRF9160 Secure peripheral: Timer/Counter 5
    pub TIMER2_S: pac::TIMER2_S,

    /// nRF9160 Secure peripheral: I2C compatible Two-Wire Master Interface with EasyDMA 1
    pub TWIM0_S: pac::TWIM0_S,

    /// nRF9160 Secure peripheral: I2C compatible Two-Wire Master Interface with EasyDMA 3
    pub TWIM1_S: pac::TWIM1_S,

    /// nRF9160 Secure peripheral: I2C compatible Two-Wire Master Interface with EasyDMA 5
    pub TWIM2_S: pac::TWIM2_S,

    /// nRF9160 Secure peripheral: I2C compatible Two-Wire Master Interface with EasyDMA 7
    pub TWIM3_S: pac::TWIM3_S,

    /// nRF9160 Secure peripheral: I2C compatible Two-Wire Slave Interface with EasyDMA 1
    pub TWIS0_S: pac::TWIS0_S,

    /// nRF9160 Secure peripheral: I2C compatible Two-Wire Slave Interface with EasyDMA 3
    pub TWIS1_S: pac::TWIS1_S,

    /// nRF9160 Secure peripheral: I2C compatible Two-Wire Slave Interface with EasyDMA 5
    pub TWIS2_S: pac::TWIS2_S,

    /// nRF9160 Secure peripheral: I2C compatible Two-Wire Slave Interface with EasyDMA 7
    pub TWIS3_S: pac::TWIS3_S,

    /// nRF9160 Secure peripheral: UART with EasyDMA 1
    pub UARTE0_S: pac::UARTE0_S,

    /// nRF9160 Secure peripheral: UART with EasyDMA 3
    pub UARTE1_S: pac::UARTE1_S,

    /// nRF9160 Secure peripheral: UART with EasyDMA 5
    pub UARTE2_S: pac::UARTE2_S,

    /// nRF9160 Secure peripheral: UART with EasyDMA 7
    pub UARTE3_S: pac::UARTE3_S,

    /// nRF9160 Secure peripheral: User information configuration registers User information configuration registers
    pub UICR_S: pac::UICR_S,

    /// nRF9160 Secure peripheral: Volatile Memory controller 1
    pub VMC_S: pac::VMC_S,

    /// nRF9160 Secure peripheral: Watchdog Timer 0
    pub WDT_S: pac::WDT_S,
}

impl Board {
    /// Take the peripherals safely
    ///
    /// This method will return an instance of `nRF9160DK` the first time it is
    /// called. It will return only `None` on subsequent calls.
    pub fn take() -> Option<Self> {
        Some(Self::new(CorePeripherals::take()?, Peripherals::take()?))
    }

    /// Steal the peripherals
    ///
    /// This method produces an instance of `nRF9160DK`, regardless of whether
    /// another instance was create previously.
    ///
    /// # Safety
    ///
    /// This method can be used to create multiple instances of `nRF9160DK`. Those
    /// instances can interfere with each other, causing all kinds of unexpected
    /// behavior and circumventing safety guarantees in many ways.
    ///
    /// Always use `nRF9160DK::take`, unless you really know what you're doing.
    pub unsafe fn steal() -> Self {
        Self::new(CorePeripherals::steal(), Peripherals::steal())
    }

    fn new(cp: CorePeripherals, p: Peripherals) -> Self {
        let pins0 = p0::Parts::new(p.P0_NS);

        // The nRF9160-DK features an USB CDC port.
        // It features HWFC but does not have to use it.
        // It can transmit a flexible baudrate of up to 1Mbps.
        let cdc_uart = Uarte::new(
            p.UARTE0_NS,
            uarte::Pins {
                txd: pins0.p0_29.into_push_pull_output(Level::High).degrade(),
                rxd: pins0.p0_28.into_floating_input().degrade(),
                cts: Some(pins0.p0_27.into_floating_input().degrade()),
                rts: Some(pins0.p0_26.into_push_pull_output(Level::High).degrade()),
            },
            UartParity::EXCLUDED,
            UartBaudrate::BAUD115200,
        );

        Board {
            cdc_uart,

            pins: Pins {
                P0_00: pins0.p0_00,
                P0_01: pins0.p0_01,
                // 2-9 are LEDs and buttons
                P0_10: pins0.p0_10,
                P0_11: pins0.p0_11,
                P0_12: pins0.p0_12,
                P0_13: pins0.p0_13,
                P0_14: pins0.p0_14,
                P0_15: pins0.p0_15,
                P0_16: pins0.p0_16,
                P0_17: pins0.p0_17,
                P0_18: pins0.p0_18,
                P0_19: pins0.p0_19,
                P0_20: pins0.p0_20,
                P0_21: pins0.p0_21,
                P0_22: pins0.p0_22,
                P0_23: pins0.p0_23,
                P0_24: pins0.p0_24,
                P0_25: pins0.p0_25,
                // 26-29 are UARTE0
                P0_30: pins0.p0_30,
                P0_31: pins0.p0_31,
            },

            leds: Leds {
                led_1: Led::new(pins0.p0_02.degrade()),
                led_2: Led::new(pins0.p0_03.degrade()),
                led_3: Led::new(pins0.p0_04.degrade()),
                led_4: Led::new(pins0.p0_05.degrade()),
            },

            buttons: Buttons {
                button_1: Button::new(pins0.p0_06.degrade()),
                button_2: Button::new(pins0.p0_07.degrade()),
                switch_1: Button::new(pins0.p0_08.degrade()),
                switch_2: Button::new(pins0.p0_09.degrade()),
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

            // nRF9160 non-secure peripherals
            CLOCK_NS: p.CLOCK_NS,
            DPPIC_NS: p.DPPIC_NS,
            EGU0_NS: p.EGU0_NS,
            EGU1_NS: p.EGU1_NS,
            EGU2_NS: p.EGU2_NS,
            EGU3_NS: p.EGU3_NS,
            EGU4_NS: p.EGU4_NS,
            EGU5_NS: p.EGU5_NS,
            FPU_NS: p.FPU_NS,
            GPIOTE1_NS: p.GPIOTE1_NS,
            I2S_NS: p.I2S_NS,
            IPC_NS: p.IPC_NS,
            KMU_NS: p.KMU_NS,
            NVMC_NS: p.NVMC_NS,
            PDM_NS: p.PDM_NS,
            POWER_NS: p.POWER_NS,
            PWM0_NS: p.PWM0_NS,
            PWM1_NS: p.PWM1_NS,
            PWM2_NS: p.PWM2_NS,
            PWM3_NS: p.PWM3_NS,
            REGULATORS_NS: p.REGULATORS_NS,
            RTC0_NS: p.RTC0_NS,
            RTC1_NS: p.RTC1_NS,
            SAADC_NS: p.SAADC_NS,
            SPIM0_NS: p.SPIM0_NS,
            SPIM1_NS: p.SPIM1_NS,
            SPIM2_NS: p.SPIM2_NS,
            SPIM3_NS: p.SPIM3_NS,
            SPIS0_NS: p.SPIS0_NS,
            SPIS1_NS: p.SPIS1_NS,
            SPIS2_NS: p.SPIS2_NS,
            SPIS3_NS: p.SPIS3_NS,
            TIMER0_NS: p.TIMER0_NS,
            TIMER1_NS: p.TIMER1_NS,
            TIMER2_NS: p.TIMER2_NS,
            TWIM0_NS: p.TWIM0_NS,
            TWIM1_NS: p.TWIM1_NS,
            TWIM2_NS: p.TWIM2_NS,
            TWIM3_NS: p.TWIM3_NS,
            TWIS0_NS: p.TWIS0_NS,
            TWIS1_NS: p.TWIS1_NS,
            TWIS2_NS: p.TWIS2_NS,
            TWIS3_NS: p.TWIS3_NS,
            UARTE1_NS: p.UARTE1_NS,
            UARTE2_NS: p.UARTE2_NS,
            UARTE3_NS: p.UARTE3_NS,
            VMC_NS: p.VMC_NS,
            WDT_NS: p.WDT_NS,
        }
    }
}

/// The nRF9160 pins that are available on the nRF9160DK
#[allow(non_snake_case)]
pub struct Pins {
    pub P0_00: p0::P0_00<Input<Floating>>,
    pub P0_01: p0::P0_01<Input<Floating>>,
    // pub P0_02: p0::P0_02<Input<Floating>>,
    // pub P0_03: p0::P0_03<Input<Floating>>,
    // pub P0_04: p0::P0_04<Input<Floating>>,
    // pub P0_05: p0::P0_05<Input<Floating>>,
    // pub P0_06: p0::P0_06<Input<Floating>>,
    // pub P0_07: p0::P0_07<Input<Floating>>,
    // pub P0_08: p0::P0_08<Input<Floating>>,
    // pub P0_09: p0::P0_09<Input<Floating>>,
    pub P0_10: p0::P0_10<Input<Floating>>,
    pub P0_11: p0::P0_11<Input<Floating>>,
    pub P0_12: p0::P0_12<Input<Floating>>,
    pub P0_13: p0::P0_13<Input<Floating>>,
    pub P0_14: p0::P0_14<Input<Floating>>,
    pub P0_15: p0::P0_15<Input<Floating>>,
    pub P0_16: p0::P0_16<Input<Floating>>,
    pub P0_17: p0::P0_17<Input<Floating>>,
    pub P0_18: p0::P0_18<Input<Floating>>,
    pub P0_19: p0::P0_19<Input<Floating>>,
    pub P0_20: p0::P0_20<Input<Floating>>,
    pub P0_21: p0::P0_21<Input<Floating>>,
    pub P0_22: p0::P0_22<Input<Floating>>,
    pub P0_23: p0::P0_23<Input<Floating>>,
    pub P0_24: p0::P0_24<Input<Floating>>,
    pub P0_25: p0::P0_25<Input<Floating>>,
    // pub P0_26: p0::P0_26<Input<Floating>>,
    // pub P0_27: p0::P0_27<Input<Floating>>,
    // pub P0_28: p0::P0_28<Input<Floating>>,
    // pub P0_29: p0::P0_29<Input<Floating>>,
    pub P0_30: p0::P0_30<Input<Floating>>,
    pub P0_31: p0::P0_31<Input<Floating>>,
}

/// The LEDs on the nRF9160-DK board
pub struct Leds {
    /// nRF9160-DK: LED1, nRF9160: P0.2
    pub led_1: Led,

    /// nRF9160-DK: LED2, nRF9160: P0.3
    pub led_2: Led,

    /// nRF9160-DK: LED3, nRF9160: P0.4
    pub led_3: Led,

    /// nRF9160-DK: LED4, nRF9160: P0.5
    pub led_4: Led,
}

/// An LED on the nRF9160-DK board
pub struct Led(Pin<Output<PushPull>>);

impl Led {
    fn new<Mode>(pin: Pin<Mode>) -> Self {
        Led(pin.into_push_pull_output(Level::Low))
    }

    /// Enable the LED
    pub fn enable(&mut self) {
        self.0.set_high()
    }

    /// Disable the LED
    pub fn disable(&mut self) {
        self.0.set_low()
    }
}

/// The Buttons on the nRF9160-DK board
pub struct Buttons {
    /// nRF9160-DK: Button 1, nRF9160: P0.13
    pub button_1: Button,

    /// nRF9160-DK: Button 2, nRF9160: P0.14
    pub button_2: Button,

    /// nRF9160-DK: Switch 1, nRF9160: P0.15
    pub switch_1: Button,

    /// nRF9160-DK: Switch 2, nRF9160: P0.16
    pub switch_2: Button,
}

/// A Button on the nRF9160-DK board
pub struct Button(Pin<Input<PullUp>>);

impl Button {
    fn new<Mode>(pin: Pin<Mode>) -> Self {
        Button(pin.into_pullup_input())
    }

    pub fn is_active(&self) -> bool {
        self.0.is_low()
    }
}
