use crate::ppi::Event;

// Event impls
//
// To reproduce, in the pac crate, search
//   `rg 'type EVENTS_.*crate::Reg' --type rust`
// Find (regex):
//   `^src/(.*)\.rs:pub type (.*) = .*$`
// Replace (regex):
//   `impl Event for crate::pac::$1::$2 { }`
impl Event for crate::pac::ecb::EVENTS_ENDECB {}
impl Event for crate::pac::ecb::EVENTS_ERRORECB {}
impl Event for crate::pac::rng::EVENTS_VALRDY {}
impl Event for crate::pac::timer0::EVENTS_COMPARE {}
impl Event for crate::pac::uart0::EVENTS_CTS {}
impl Event for crate::pac::uart0::EVENTS_NCTS {}
impl Event for crate::pac::uart0::EVENTS_RXDRDY {}
impl Event for crate::pac::uart0::EVENTS_TXDRDY {}
impl Event for crate::pac::uart0::EVENTS_ERROR {}
impl Event for crate::pac::uart0::EVENTS_RXTO {}
impl Event for crate::pac::gpiote::EVENTS_IN {}
impl Event for crate::pac::gpiote::EVENTS_PORT {}
impl Event for crate::pac::power::EVENTS_POFWARN {}
impl Event for crate::pac::clock::EVENTS_HFCLKSTARTED {}
impl Event for crate::pac::clock::EVENTS_LFCLKSTARTED {}
impl Event for crate::pac::clock::EVENTS_DONE {}
impl Event for crate::pac::clock::EVENTS_CTTO {}
impl Event for crate::pac::spi0::EVENTS_READY {}
impl Event for crate::pac::twi0::EVENTS_STOPPED {}
impl Event for crate::pac::twi0::EVENTS_RXDREADY {}
impl Event for crate::pac::twi0::EVENTS_TXDSENT {}
impl Event for crate::pac::twi0::EVENTS_ERROR {}
impl Event for crate::pac::twi0::EVENTS_BB {}
impl Event for crate::pac::twi0::EVENTS_SUSPENDED {}
impl Event for crate::pac::spis1::EVENTS_END {}
impl Event for crate::pac::spis1::EVENTS_ENDRX {}
impl Event for crate::pac::spis1::EVENTS_ACQUIRED {}
impl Event for crate::pac::rtc0::EVENTS_TICK {}
impl Event for crate::pac::rtc0::EVENTS_OVRFLW {}
impl Event for crate::pac::rtc0::EVENTS_COMPARE {}
impl Event for crate::pac::wdt::EVENTS_TIMEOUT {}
impl Event for crate::pac::temp::EVENTS_DATARDY {}
impl Event for crate::pac::radio::EVENTS_READY {}
impl Event for crate::pac::radio::EVENTS_ADDRESS {}
impl Event for crate::pac::radio::EVENTS_PAYLOAD {}
impl Event for crate::pac::radio::EVENTS_END {}
impl Event for crate::pac::radio::EVENTS_DISABLED {}
impl Event for crate::pac::radio::EVENTS_DEVMATCH {}
impl Event for crate::pac::radio::EVENTS_DEVMISS {}
impl Event for crate::pac::radio::EVENTS_RSSIEND {}
impl Event for crate::pac::radio::EVENTS_BCMATCH {}
impl Event for crate::pac::lpcomp::EVENTS_READY {}
impl Event for crate::pac::lpcomp::EVENTS_DOWN {}
impl Event for crate::pac::lpcomp::EVENTS_UP {}
impl Event for crate::pac::lpcomp::EVENTS_CROSS {}
impl Event for crate::pac::ccm::EVENTS_ENDKSGEN {}
impl Event for crate::pac::ccm::EVENTS_ENDCRYPT {}
impl Event for crate::pac::ccm::EVENTS_ERROR {}
impl Event for crate::pac::aar::EVENTS_END {}
impl Event for crate::pac::aar::EVENTS_RESOLVED {}
impl Event for crate::pac::aar::EVENTS_NOTRESOLVED {}
impl Event for crate::pac::qdec::EVENTS_SAMPLERDY {}
impl Event for crate::pac::qdec::EVENTS_REPORTRDY {}
impl Event for crate::pac::qdec::EVENTS_ACCOF {}
impl Event for crate::pac::adc::EVENTS_END {}
