# nRF52840 HAL tests

Run `cargo test -p nrf52840-hal-tests` to test the HAL on a nRF52840.

The crate assumes that you'll test the HAL on a nRF52840 Development Kit.
If you wish to use a different development board you'll need to update the flags passed to `probe-run` in `.cargo/config.toml`.

The following wiring is required:

- P0.03 <-> GND
- P0.04 <-> VDD
- P0.28 <-> P0.29
