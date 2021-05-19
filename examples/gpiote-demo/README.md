# Gpiote demo

The General-Purpose Input Output Tasks and Events (gpiote) module demo targets the nRF52840-DK in particular because of the 4 available hardware buttons on the board itself. The demo shows how you can use the `cortex-m-rtic` crate to easily debounce some buttons without blocking the CPU. GPIO pin state changes fire events which can be used to carry out tasks. This showcases the PPI (programmable peripheral interconnect) system for which there is also a dedicated demo.

## How to run

If using `cargo-embed`, just run

```console
$ cargo embed --release --target=thumbv7em-none-eabihf
```