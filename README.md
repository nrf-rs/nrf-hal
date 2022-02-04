# `nrf-hal`

> [HAL] for the nRF51, nRF52 and nRF91 families of microcontrollers

[HAL]: https://crates.io/crates/embedded-hal

![CI](https://github.com/nrf-rs/nrf-hal/workflows/CI/badge.svg)

Please refer to the [changelog] to see what changed in the last releases.

[changelog]: ./CHANGELOG.md

## Crates

Every nRF chip has its own crate, listed below:

| Crate | Docs | crates.io | target |
|-------|------|-----------|--------|
| `nrf51-hal` | [![docs.rs](https://docs.rs/nrf51-hal/badge.svg)](https://docs.rs/nrf51-hal) | [![crates.io](https://img.shields.io/crates/d/nrf51-hal.svg)](https://crates.io/crates/nrf51-hal) | `thumbv6m-none-eabi` |
| `nrf52810-hal` | [![docs.rs](https://docs.rs/nrf52810-hal/badge.svg)](https://docs.rs/nrf52810-hal) | [![crates.io](https://img.shields.io/crates/d/nrf52810-hal.svg)](https://crates.io/crates/nrf52810-hal) | `thumbv7em-none-eabi` |
| `nrf52811-hal` | [![docs.rs](https://docs.rs/nrf52811-hal/badge.svg)](https://docs.rs/nrf52811-hal) | [![crates.io](https://img.shields.io/crates/d/nrf52811-hal.svg)](https://crates.io/crates/nrf52811-hal) | `thumbv7em-none-eabi` |
| `nrf52832-hal` | [![docs.rs](https://docs.rs/nrf52832-hal/badge.svg)](https://docs.rs/nrf52832-hal) | [![crates.io](https://img.shields.io/crates/d/nrf52832-hal.svg)](https://crates.io/crates/nrf52832-hal) | `thumbv7em-none-eabihf` |
| `nrf52833-hal` | [![docs.rs](https://docs.rs/nrf52833-hal/badge.svg)](https://docs.rs/nrf52833-hal) | [![crates.io](https://img.shields.io/crates/d/nrf52833-hal.svg)](https://crates.io/crates/nrf52833-hal) | `thumbv7em-none-eabihf` |
| `nrf52840-hal` | [![docs.rs](https://docs.rs/nrf52840-hal/badge.svg)](https://docs.rs/nrf52840-hal) | [![crates.io](https://img.shields.io/crates/d/nrf52840-hal.svg)](https://crates.io/crates/nrf52840-hal) | `thumbv7em-none-eabihf` |
| `nrf9160-hal` | [![docs.rs](https://docs.rs/nrf9160-hal/badge.svg)](https://docs.rs/nrf9160-hal) | [![crates.io](https://img.shields.io/crates/d/nrf9160-hal.svg)](https://crates.io/crates/nrf9160-hal) | `thumbv8m.main-none-eabihf` |

## Device Reference Manuals from Nordic

| Device | Product Specification | DK Reference Guide |
|-------|------|-----------|
| [`nRF52810`](https://www.nordicsemi.com/Products/Low-power-short-range-wireless/nRF52810) | [`v1.3`](https://infocenter.nordicsemi.com/pdf/nRF52810_PS_v1.3.pdf) | [`v1.3.1*`](https://infocenter.nordicsemi.com/pdf/nRF52_DK_User_Guide_v1.3.1.pdf) |
| [`nRF52811`](https://www.nordicsemi.com/Products/Low-power-short-range-wireless/nRF52811) | [`v1.0`](https://infocenter.nordicsemi.com/pdf/nRF52811_PS_v1.0.pdf) | [`v1.3.1*`](https://infocenter.nordicsemi.com/pdf/nRF52_DK_User_Guide_v1.3.1.pdf) |
| [`nRF52832`](https://www.nordicsemi.com/Products/Low-power-short-range-wireless/nRF52832) | [`v1.4`](https://infocenter.nordicsemi.com/pdf/nRF52832_PS_v1.4.pdf) | [`v1.3.1*`](https://infocenter.nordicsemi.com/pdf/nRF52_DK_User_Guide_v1.3.1.pdf) |
| [`nRF52833`](https://www.nordicsemi.com/Products/Low-power-short-range-wireless/nRF52833) | [`v1.3`](https://infocenter.nordicsemi.com/pdf/nRF52833_PS_v1.3.pdf) | [`v1.0.1`](http://infocenter.nordicsemi.com/pdf/nRF52833_DK_User_Guide_v1.0.1.pdf) |
| [`nRF52840`](https://www.nordicsemi.com/Products/Low-power-short-range-wireless/nRF52840) | [`v1.1`](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.1.pdf) | [`v1.2`](https://infocenter.nordicsemi.com/pdf/nRF52840_DK_User_Guide_v1.2.pdf) |
| [`nRF9160`](https://www.nordicsemi.com/Products/Low-power-cellular-IoT/nRF9160) | [`v2.0`](https://infocenter.nordicsemi.com/pdf/nRF9160_PS_v2.0.pdf) | [`v0.9.3`](https://infocenter.nordicsemi.com/pdf/nRF9160_DK_HW_User_Guide_v0.9.3.pdf) |

\* These devices do not have a separate development kit and share the [NRF52 DK](https://www.nordicsemi.com/Software-and-tools/Development-Kits/nRF52-DK)

## Development

Be sure to copy and edit the `Cargo.example.toml` file to `Cargo.toml`. The file will require editing dependent on the target you wish to work with and contains some further
instructions. Similarly, check out the `.vscode/settings.json` file when used in the context of Visual Studio Code. By default, all of these files are configured to work
with the nRF52840 target.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
