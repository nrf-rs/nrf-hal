# Monotonic demo

This crate defines a minimal [`corex-m-rtic`](https://docs.rs/cortex-m-rtic/1.1.4/rtic/)-app using the [`rtc`](../../nrf-hal-common/src/rtc.rs) or [`timer`](../../nrf-hal-common/src/timer.rs)
for software task scheduling. This example shows how to use the different clocks and how to switch inbetween them.

## How to run

To run the default blinky example
```bash
cargo embed --release
```
To run the example using the `rtc`
```bash
cargo embed --release --example rtc
```
To run the example using the `timer`
```bash
cargo embed --release --example timer
```

