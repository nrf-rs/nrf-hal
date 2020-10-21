# The blinky button demo

This hello world example turns on LED 1 when you press Button 1 on the nrf52-dk (PCA10040).
> You will have to change the pin numbers if you use a nRF52840-DK device.

## Set up with `cargo-embed`

Install `cargo-embed` if you don't have it already:

```console
$ cargo install cargo-embed
```

Then just `cd` to the example folder and run

```console
$ cargo embed --target thumbv7em-none-eabihf
```

