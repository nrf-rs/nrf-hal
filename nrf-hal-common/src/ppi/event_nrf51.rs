use crate::ppi::Event;

// Event impls
//
// To reproduce, in the pac crate, search
//   `rg 'type EVENTS_.*crate::Reg' --type rust`
// Find (regex):
//   `^src/(.*)\.rs:pub type (.*) = .*$`
// Replace (regex):
//   `impl Event for crate::target::$1::$2 { }`
impl Event for crate::target::ecb::EVENTS_ENDECB {}
impl Event for crate::target::ecb::EVENTS_ERRORECB {}
impl Event for crate::target::rng::EVENTS_VALRDY {}
impl Event for crate::target::timer0::EVENTS_COMPARE {}
impl Event for crate::target::uart0::EVENTS_CTS {}
impl Event for crate::target::uart0::EVENTS_NCTS {}
impl Event for crate::target::uart0::EVENTS_RXDRDY {}
impl Event for crate::target::uart0::EVENTS_TXDRDY {}
impl Event for crate::target::uart0::EVENTS_ERROR {}
impl Event for crate::target::uart0::EVENTS_RXTO {}
impl Event for crate::target::gpiote::EVENTS_IN {}
impl Event for crate::target::gpiote::EVENTS_PORT {}
impl Event for crate::target::power::EVENTS_POFWARN {}
impl Event for crate::target::clock::EVENTS_HFCLKSTARTED {}
impl Event for crate::target::clock::EVENTS_LFCLKSTARTED {}
impl Event for crate::target::clock::EVENTS_DONE {}
impl Event for crate::target::clock::EVENTS_CTTO {}
impl Event for crate::target::spi0::EVENTS_READY {}
impl Event for crate::target::twi0::EVENTS_STOPPED {}
impl Event for crate::target::twi0::EVENTS_RXDREADY {}
impl Event for crate::target::twi0::EVENTS_TXDSENT {}
impl Event for crate::target::twi0::EVENTS_ERROR {}
impl Event for crate::target::twi0::EVENTS_BB {}
impl Event for crate::target::twi0::EVENTS_SUSPENDED {}
impl Event for crate::target::spis1::EVENTS_END {}
impl Event for crate::target::spis1::EVENTS_ENDRX {}
impl Event for crate::target::spis1::EVENTS_ACQUIRED {}
impl Event for crate::target::rtc0::EVENTS_TICK {}
impl Event for crate::target::rtc0::EVENTS_OVRFLW {}
impl Event for crate::target::rtc0::EVENTS_COMPARE {}
impl Event for crate::target::wdt::EVENTS_TIMEOUT {}
impl Event for crate::target::temp::EVENTS_DATARDY {}
impl Event for crate::target::radio::EVENTS_READY {}
impl Event for crate::target::radio::EVENTS_ADDRESS {}
impl Event for crate::target::radio::EVENTS_PAYLOAD {}
impl Event for crate::target::radio::EVENTS_END {}
impl Event for crate::target::radio::EVENTS_DISABLED {}
impl Event for crate::target::radio::EVENTS_DEVMATCH {}
impl Event for crate::target::radio::EVENTS_DEVMISS {}
impl Event for crate::target::radio::EVENTS_RSSIEND {}
impl Event for crate::target::radio::EVENTS_BCMATCH {}
impl Event for crate::target::lpcomp::EVENTS_READY {}
impl Event for crate::target::lpcomp::EVENTS_DOWN {}
impl Event for crate::target::lpcomp::EVENTS_UP {}
impl Event for crate::target::lpcomp::EVENTS_CROSS {}
impl Event for crate::target::ccm::EVENTS_ENDKSGEN {}
impl Event for crate::target::ccm::EVENTS_ENDCRYPT {}
impl Event for crate::target::ccm::EVENTS_ERROR {}
impl Event for crate::target::aar::EVENTS_END {}
impl Event for crate::target::aar::EVENTS_RESOLVED {}
impl Event for crate::target::aar::EVENTS_NOTRESOLVED {}
impl Event for crate::target::qdec::EVENTS_SAMPLERDY {}
impl Event for crate::target::qdec::EVENTS_REPORTRDY {}
impl Event for crate::target::qdec::EVENTS_ACCOF {}
impl Event for crate::target::adc::EVENTS_END {}
