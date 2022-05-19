# Changelog

## Unreleased

- Fixed the nvmc erase procedure on nRF91 & nRF53 ([#387])

## [0.15.0]

### New Features

- Implement `MultiwriteNorFlash` for nRF52 boards that support it ([#373]).
- Enable GPIOTE module for nRF9160 ([#376]).

### Enhancements

- Unified how pins are are returned in `free` calls ([#372]).
- Improvements to the NVMC driver ([#374]).
- Updated `embedded-dma`, `embedded-storage`, and PACs ([#379]).

[#372]: https://github.com/nrf-rs/nrf-hal/pull/372
[#373]: https://github.com/nrf-rs/nrf-hal/pull/373
[#374]: https://github.com/nrf-rs/nrf-hal/pull/374
[#376]: https://github.com/nrf-rs/nrf-hal/pull/376
[#379]: https://github.com/nrf-rs/nrf-hal/pull/379

## [0.14.1]

### New Features

- Add `From` impl to degrade pins more easily ([#364]).
- Support the nRF5340 Application Core with the new `nrf5340-app-hal` crate ([#366]).

### Fixes

- Fix panic in `ieee802154` module introduced in 0.14.0 ([#369]).

[#364]: https://github.com/nrf-rs/nrf-hal/pull/364
[#366]: https://github.com/nrf-rs/nrf-hal/pull/366
[#369]: https://github.com/nrf-rs/nrf-hal/pull/369

## [0.14.0]

### New Features

- Implement `embedded_hal::serial` traits for UARTE ([#343]).

### Enhancements

- Update PACs and other dependencies ([#357]).

### Fixes

- IEEE 802.15.4: automatically disable radio after transmission ([#356]).

[#343]: https://github.com/nrf-rs/nrf-hal/pull/343
[#356]: https://github.com/nrf-rs/nrf-hal/pull/356
[#357]: https://github.com/nrf-rs/nrf-hal/pull/357

## [0.13.0]

### New Features

- USB support ([#295]).
- Added `Pwm::{swap_output_pin, clear_output_pin}` to allow for more flexible PWM pin management ([#335]).
- Added an API for the NVMC peripheral ([#337]).

### Enhancements

- `[breaking change]` Update `rand_core` and `cortex-m` dependencies ([#332]).
- Make the deprecated SPI peripheral available on all nRF52 chips ([#344]).

### Fixes

- Fix TWIS transfer `is_done()` always returns true ([#329]).
- Fix mistake in SPIS `Transfer` `is_done` to borrow `inner` ([#330]).
- Fix I2S frequency mapping ([#333]).
- `[breaking change]` Make `Pwm::set_output_pin` take the pin by-value to fix its soundness ([#335]).

[#295]: https://github.com/nrf-rs/nrf-hal/pull/295
[#329]: https://github.com/nrf-rs/nrf-hal/pull/329
[#330]: https://github.com/nrf-rs/nrf-hal/pull/330
[#332]: https://github.com/nrf-rs/nrf-hal/pull/332
[#333]: https://github.com/nrf-rs/nrf-hal/pull/333
[#335]: https://github.com/nrf-rs/nrf-hal/pull/335
[#337]: https://github.com/nrf-rs/nrf-hal/pull/337
[#344]: https://github.com/nrf-rs/nrf-hal/pull/344

## [0.12.2]

### New Features

- Enable PWM for the nRF9160 and nRF52832 ([#311] [#318]).

### Enhancements

- Add a testsuite for the HAL ([#291]).
- Document that `ieee802154::Radio::recv_timeout` writes the received data to `packet` ([#307]).
- Update nRF9160 HAL with latest memory map ([#321]).
- Add a simple UART example ([#317]).
- Add readme documentation for demos ([#246]).
- Link `README.md` into all sub-crates so they show up on crates.io ([#322]).
- Enhance the RTC example with an interrupt ([#324]).

### Fixes

- Fix spelling errors and RTIC name ([#308]).
- `ieee802154`: mark `start_recv` as unsafe ([#312]).
- Fix PWM EasyDMA max length ([#313]).
- Fix EasyDMA max size ([#315]).
- Work around erratum when enabling UARTE on nRF9160 ([#319]).

[#246]: https://github.com/nrf-rs/nrf-hal/pull/246
[#291]: https://github.com/nrf-rs/nrf-hal/pull/291
[#307]: https://github.com/nrf-rs/nrf-hal/pull/307
[#308]: https://github.com/nrf-rs/nrf-hal/pull/308
[#311]: https://github.com/nrf-rs/nrf-hal/pull/311
[#312]: https://github.com/nrf-rs/nrf-hal/pull/312
[#313]: https://github.com/nrf-rs/nrf-hal/pull/313
[#315]: https://github.com/nrf-rs/nrf-hal/pull/315
[#317]: https://github.com/nrf-rs/nrf-hal/pull/317
[#318]: https://github.com/nrf-rs/nrf-hal/pull/318
[#319]: https://github.com/nrf-rs/nrf-hal/pull/319
[#321]: https://github.com/nrf-rs/nrf-hal/pull/321
[#322]: https://github.com/nrf-rs/nrf-hal/pull/322
[#324]: https://github.com/nrf-rs/nrf-hal/pull/324

## [0.12.1]

### New Features

- nRF9160: Add support for TWIM1-3 ([#273]).
- nRF9160: Add support for WDT ([#283]).
- PPI: Add `clear_fork_task_endpoint` ([#282]).
- Refactor Pin Selection, add `Pin::from_psel_bits` and `Pin::psel_bits` ([#285]).
- SAADC: Support internal `vddhdiv5` channel ([#297]).
- Add an IEEE 802.15.4 radio API ([#143] [#299]).

### Enhancements

- Explain what "sealing" a trait means ([#271]).
- Update `cfg-if` to 1.0 ([#286]).

### Fixes

- Fix TWIM pin selection for nRF52833 ([#274]).
- Return correct error code in UARTE `start_read` ([#280]).
- Fix en-/disabling GPIOTE interrupts for channels ([#278]).
- UARTE: Check rx buf against `EASY_DMA_SIZE` ([#284]).
- SAADC: Clear `events_calibratedone` before calibration ([#298]).

[#143]: https://github.com/nrf-rs/nrf-hal/pull/143
[#271]: https://github.com/nrf-rs/nrf-hal/pull/271
[#273]: https://github.com/nrf-rs/nrf-hal/pull/273
[#274]: https://github.com/nrf-rs/nrf-hal/pull/274
[#278]: https://github.com/nrf-rs/nrf-hal/pull/278
[#280]: https://github.com/nrf-rs/nrf-hal/pull/280
[#282]: https://github.com/nrf-rs/nrf-hal/pull/282
[#283]: https://github.com/nrf-rs/nrf-hal/pull/283
[#284]: https://github.com/nrf-rs/nrf-hal/pull/284
[#285]: https://github.com/nrf-rs/nrf-hal/pull/285
[#286]: https://github.com/nrf-rs/nrf-hal/pull/286
[#297]: https://github.com/nrf-rs/nrf-hal/pull/297
[#298]: https://github.com/nrf-rs/nrf-hal/pull/298
[#299]: https://github.com/nrf-rs/nrf-hal/pull/299

## [0.12.0]

### New Features

- Derive more traits for `gpio::{Level, Port}` ([#185]).
- COMP module ([#189]).
- QDEC module ([#188]).
- LPCOMP module ([#195]).
- TWIS module ([#196] [#230]).
- PWM module ([#200] [#231]).
- I2S module ([#201] [#209] [#225] [#237]).
- SPIS module ([#226] [#236]).
- Add support for the nRF52811 ([#227]).
- Add PPI channel group tasks ([#212]).
- Add PPI endpoints for timers ([#239]).
- Allow disabling and reenabling the TWIM instance ([#266]).

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
- Make GPIOs start in a `Disconnected` state instead of `Input<Floating>` ([#220] [#245]).
- Seal¹ the `timer::Instance` trait ([#214]).
- Seal¹ all `Instance` traits ([#255]).
- Seal¹ PPI traits ([#259]).
- Various TWIM fixes and improvements - removes automatic transfer splitting ([#242]).
- Remove typestate from RTC to make it easier to use ([#252]).
- Also return owned `Pins` from `Usart::free()` ([#261]).

¹ _A trait can be sealed by making a private trait a supertrait. That way, no
downstream crates can implement it (since they can't name the supertrait).
This is just to make sure the trait isn't implemented by types that shouldn't
implement it._

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
[#230]: https://github.com/nrf-rs/nrf-hal/pull/230
[#231]: https://github.com/nrf-rs/nrf-hal/pull/231
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
[#261]: https://github.com/nrf-rs/nrf-hal/pull/261
[#266]: https://github.com/nrf-rs/nrf-hal/pull/266
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
[0.12.0]: https://github.com/nrf-rs/nrf-hal/releases/tag/v0.12.0
[0.12.1]: https://github.com/nrf-rs/nrf-hal/releases/tag/v0.12.1
[0.12.2]: https://github.com/nrf-rs/nrf-hal/releases/tag/v0.12.2
[0.13.0]: https://github.com/nrf-rs/nrf-hal/releases/tag/v0.13.0
[0.14.0]: https://github.com/nrf-rs/nrf-hal/releases/tag/v0.14.0
[0.14.1]: https://github.com/nrf-rs/nrf-hal/releases/tag/v0.14.1
[0.15.0]: https://github.com/nrf-rs/nrf-hal/releases/tag/v0.15.0
