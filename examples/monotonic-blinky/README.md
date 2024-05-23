# Monotonic demo

This crate defines a minimal [`corex-m-rtic`](https://docs.rs/cortex-m-rtic/latest/rtic/)-app using the [`rtc`](../../nrf-hal-common/src/rtc.rs) or [`timer`](../../nrf-hal-common/src/timer.rs)
for software task scheduling. This example shows how to use the different clocks and how to switch inbetween them.

## How to run

To run the example using the `rtc`
```bash
cargo embed --release --bin rtc
```
To run the example using the `timer`
```bash
cargo embed --release --bin timer
```

