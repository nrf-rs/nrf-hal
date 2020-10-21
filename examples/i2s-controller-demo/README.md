# Inter-IC Sound interface "controller mode (aka master mode)" - Digital Pins

This demo generates Morse code audio signals from text received over UART and plays them back over I2S. Tested with nRF52840-DK and a UDA1334a DAC. 

## How to run 

If using `cargo-embed`, just run

```console
$ cargo embed --release --target=thumbv7em-none-eabihf
```