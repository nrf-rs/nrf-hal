# UART EasyDMA Demo

This example echos commands, denoted by a new line, sent to the UARTE0 interface and over RTT (Real-Time Transfer) I/O protocol for debug purposes.

This was designed for the nRF52840-DK (PCA10056):
https://www.nordicsemi.com/Software-and-Tools/Development-Kits/nRF52840-DK

## HW connections
Pin     Connecton
P0.06   TX
P0.08   RX

## Set up with `cargo-embed`

Install `cargo-embed` if you don't have it already:

```console
$ cargo install cargo-embed
```

Then just `cd` to the example folder and run

```console
$ cargo embed --target thumbv7em-none-eabihf
```

