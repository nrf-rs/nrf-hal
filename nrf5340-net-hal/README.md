# Hardware Abstration Layer for the Nordic nRF5340

This crate is a Hardware Abstraction Layer (HAL) for the Nordic nRF5340. It
wraps the PAC (`nrf5340-pac`) and provides high level wrappers for the chip's
peripherals.

This crate knows nothing about your PCB layout, or which pins you have assigned
to which functions. The only exception are the examples, which are written to
run on the official nRF5340-DK developer kit.

## Usage

You will require the `thumbv8m.main-none-eabihf` target installed.

```console
$ rustup target add thumbv8m.main-none-eabihf
```
## Licence

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
