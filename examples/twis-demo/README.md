# TWIS demo

I2C compatible Two-Wire Interface Slave mode (TWIS) demo. This demo uses the TWIS peripheral with RTIC (Real-Time Interrupt-driven Concurrency) to listen for I2C signals which are exposed as events. The event handler reads data from the peripheral at the address specified.

## How to run 

If using `cargo-embed`, just run

```console
$ cargo embed --release --target=thumbv7em-none-eabihf
```