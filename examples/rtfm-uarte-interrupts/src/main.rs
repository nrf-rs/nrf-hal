#![no_main]
#![no_std]

// panic handler
extern crate panic_semihosting;

use cortex_m_semihosting::hprintln;

use nrf52832_hal as hal;

use hal::gpio::{p0, Level};
use hal::target::{interrupt, TIMER0 as TIM0, UARTE0};
use hal::timer::*;
use hal::{uarte, Uarte};
use hal::{RXError, UARTEDMAPool, UARTEDMAPoolNode, UarteRX, UarteTX};

use heapless::{
    consts::U3,
    pool::singleton::Box,
    spsc::{Producer, Queue},
};

use rtfm::app;

const NR_PACKAGES: usize = 10;
const DMA_MEM: usize = core::mem::size_of::<UARTEDMAPoolNode>() * NR_PACKAGES;
type TXQSize = U3;

#[app(device = crate::hal::target)]
const APP: () = {
    static mut RX: UarteRX<UARTE0, TIM0> = ();
    static mut TX: UarteTX<UARTE0, TXQSize> = ();
    static mut PRODUCER: Producer<'static, Box<UARTEDMAPool>, TXQSize> = ();

    #[init(spawn = [])]
    fn init() -> init::LateResources {
        // for the actual DMA buffers
        static mut MEMORY: [u8; DMA_MEM] = [0; DMA_MEM];
        // for the producer/consumer of TX
        static mut TX_RB: Queue<Box<UARTEDMAPool>, TXQSize> = Queue(heapless::i::Queue::new());

        hprintln!("init").unwrap();

        let port0 = p0::Parts::new(device.P0);

        //adafruit nrf52 le
        let uarte0 = Uarte::new(
            device.UARTE0,
            uarte::Pins {
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
        let (rx, tx) = uarte0.split(txc, timer, MEMORY);

        init::LateResources {
            RX: rx,
            TX: tx,
            PRODUCER: txp,
        }
    }

    // // we can get Box<P> us being now the owner
    #[task(capacity = 2, resources = [PRODUCER])]
    fn printer(data: Box<UARTEDMAPool>) {
        // enqueue a test message
        // let mut node = UARTEDMAPoolNode::new();
        // node.write(&[95, 95, 95, 95]);
        // let b = UARTEDMAPool::alloc()
        //     .unwrap()
        //     .init(node);
        // resources.PRODUCER.enqueue(b).unwrap();
        // hprintln!("{:?}", &data).unwrap();

        // just do the buffer dance without copying
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
