use crate::ppi::Task;

// Task Impls
//
// To reproduce, in the pac crate, search
//   `rg 'type TASKS_.*crate::Reg' --type rust`
// Find (regex):
//   `^src/(.*)\.rs:pub type (.*) = .*$`
// Replace (regex):
//   `impl Task for crate::target::$1::$2 { }`
impl Task for crate::target::rng::TASKS_START {}
impl Task for crate::target::rng::TASKS_STOP {}
impl Task for crate::target::timer0::TASKS_START {}
impl Task for crate::target::timer0::TASKS_STOP {}
impl Task for crate::target::timer0::TASKS_COUNT {}
impl Task for crate::target::timer0::TASKS_CLEAR {}
impl Task for crate::target::timer0::TASKS_SHUTDOWN {}
impl Task for crate::target::timer0::TASKS_CAPTURE {}
impl Task for crate::target::uart0::TASKS_STARTRX {}
impl Task for crate::target::uart0::TASKS_STOPRX {}
impl Task for crate::target::uart0::TASKS_STARTTX {}
impl Task for crate::target::uart0::TASKS_STOPTX {}
impl Task for crate::target::uart0::TASKS_SUSPEND {}
impl Task for crate::target::gpiote::TASKS_OUT {}
impl Task for crate::target::clock::TASKS_HFCLKSTART {}
impl Task for crate::target::clock::TASKS_HFCLKSTOP {}
impl Task for crate::target::clock::TASKS_LFCLKSTART {}
impl Task for crate::target::clock::TASKS_LFCLKSTOP {}
impl Task for crate::target::clock::TASKS_CAL {}
impl Task for crate::target::clock::TASKS_CTSTART {}
impl Task for crate::target::clock::TASKS_CTSTOP {}
impl Task for crate::target::power::TASKS_CONSTLAT {}
impl Task for crate::target::power::TASKS_LOWPWR {}
impl Task for crate::target::twi0::TASKS_STARTRX {}
impl Task for crate::target::twi0::TASKS_STARTTX {}
impl Task for crate::target::twi0::TASKS_STOP {}
impl Task for crate::target::twi0::TASKS_SUSPEND {}
impl Task for crate::target::twi0::TASKS_RESUME {}
impl Task for crate::target::ecb::TASKS_STARTECB {}
impl Task for crate::target::ecb::TASKS_STOPECB {}
impl Task for crate::target::wdt::TASKS_START {}
impl Task for crate::target::spis1::TASKS_ACQUIRE {}
impl Task for crate::target::spis1::TASKS_RELEASE {}
impl Task for crate::target::rtc0::TASKS_START {}
impl Task for crate::target::rtc0::TASKS_STOP {}
impl Task for crate::target::rtc0::TASKS_CLEAR {}
impl Task for crate::target::rtc0::TASKS_TRIGOVRFLW {}
impl Task for crate::target::lpcomp::TASKS_START {}
impl Task for crate::target::lpcomp::TASKS_STOP {}
impl Task for crate::target::lpcomp::TASKS_SAMPLE {}
impl Task for crate::target::radio::TASKS_TXEN {}
impl Task for crate::target::radio::TASKS_RXEN {}
impl Task for crate::target::radio::TASKS_START {}
impl Task for crate::target::radio::TASKS_STOP {}
impl Task for crate::target::radio::TASKS_DISABLE {}
impl Task for crate::target::radio::TASKS_RSSISTART {}
impl Task for crate::target::radio::TASKS_RSSISTOP {}
impl Task for crate::target::radio::TASKS_BCSTART {}
impl Task for crate::target::radio::TASKS_BCSTOP {}
impl Task for crate::target::temp::TASKS_START {}
impl Task for crate::target::temp::TASKS_STOP {}
impl Task for crate::target::ccm::TASKS_KSGEN {}
impl Task for crate::target::ccm::TASKS_CRYPT {}
impl Task for crate::target::ccm::TASKS_STOP {}
impl Task for crate::target::adc::TASKS_START {}
impl Task for crate::target::adc::TASKS_STOP {}
impl Task for crate::target::aar::TASKS_START {}
impl Task for crate::target::aar::TASKS_STOP {}
impl Task for crate::target::qdec::TASKS_START {}
impl Task for crate::target::qdec::TASKS_STOP {}
impl Task for crate::target::qdec::TASKS_READCLRACC {}
