#!/usr/bin/env bash

# thumbv7em-none-eabihf is the default target if not specified
# (see ../.config/cargo)

set -e

echo Building nrf51-hal...
cargo build --manifest-path nrf51-hal/Cargo.toml --target thumbv6m-none-eabi
echo Building nrf9160-hal...
cargo build --manifest-path nrf9160-hal/Cargo.toml --target thumbv8m.main-none-eabi
echo Building nrf52810-hal...
cargo build --manifest-path nrf52810-hal/Cargo.toml --target thumbv7em-none-eabi
echo Building nrf52832-hal...
cargo build --manifest-path nrf52832-hal/Cargo.toml
echo Building nrf52833-hal...
cargo build --manifest-path nrf52833-hal/Cargo.toml
echo Building nrf52840-hal...
cargo build --manifest-path nrf52840-hal/Cargo.toml

# Build all the example projects.

echo Building examples/ccm-demo...
cargo build --manifest-path examples/ccm-demo/Cargo.toml --features=52810 --target thumbv7em-none-eabi
cargo build --manifest-path examples/ccm-demo/Cargo.toml --features=52832
cargo build --manifest-path examples/ccm-demo/Cargo.toml --features=52833
cargo build --manifest-path examples/ccm-demo/Cargo.toml --features=52840

echo Building examples/comp-demo...
cargo build --manifest-path examples/comp-demo/Cargo.toml

echo Building examples/ecb-demo...
cargo build --manifest-path examples/ecb-demo/Cargo.toml --features=52810 --target thumbv7em-none-eabi
cargo build --manifest-path examples/ecb-demo/Cargo.toml --features=52832
cargo build --manifest-path examples/ecb-demo/Cargo.toml --features=52833
cargo build --manifest-path examples/ecb-demo/Cargo.toml --features=52840
cargo build --manifest-path examples/ecb-demo/Cargo.toml --features=51 --target thumbv6m-none-eabi

echo Building examples/gpiote-demo...
cargo build --manifest-path examples/gpiote-demo/Cargo.toml

echo Building examples/i2s-controller-demo...
cargo build --manifest-path examples/i2s-controller-demo/Cargo.toml

# FIXME: Does not build
#echo Building examples/i2s-peripheral-demo...
#cargo build --manifest-path examples/i2s-peripheral-demo/Cargo.toml

echo Building examples/lpcomp-demo...
cargo build --manifest-path examples/lpcomp-demo/Cargo.toml

echo Building examples/ppi-demo...
cargo build --manifest-path examples/ppi-demo/Cargo.toml --features=51 --target thumbv6m-none-eabi
cargo build --manifest-path examples/ppi-demo/Cargo.toml --features=52810 --target thumbv7em-none-eabi
cargo build --manifest-path examples/ppi-demo/Cargo.toml --features=52832
cargo build --manifest-path examples/ppi-demo/Cargo.toml --features=52833
cargo build --manifest-path examples/ppi-demo/Cargo.toml --features=52840

echo Building examples/pwm-demo...
cargo build --manifest-path examples/pwm-demo/Cargo.toml

echo Building examples/qdec-demo...
cargo build --manifest-path examples/qdec-demo/Cargo.toml

echo Building examples/rtic-demo...
cargo build --manifest-path examples/rtic-demo/Cargo.toml
echo Building examples/rtic-demo...
cargo build --manifest-path examples/rtic-demo/Cargo.toml --no-default-features --features="51" --target thumbv6m-none-eabi
echo Building examples/rtic-demo...
cargo build --manifest-path examples/rtic-demo/Cargo.toml --no-default-features --features="52810" --target thumbv7em-none-eabi
echo Building examples/rtic-demo...
cargo build --manifest-path examples/rtic-demo/Cargo.toml --no-default-features --features="52840"

echo Building examples/spi-demo...
cargo build --manifest-path examples/spi-demo/Cargo.toml

echo Building examples/twi-ssd1306...
cargo build --manifest-path examples/twi-ssd1306/Cargo.toml
echo Building examples/twi-ssd1306...
cargo build --manifest-path examples/twi-ssd1306/Cargo.toml --no-default-features --features="52840" --target thumbv7em-none-eabi

echo Building examples/twim-demo...
cargo build --manifest-path examples/twim-demo/Cargo.toml

echo Building examples/twis-demo...
cargo build --manifest-path examples/twis-demo/Cargo.toml

echo Building examples/wdt-demo...
cargo build --manifest-path examples/wdt-demo/Cargo.toml

echo Checking source code formatting...
cargo +stable fmt -- --check
