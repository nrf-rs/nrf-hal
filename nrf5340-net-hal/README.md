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

## Secure vs Non-Secure

This HAL is designed to run in non-secure mode, as should most of your
application code. You will therefore need a 'bootloader' which starts in secure
mode, moves the required peripherals into 'non-secure' world, and then jumps to
your application.

We have succesfully used Nordic's [Secure Partition Manager](https://github.com/nrfconnect/sdk-nrf/tree/v1.7/samples/spm)
from nRF SDK v1.7. SPM v1.7 is configured to expect your application at address
`0x0005_0000` and so that is what `memory.x` must specify as the start of Flash.

_Note:_ Other versions of SPM might expect a different start address -
especially those compiled as a child image of another application (like
`at_sample`)! You can see the start address on boot-up:

```
SPM: NS image at 0x50000
```

This tells you SPM is looking for a non-secure (NS) image at `0x0005_0000`.

To build SPM, run:

```console
$ west init -m https://github.com/nrfconnect/sdk-nrf --mr v1.5.1 ncs
$ cd ncs
$ west update # This takes *ages*
$ cd nrf/examples/spm
$ west build --board=nrf5340dk_nrf5340_cpuapp
$ west flash
```

West is a Python tool supplied by Nordic for building the nRF Connect SDK. See
[Nordic's website](https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.5.1/nrf/gs_installing.html)
for more details.

Your nRF5340-DK will now have SPM installed between `0x0000_0000` and
`0x0004_FFFF`. Flashing your application at `0x0005_0000` should not affect SPM,
provided you do not select *erase entire chip* or somesuch!

If you want to change the flash address, supply your own `memory.x` file in your
application crate (or your Board Support Crate) and write a `build.rs` file that
copies your `memory.x` over the top of this one.

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
