# Quadrature decoder (QDEC) demo - Sensor Decoding

This peripheral supports buffered decoding of quadrature-encoded sensor signals (typically used for mechanical and optical sensors). The demo shows how to use a rotary encoder to trigger an interrupt and update a variable by the amount of rotation. (nRF52840 + Bourns PEC11R rotary encoder)

## How to run 

If using `cargo-embed`, just run

```console
$ cargo embed --release --target=thumbv7em-none-eabihf
```