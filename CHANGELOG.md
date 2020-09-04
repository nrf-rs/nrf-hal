# Changelog

## Unreleased

### New Features

- Derive more traits for `gpio::{Level, Port}` ([#185]).
- COMP module ([#189]).
- QDEC module ([#188]).
- LPCOMP module ([#195]).
- TWIS module ([#196]).
- PWM module ([#200]).
- I2S module ([#201]).

### Fixes

- Refuse to build nRF52+ HALs for thumbv6m targets ([#203]).
- Refuse to build `nrf52810-hal` for hard-float targets, and `nrf51-hal` for thumbv7+ targets
  ([#206]).

### Breaking Changes

- Remove `Spi::read` ([#190]).

[#185]: https://github.com/nrf-rs/nrf-hal/pull/185
[#188]: https://github.com/nrf-rs/nrf-hal/pull/188
[#189]: https://github.com/nrf-rs/nrf-hal/pull/189
[#195]: https://github.com/nrf-rs/nrf-hal/pull/195
[#196]: https://github.com/nrf-rs/nrf-hal/pull/196
[#200]: https://github.com/nrf-rs/nrf-hal/pull/200
[#201]: https://github.com/nrf-rs/nrf-hal/pull/201
[#203]: https://github.com/nrf-rs/nrf-hal/pull/203
[#190]: https://github.com/nrf-rs/nrf-hal/pull/190
[#206]: https://github.com/nrf-rs/nrf-hal/pull/206

## [0.11.1]

### New Features

- Add support for the Watchdog Timer peripheral ([#175]).
- Support VDD source for the ADC ([#181]).

### Fixes

- Renamed RTFM examples to RTIC examples ([#183]).
- Updated comment style ([#180]).

### Breaking Changes

None

[#175]: https://github.com/nrf-rs/nrf-hal/pull/175
[#180]: https://github.com/nrf-rs/nrf-hal/pull/180
[#181]: https://github.com/nrf-rs/nrf-hal/pull/181
[#183]: https://github.com/nrf-rs/nrf-hal/pull/183

## [0.11.0]

### New Features

- Add support for Nordic nRF52833 ([#148]).
- Add a driver for the AES-ECB peripheral ([#145]).
- Add a driver for the AES-CCM peripheral ([#154]).
- Add PPI support and example ([#162]).
- Add methods for task clear and trigger overflow to the RTC ([#168]).
- Add a driver for the GPIOTE peripheral ([#167]).

### Fixes

- Fix incorrect logic in `transfer_split_uneven` ([#159]).
- Twim: Implicitly copy buffer into RAM if needed when using embedded hal traits ([#165]).
- Fix Twim hangs on address NACK ([#166]).

### Breaking Changes

- Made GPIO pin fields private and reduced their memory footprint ([#155]).
- Stop reexporting the PAC under `target` ([#172]).

[#148]: https://github.com/nrf-rs/nrf-hal/pull/148
[#145]: https://github.com/nrf-rs/nrf-hal/pull/145
[#154]: https://github.com/nrf-rs/nrf-hal/pull/154
[#155]: https://github.com/nrf-rs/nrf-hal/pull/155
[#159]: https://github.com/nrf-rs/nrf-hal/pull/159
[#162]: https://github.com/nrf-rs/nrf-hal/pull/162
[#165]: https://github.com/nrf-rs/nrf-hal/pull/165
[#166]: https://github.com/nrf-rs/nrf-hal/pull/166
[#168]: https://github.com/nrf-rs/nrf-hal/pull/168
[#167]: https://github.com/nrf-rs/nrf-hal/pull/167
[#172]: https://github.com/nrf-rs/nrf-hal/pull/172

[0.11.0]: https://github.com/nrf-rs/nrf-hal/releases/tag/v0.11.0
[0.11.1]: https://github.com/nrf-rs/nrf-hal/releases/tag/v0.11.1
