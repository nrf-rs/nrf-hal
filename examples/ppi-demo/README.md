# PPI demo

The Programmable Peripheral Interconnect (PPI) allows peripherals to interact with each other without having to go through the CPU. Note that you need to choose a chip feature in order for this demo to build. See above. This demo uses the Bluetooth RADIO peripheral as an example but does nothing special with Bluetooth itself so this is not the demo to learn about that capability.

## How to run 

Choose the microcontroller with one of the following features:
- 51
- 52810
- 52811
- 52832
- 52840

Also, if using `cargo-embed`, change the `chip` and `protocol` fields in [Embed.toml](Embed.toml).

This demo uses the [rtt-target](https://crates.io/crates/rtt-target) crate for communication.

If using `cargo-embed`, just run

```console
$ cargo embed --release --features=52832 --target=thumbv7em-none-eabihf
```

Replace `52832` and `thumbv7em-none-eabihf` with the correct feature and target for your microcontroller.
