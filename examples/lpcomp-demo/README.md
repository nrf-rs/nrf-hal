# Low power voltage comparator demo - Analog Pins

This demo shows how you would keep the device in low power mode and power it up when an analog voltage on a pin changes with respect to a voltage on a reference pin. This targets the nrf52840 family of devices.

## How to run 

If using `cargo-embed`, just run

```console
$ cargo embed --release --target=thumbv7em-none-eabihf
```