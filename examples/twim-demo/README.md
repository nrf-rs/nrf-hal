# I2C compatible Two-Wire Interface Master mode demo - Digital Pins

This demo uses the TWIM peripheral to read and write 8 bytes of data to arbitrary addresses on a device that is connected to the I2C pins p0_30 and p0_31. It demonstrates error handling if the device does not respond properly (or it is not connected).

## How to run 

If using `cargo-embed`, just run

```console
$ cargo embed --release --target=thumbv7em-none-eabihf
```