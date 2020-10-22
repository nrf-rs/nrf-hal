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
| [`nrf51-hal`](./nrf51-hal) | [![docs.rs](https://docs.rs/nrf51-hal/badge.svg)](https://docs.rs/nrf51-hal) | [![crates.io](https://img.shields.io/crates/d/nrf51-hal.svg)](https://crates.io/crates/nrf51-hal) | `thumbv6m-none-eabi` |
| [`nrf52810-hal`](./nrf52810-hal) | [![docs.rs](https://docs.rs/nrf52810-hal/badge.svg)](https://docs.rs/nrf52810-hal) | [![crates.io](https://img.shields.io/crates/d/nrf52810-hal.svg)](https://crates.io/crates/nrf52810-hal) | `thumbv7em-none-eabi` |
| [`nrf52811-hal`](./nrf52811-hal) | [![docs.rs](https://docs.rs/nrf52811-hal/badge.svg)](https://docs.rs/nrf52811-hal) | [![crates.io](https://img.shields.io/crates/d/nrf52811-hal.svg)](https://crates.io/crates/nrf52811-hal) | `thumbv7em-none-eabi` |
| [`nrf52832-hal`](./nrf52832-hal) | [![docs.rs](https://docs.rs/nrf52832-hal/badge.svg)](https://docs.rs/nrf52832-hal) | [![crates.io](https://img.shields.io/crates/d/nrf52832-hal.svg)](https://crates.io/crates/nrf52832-hal) | `thumbv7em-none-eabihf` |
| [`nrf52833-hal`](./nrf52833-hal) | [![docs.rs](https://docs.rs/nrf52833-hal/badge.svg)](https://docs.rs/nrf52833-hal) | [![crates.io](https://img.shields.io/crates/d/nrf52833-hal.svg)](https://crates.io/crates/nrf52833-hal) | `thumbv7em-none-eabihf` |
| [`nrf52840-hal`](./nrf52840-hal) | [![docs.rs](https://docs.rs/nrf52840-hal/badge.svg)](https://docs.rs/nrf52840-hal) | [![crates.io](https://img.shields.io/crates/d/nrf52840-hal.svg)](https://crates.io/crates/nrf52840-hal) | `thumbv7em-none-eabihf` |
| [`nrf9160-hal`](./nrf9160-hal) | [![docs.rs](https://docs.rs/nrf9160-hal/badge.svg)](https://docs.rs/nrf9160-hal) | [![crates.io](https://img.shields.io/crates/d/nrf9160-hal.svg)](https://crates.io/crates/nrf9160-hal) | `thumbv8m.main-none-eabihf` |

## Device Reference Manuals from Nordic

| Device | Product Specification | DK Reference Guide |
|-------|------|-----------|
| [`nRF52810`](https://www.nordicsemi.com/Products/Low-power-short-range-wireless/nRF52810) | [`v1.3`](https://infocenter.nordicsemi.com/pdf/nRF52810_PS_v1.3.pdf) | [`v1.3.1*`](https://infocenter.nordicsemi.com/pdf/nRF52_DK_User_Guide_v1.3.1.pdf) |
| [`nRF52811`](https://www.nordicsemi.com/Products/Low-power-short-range-wireless/nRF52811) | [`v1.0`](https://infocenter.nordicsemi.com/pdf/nRF52811_PS_v1.0.pdf) | [`v1.3.1*`](https://infocenter.nordicsemi.com/pdf/nRF52_DK_User_Guide_v1.3.1.pdf) |
| [`nRF52832`](https://www.nordicsemi.com/Products/Low-power-short-range-wireless/nRF52832) | [`v1.4`](https://infocenter.nordicsemi.com/pdf/nRF52832_PS_v1.4.pdf) | [`v1.3.1*`](https://infocenter.nordicsemi.com/pdf/nRF52_DK_User_Guide_v1.3.1.pdf) |
| [`nRF52833`](https://www.nordicsemi.com/Products/Low-power-short-range-wireless/nRF52833) | [`v1.3`](https://infocenter.nordicsemi.com/pdf/nRF52833_PS_v1.3.pdf) | [`v1.0.1`](http://infocenter.nordicsemi.com/pdf/nRF52833_DK_User_Guide_v1.0.1.pdf) |
| [`nRF52840`](https://www.nordicsemi.com/Products/Low-power-short-range-wireless/nRF52840) | [`v1.1`](https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.1.pdf) | [`v1.2`](https://infocenter.nordicsemi.com/pdf/nRF52840_DK_User_Guide_v1.2.pdf) |
| [`nRF9160`](https://www.nordicsemi.com/Products/Low-power-cellular-IoT/nRF9160) | [`v2.0`](https://infocenter.nordicsemi.com/pdf/nRF9160_PS_v2.0.pdf) | [`v0.9.3`](https://infocenter.nordicsemi.com/pdf/nRF9160_DK_HW_User_Guide_v0.9.3.pdf) |
\* These devices do not have a seperate developement kit and share the [NRF52 DK](https://www.nordicsemi.com/Software-and-tools/Development-Kits/nRF52-DK)

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

## Summary of the Examples

Here follows a brief description of each demo for quick reference. For a more in-depth explanation on how the peripherals work please refer to the device reference manuals linked above, the readme for each demo and the comments in the demo code itself.

| Demo                                                  | Category          | Description                                                           |
|-------------------------------------------------------|-------------------|-----------------------------------------------------------------------|
| [blinky-button-demo](./blinky-button-demo/README.md)  | Hello World       | Blinky button demo                                                    |
| [ccm-demo](./ccm-demo/README.md)                      | Encryption        | Cipher block chaining - message authentication code (CCM) mode demo   |
| [comp-demo](./comp-demo/README.md)                    | Analog Pins       | Voltage comparator peripheral demo                                    |
| [ecb-demo](./ecb-demo/README.md)                      | Encryption        | AES electronic codebook mode encryption demo                          |
| [gpiote-demo](./gpiote-demo/README.md)                | Digital Pins      | General-Purpose Input Output Tasks and Events module demo             |
| [i2s-controller-demo](./i2s-controller-demo/README.md)| Audio             | Inter-IC Sound interface "controller mode (aka master mode)" demo     |
| [i2s-peripheral-demo](./i2s-peripheral-demo/README.md)| Audio             | Inter-IC Sound interface "peripheral mode (aka slave mode)" demo      |
| [lpcomp-demo](./lpcomp-demo/README.md)                | Analog Pins       | Low power voltage comparator demo                                     |
| [ppi-demo](./ppi-demo/README.md)                      | Channels          | Programmable peripheral interconnect (PPI) demo                       |
| [pwm-demo](./pwm-demo/README.md)                      | Digital Pins      | Pulse width modulation demo                                           |
| [qdec-demo](./qdec-demo/README.md)                    | Sensor Decoding   | Quadrature sensor decoder (QDEC) demo                                 |
| [rtic-demo](./rtic-demo/README.md)                    | Framework         | The Real-Time Interrupt-driven Concurrency framework demo             |
| [spi-demo](./spi-demo/README.md)                      | Digital Pins      | Serial peripheral interface master (SPIM) with EasyDMA demo           |
| [twi-ssd1306](./twi-ssd1306/README.md)                | Digital Pins      | I2C compatible Two-Wire Interface with the SSD1306 OLED Display demo  |
| [twim-demo](./twim-demo/README.md)                    | Digital Pins      | I2C compatible Two-Wire Interface Master mode demo                    |
| [twis-demo](./twis-demo/README.md)                    | Digital Pins      | I2C compatible Two-Wire Interface Slave mode demo                     |
| [wdt-demo](./wdt-demo/README.md)                      | Timer             | Watchdog timer demo                                                   |


## Running the Examples

Each demo readme should contain instructions on how to run it. However, the information below describes the technologies used and can be used to troubleshoot your system setup. Run the demos from within their respective project directories. E.g. to run `ccm-demo`, you must be in the `nrf-hal/examples/ccm-demo/` directory and run `cargo run`.
> Since the demos are stand-alone projects you would **NOT** typically run them with `cargo run --example xyz-demo` like some cargo projects are configured to do.

There are many ways to setup programming and debugging with an embedded device. Here we will describe how to do this on the nRF52840-DK using the [probe-rs](https://probe.rs/) set of tools. 

### Once Off System Setup

Install the cross compilation toolchain to target the device.
```console
$ rustup target add thumbv7em-none-eabihf
```
Install the tools to program and run on the device. See [probe-rs](https://github.com/probe-rs/probe-rs) for more details on other required dependencies.
```console
$ cargo install probe-run
```

### For Every Project (optional)

Optional if you want to use `cargo run` and `cargo check` without extra args. Setup your `.cargo/config` file (create one in the project root if it does not exist. E.g., `nrf-hal/examples/ccm-demo/.cargo/config`). This example will call the prope-run executable configured for the nrf52840 chip when you call `cargo run`:
```
[target.thumbv7em-none-eabihf]
runner = "probe-run --chip nRF52840_xxAA"

[build]
target = "thumbv7em-none-eabihf"
```
Setup the `Cargo.toml` file to use the correct features. Features allow for conditional compilation which is essential for a library like this that supports multiple different devices. Under the `[features]` section add the following line `default = ["52840"]` for the nRF52840-DK device. This is optional but it will allow you to simply call `cargo run` and `cargo build` instead of `cargo run --features 52840` and `cargo build --features 52840` respectively. Note that some demo projects do not have features so this step may not be necessary. If you get a whole bunch of compilation errors then check that you have set the default features correctly. 

### To Run

Plug in your device (on the nRF52840-DK it is the J2 usb port)
`cargo run`
This will flash the device, reset it and send `rprintln!` debug messages from the device to the terminal automatically.

### An Alternative to probe-run. Try cargo embed

You can also use `cargo embed` instead of the probe-run tool to flash the demos to your device. This tool uses the `Embed.toml` file to configure logging and other device characteristics and can be installed by running `cargo install cargo-embed`. The editing of `Cargo.toml` to set the default features still apply but you can pass them into the `cargo embed` tool instead if you wish. Some of the demo readme files show examples of command line arguments to pass to the tool without any additional configuration.

