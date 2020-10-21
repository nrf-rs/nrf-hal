# I2C compatible Two-Wire Interface Slave mode demo - Digital Pins

This demo uses the twis peripheral with rtic to listen for I2C signals which are exposed as events. The event handler reads data from the peripheral at the address specified.

## How to run 

If using `cargo-embed`, just run

```console
$ cargo embed --release --target=thumbv7em-none-eabihf
```