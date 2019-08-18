#![no_main]
#![no_std]

// panic handler
extern crate panic_semihosting;

use cortex_m_semihosting::hprintln;

#[cfg(feature = "52810")]
use nrf52810_hal as hal;

#[cfg(feature = "52832")]
use nrf52832_hal as hal;

#[cfg(feature = "52840")]
use nrf52840_hal as hal;

use hal::gpio::{p0, Level};
use hal::target::{interrupt, TIMER0 as TIM0, UARTE0};
use hal::timer::*;
use hal::{
    uarte::{
        self,
        interrupt_driven::{RXError, UarteDMAPool, UarteDMAPoolNode, UarteRX, UarteTX},
    },
    Uarte,
};

use heapless::{
    consts,
    pool::singleton::Box,
    spsc::{Producer, Queue},
};

// Needed for the write! example code
// use core::{fmt::Write, ops::DerefMut};
// use heapless::pool::singleton::Pool;

use rtfm::app;

const NR_PACKAGES: usize = 10;
const DMA_MEM: usize = core::mem::size_of::<UarteDMAPoolNode>() * NR_PACKAGES;

// Using power-of-2 constants is faster (see the crate heapless for details)
type TXQSize = consts::U4;

#[app(device = crate::hal::target)]
const APP: () = {
    static mut RX: UarteRX<UARTE0, TIM0> = ();
    static mut TX: UarteTX<UARTE0, TXQSize> = ();
    static mut PRODUCER: Producer<'static, Box<UarteDMAPool>, TXQSize> = ();

    #[init(spawn = [])]
    fn init() -> init::LateResources {
        // for the actual DMA buffers
        static mut MEMORY: [u8; DMA_MEM] = [0; DMA_MEM];
        // for the producer/consumer of TX
        static mut TX_RB: Queue<Box<UarteDMAPool>, TXQSize> = Queue(heapless::i::Queue::new());

        hprintln!("init").unwrap();

        let port0 = p0::Parts::new(device.P0);

        let uarte0 = Uarte::new(
            device.UARTE0,
            uarte::Pins {
                // adafruit-nrf52-bluefruit-le, adafruit_nrf52pro, nRF52-DK, nRF52840-DK
                txd: port0.p0_06.into_push_pull_output(Level::High).degrade(),
                rxd: port0.p0_08.into_floating_input().degrade(),
                // Use the following for DWM-1001 dev board
                // txd: port0.p0_05.into_push_pull_output(Level::High).degrade(),
                // rxd: port0.p0_11.into_floating_input().degrade(),
                cts: None,
                rts: None,
            },
            uarte::Parity::EXCLUDED,
            uarte::Baudrate::BAUD115200,
        );

        let timer = Timer::new(device.TIMER0);
        let (txp, txc) = TX_RB.split();
        let (rx, tx) = uarte0.into_interrupt_driven(txc, timer, MEMORY);

        init::LateResources {
            RX: rx,
            TX: tx,
            PRODUCER: txp,
        }
    }

    // // we can get Box<P> us being now the owner
    #[task(capacity = 2, resources = [PRODUCER])]
    fn printer(data: Box<UarteDMAPool>) {
        // enqueue a test message
        // let mut node = UarteDMAPool::alloc().unwrap().init(UarteDMAPoolNode::new());
        // write!(&mut node, "test").unwrap(); // Using the write! trait
        // node.write_slice(&[95, 95, 95, 95]); // Using raw slice writing
        // resources.PRODUCER.enqueue(node).unwrap();
        // hprintln!("{:?}", &data).unwrap();

        // Echo the buffer back without any changes or copying
        resources.PRODUCER.enqueue(data).unwrap();
        rtfm::pend(interrupt::UARTE0_UART0);
    }

    #[task]
    fn rx_error(err: RXError) {
        hprintln!("rx_error {:?}", err).unwrap();
    }

    #[interrupt(priority = 2, resources = [RX])]
    fn TIMER0() {
        resources.RX.process_timeout_interrupt();
    }

    #[interrupt(priority = 2, resources = [RX, TX], spawn = [printer, rx_error])]
    fn UARTE0_UART0() {
        // probe RX
        match resources.RX.process_interrupt() {
            Ok(Some(b)) => {
                // delegate data to printer
                match spawn.printer(b) {
                    Err(_) => spawn.rx_error(RXError::OOM).unwrap(),
                    _ => (),
                };
            }
            Ok(None) => (), // no
            Err(err) => spawn.rx_error(err).unwrap(),
        }

        resources.TX.process_interrupt();
    }

    extern "C" {
        fn SWI1_EGU1();
        fn SWI2_EGU2();
    }
};
