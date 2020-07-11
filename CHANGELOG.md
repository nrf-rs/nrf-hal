# Changelog

## Unreleased

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
