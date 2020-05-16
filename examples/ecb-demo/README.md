# AES electronic codebook mode encryption demo

Chose the microcontroller with one of the following features:
- 51
- 52810
- 52832
- 52840

Also, if using `cargo-embed`, change the `chip` and `protocol` fields in [Embed.toml](Embed.toml).

This demo uses the [rtt-target](https://crates.io/crates/rtt-target) crate for communication.

If using `cargo-embed`, just run

```console
$ cargo embed --release --features=52832
```

Replace `52832` with the correct feature for your microcontroller.
