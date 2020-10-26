# Changelog

## Unreleased

### New Features

- Derive more traits for `gpio::{Level, Port}` ([#185]).
- COMP module ([#189]).
- QDEC module ([#188]).
- LPCOMP module ([#195]).
- TWIS module ([#196]).
- PWM module ([#200]).
- I2S module ([#201] [#209] [#225] [#237]).
- SPIS module ([#226] [#236]).
- Add support for the nRF52811 ([#227]).
- Add PPI channel group tasks ([#212]).
- Add PPI endpoints for timers ([#239]).

### Enhancements

- Improve SAADC docs ([#218]).
- Update Embed.toml of all examples to new defaults ([#229]).
- Make `ConfigurablePpi` and subtrait of `Ppi` ([#244]).

### Fixes

- Refuse to build nRF52+ HALs for thumbv6m targets ([#203]).
- Refuse to build `nrf52810-hal` for hard-float targets, and `nrf51-hal` for thumbv7+ targets
  ([#206]).
- Set the correct Port in GPIOTE ([#217] [#248]).
- Correct TWIM port initialization for P1 pins ([#221]).
- Fix race condition in RTC event handling ([#243]).

### Breaking Changes

- Remove `Spi::read` in favor of `transfer_split_uneven` ([#190]).
- Seal the `timer::Instance` trait ([#214]).
- Make GPIOs start in a `Disconnected` state instead of `Input<Floating>` ([#220] [#245]).
- 🦭 all `Instance` traits ([#255]).
- 🦭 PPI traits ([#259]).
- Various TWIM fixes and improvements - removes automatic transfer splitting ([#242]).
- Remove typestate from RTC to make it easier to use ([#252]).
- Mark error enums as `#[non_exhaustive]` ([#260]).

### Internal Improvements

- Utilize [`cargo-xtask`] to simplify CI and the release process ([#207] [#210]).
- Add `conf()` utility function to reduce code duplication in GPIO ([#250]).

[#185]: https://github.com/nrf-rs/nrf-hal/pull/185
[#188]: https://github.com/nrf-rs/nrf-hal/pull/188
[#189]: https://github.com/nrf-rs/nrf-hal/pull/189
[#195]: https://github.com/nrf-rs/nrf-hal/pull/195
[#196]: https://github.com/nrf-rs/nrf-hal/pull/196
[#200]: https://github.com/nrf-rs/nrf-hal/pull/200
[#201]: https://github.com/nrf-rs/nrf-hal/pull/201
[#203]: https://github.com/nrf-rs/nrf-hal/pull/203
[#209]: https://github.com/nrf-rs/nrf-hal/pull/209
[#190]: https://github.com/nrf-rs/nrf-hal/pull/190
[#206]: https://github.com/nrf-rs/nrf-hal/pull/206
[#207]: https://github.com/nrf-rs/nrf-hal/pull/207
[#210]: https://github.com/nrf-rs/nrf-hal/pull/210
[#212]: https://github.com/nrf-rs/nrf-hal/pull/212
[#217]: https://github.com/nrf-rs/nrf-hal/pull/217
[#214]: https://github.com/nrf-rs/nrf-hal/pull/214
[#218]: https://github.com/nrf-rs/nrf-hal/pull/218
[#220]: https://github.com/nrf-rs/nrf-hal/pull/220
[#221]: https://github.com/nrf-rs/nrf-hal/pull/221
[#225]: https://github.com/nrf-rs/nrf-hal/pull/225
[#226]: https://github.com/nrf-rs/nrf-hal/pull/226
[#227]: https://github.com/nrf-rs/nrf-hal/pull/227
[#229]: https://github.com/nrf-rs/nrf-hal/pull/229
[#236]: https://github.com/nrf-rs/nrf-hal/pull/236
[#237]: https://github.com/nrf-rs/nrf-hal/pull/237
[#239]: https://github.com/nrf-rs/nrf-hal/pull/239
[#242]: https://github.com/nrf-rs/nrf-hal/pull/242
[#243]: https://github.com/nrf-rs/nrf-hal/pull/243
[#244]: https://github.com/nrf-rs/nrf-hal/pull/244
[#245]: https://github.com/nrf-rs/nrf-hal/pull/245
[#248]: https://github.com/nrf-rs/nrf-hal/pull/248
[#250]: https://github.com/nrf-rs/nrf-hal/pull/250
[#252]: https://github.com/nrf-rs/nrf-hal/pull/252
[#255]: https://github.com/nrf-rs/nrf-hal/pull/255
[#259]: https://github.com/nrf-rs/nrf-hal/pull/259
[#260]: https://github.com/nrf-rs/nrf-hal/pull/260
[`cargo-xtask`]: https://github.com/matklad/cargo-xtask

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
