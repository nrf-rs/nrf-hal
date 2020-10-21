# The comparator peripheral demo - Analog Pins

 This demo uses the comp peripheral to compare the differential voltages between two pins. If the voltage on Pin 30 is higher than Pin 31 (reference voltage) the built in LED will switch off otherwise it will switch on. The demo uses `nrf52840-hal` but this can be swapped out with an alternative if required.

## How to run

If using `cargo-embed`, just run

```console
$ cargo embed --release --target=thumbv7em-none-eabihf
```