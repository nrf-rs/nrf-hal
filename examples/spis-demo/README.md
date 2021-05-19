# SPI slave demo

The Serial peripheral interface slave (SPIS) demo. This demonstrates the SPIS module, transmitting the current buffer contents while receiving new data. Press the button to zero the buffer.

## How to run 

If using `cargo-embed`, just run

```console
$ cargo embed --release --target=thumbv7em-none-eabihf
```