use crate::ppi::Task;

// Task Impls
//
// To reproduce, in the pac crate, search
//   `rg 'type TASKS_.*crate::Reg' --type rust --no-heading --no-line-number`
// Find (regex):
//   `^src/(.*)\.rs:pub type (.*) = .*$`
// Replace (regex):
//   `impl Task for crate::pac::$1::$2 { }`
// Find (regex):
//   `^impl Task for crate::pac::spim0::(.*)$`
// Replace (regex):
//   `impl Task for crate::pac::spim1::$1`
impl Task for crate::pac::spim1::TASKS_START {}
impl Task for crate::pac::spim1::TASKS_STOP {}
impl Task for crate::pac::spim1::TASKS_SUSPEND {}
impl Task for crate::pac::spim1::TASKS_RESUME {}
impl Task for crate::pac::rng::TASKS_START {}
impl Task for crate::pac::rng::TASKS_STOP {}
impl Task for crate::pac::timer0::TASKS_START {}
impl Task for crate::pac::timer0::TASKS_STOP {}
impl Task for crate::pac::timer0::TASKS_COUNT {}
impl Task for crate::pac::timer0::TASKS_CLEAR {}
impl Task for crate::pac::timer0::TASKS_SHUTDOWN {}
impl Task for crate::pac::timer0::TASKS_CAPTURE {}
impl Task for crate::pac::spis1::TASKS_ACQUIRE {}
impl Task for crate::pac::spis1::TASKS_RELEASE {}
impl Task for crate::pac::uart0::TASKS_STARTRX {}
impl Task for crate::pac::uart0::TASKS_STOPRX {}
impl Task for crate::pac::uart0::TASKS_STARTTX {}
impl Task for crate::pac::uart0::TASKS_STOPTX {}
impl Task for crate::pac::uart0::TASKS_SUSPEND {}
impl Task for crate::pac::gpiote::TASKS_OUT {}
impl Task for crate::pac::gpiote::TASKS_SET {}
impl Task for crate::pac::gpiote::TASKS_CLR {}
impl Task for crate::pac::clock::TASKS_HFCLKSTART {}
impl Task for crate::pac::clock::TASKS_HFCLKSTOP {}
impl Task for crate::pac::clock::TASKS_LFCLKSTART {}
impl Task for crate::pac::clock::TASKS_LFCLKSTOP {}
impl Task for crate::pac::clock::TASKS_CAL {}
impl Task for crate::pac::clock::TASKS_CTSTART {}
impl Task for crate::pac::clock::TASKS_CTSTOP {}
impl Task for crate::pac::power::TASKS_CONSTLAT {}
impl Task for crate::pac::power::TASKS_LOWPWR {}
impl Task for crate::pac::egu0::TASKS_TRIGGER {}
impl Task for crate::pac::twim0::TASKS_STARTRX {}
impl Task for crate::pac::twim0::TASKS_STARTTX {}
impl Task for crate::pac::twim0::TASKS_STOP {}
impl Task for crate::pac::twim0::TASKS_SUSPEND {}
impl Task for crate::pac::twim0::TASKS_RESUME {}
impl Task for crate::pac::pdm::TASKS_START {}
impl Task for crate::pac::pdm::TASKS_STOP {}
impl Task for crate::pac::ecb::TASKS_STARTECB {}
impl Task for crate::pac::ecb::TASKS_STOPECB {}
impl Task for crate::pac::twi0::TASKS_STARTRX {}
impl Task for crate::pac::twi0::TASKS_STARTTX {}
impl Task for crate::pac::twi0::TASKS_STOP {}
impl Task for crate::pac::twi0::TASKS_SUSPEND {}
impl Task for crate::pac::twi0::TASKS_RESUME {}
impl Task for crate::pac::wdt::TASKS_START {}
impl Task for crate::pac::rtc0::TASKS_START {}
impl Task for crate::pac::rtc0::TASKS_STOP {}
impl Task for crate::pac::rtc0::TASKS_CLEAR {}
impl Task for crate::pac::rtc0::TASKS_TRIGOVRFLW {}
impl Task for crate::pac::radio::TASKS_TXEN {}
impl Task for crate::pac::radio::TASKS_RXEN {}
impl Task for crate::pac::radio::TASKS_START {}
impl Task for crate::pac::radio::TASKS_STOP {}
impl Task for crate::pac::radio::TASKS_DISABLE {}
impl Task for crate::pac::radio::TASKS_RSSISTART {}
impl Task for crate::pac::radio::TASKS_RSSISTOP {}
impl Task for crate::pac::radio::TASKS_BCSTART {}
impl Task for crate::pac::radio::TASKS_BCSTOP {}
impl Task for crate::pac::temp::TASKS_START {}
impl Task for crate::pac::temp::TASKS_STOP {}
impl Task for crate::pac::ccm::TASKS_KSGEN {}
impl Task for crate::pac::ccm::TASKS_CRYPT {}
impl Task for crate::pac::ccm::TASKS_STOP {}
impl Task for crate::pac::ccm::TASKS_RATEOVERRIDE {}
impl Task for crate::pac::uarte0::TASKS_STARTRX {}
impl Task for crate::pac::uarte0::TASKS_STOPRX {}
impl Task for crate::pac::uarte0::TASKS_STARTTX {}
impl Task for crate::pac::uarte0::TASKS_STOPTX {}
impl Task for crate::pac::uarte0::TASKS_FLUSHRX {}
impl Task for crate::pac::twis0::TASKS_STOP {}
impl Task for crate::pac::twis0::TASKS_SUSPEND {}
impl Task for crate::pac::twis0::TASKS_RESUME {}
impl Task for crate::pac::twis0::TASKS_PREPARERX {}
impl Task for crate::pac::twis0::TASKS_PREPARETX {}
impl Task for crate::pac::aar::TASKS_START {}
impl Task for crate::pac::aar::TASKS_STOP {}
impl Task for crate::pac::comp::TASKS_START {}
impl Task for crate::pac::comp::TASKS_STOP {}
impl Task for crate::pac::comp::TASKS_SAMPLE {}
impl Task for crate::pac::qdec::TASKS_START {}
impl Task for crate::pac::qdec::TASKS_STOP {}
impl Task for crate::pac::qdec::TASKS_READCLRACC {}
impl Task for crate::pac::qdec::TASKS_RDCLRACC {}
impl Task for crate::pac::qdec::TASKS_RDCLRDBL {}
impl Task for crate::pac::saadc::TASKS_START {}
impl Task for crate::pac::saadc::TASKS_SAMPLE {}
impl Task for crate::pac::saadc::TASKS_STOP {}
impl Task for crate::pac::saadc::TASKS_CALIBRATEOFFSET {}
impl Task for crate::pac::pwm0::TASKS_STOP {}
impl Task for crate::pac::pwm0::TASKS_SEQSTART {}
impl Task for crate::pac::pwm0::TASKS_NEXTSTEP {}
