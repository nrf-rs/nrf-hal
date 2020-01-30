# `nrf52-hal`

> [HAL] for the nRF52 family of microcontrollers

[HAL]: https://crates.io/crates/embedded-hal

## Documentation

* [`nrf52810-hal`](https://docs.rs/nrf52810-hal)
* [`nrf52832-hal`](https://docs.rs/nrf52832-hal)
* [`nrf52840-hal`](https://docs.rs/nrf52840-hal)

## Getting Started

### Prerequisites

#### rustc with Cortex-M FPU support
- https://www.rust-lang.org/tools/install
- `rustup target add thumbv7em-none-eabihf`

#### nRF Command Line Tools
- https://www.nordicsemi.com/Software-and-tools/Development-Tools/nRF-Command-Line-Tools/Download#infotabs

#### GNU Embedded Toolchain for ARM
- https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads

### Building blinky

To build blinky for nRF52840 DK you have to move to the relevant library:

`cd boards/nRF52840-DK/`

and compile the example:

`cargo build --examples`

This generates the ELF file, which has to be converted to .hex to flash the chip:

```
cd ../../target/thumbv7em-none-eabihf/debug/examples/
arm-none-eabi-objcopy -O ihex blinky blinky.hex
```

Finally, connect the DK and upload the firmware:

```
nrfjprog -e
nrfjprog --program blinky.hex
nrfjprog -r
```

## Resources on the nRF52 devices

- [nRF52840 Reference Manual](http://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.0.pdf)
- [nRF52832 Reference Manual](http://infocenter.nordicsemi.com/pdf/nRF52832_PS_v1.4.pdf)
- [nRF52810 Reference Manual](http://infocenter.nordicsemi.com/pdf/nRF52810_PS_v1.2.pdf)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
