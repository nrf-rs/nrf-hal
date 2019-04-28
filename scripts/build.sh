#!/usr/bin/env bash

set -e

cargo build --manifest-path nrf52810-hal/Cargo.toml --target thumbv7em-none-eabi
cargo build --manifest-path nrf52832-hal/Cargo.toml
cargo build --manifest-path nrf52840-hal/Cargo.toml
cargo build --manifest-path boards/adafruit_nrf52pro/Cargo.toml --examples
cargo build --manifest-path boards/adafruit-nrf52-bluefruit-le/Cargo.toml --examples
cargo build --manifest-path boards/nRF52-DK/Cargo.toml --examples
cargo build --manifest-path boards/nRF52840-DK/Cargo.toml --examples
cargo build --manifest-path examples/rtfm-demo/Cargo.toml
cargo build --manifest-path examples/rtfm-demo/Cargo.toml --no-default-features --features="52810" --target thumbv7em-none-eabi
cargo build --manifest-path examples/rtfm-demo/Cargo.toml --no-default-features --features="52840"
cargo build --manifest-path examples/spi-demo/Cargo.toml
cargo build --manifest-path examples/twi-ssd1306/Cargo.toml
cargo build --manifest-path examples/twi-ssd1306/Cargo.toml --no-default-features --features="52840" --target thumbv7em-none-eabi
