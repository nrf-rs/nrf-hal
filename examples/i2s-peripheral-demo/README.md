# I2S peripheral demo

The Inter-IC Sound interface (I2S) peripheral mode (aka slave mode) demo. This demonstrates full duplex communication between a controller and peripheral mode I2S peripheral using the EasyDMA capabilities of the chip. This targets the nrf52840 family of devices.

## How to run 

If using `cargo-embed`, just run

```console
$ cargo embed --release --target=thumbv7em-none-eabihf
```