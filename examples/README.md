# Examples

This folder contains a set of independent demo projects that can be used to discover functionality supported by this library and how to use it.


# Summary of the Demos

Here follows a brief description of each demo for quick reference. For a more in-depth explanation on how the peripherals work please refer to the device reference manuals linked in the project root, the readme for each demo and the comments in the demo code itself.

| Demo                                                  | Category          | Description                                                           |
|-------------------------------------------------------|-------------------|-----------------------------------------------------------------------|
| [blinky-button-demo](./blinky-button-demo/README.md)  | Hello World       | Blinky button demo                                                    |
| [ccm-demo](./ccm-demo/README.md)                      | Encryption        | Cipher block chaining - message authentication code (CCM) mode demo   |
| [comp-demo](./comp-demo/README.md)                    | Analog Pins       | Voltage comparator peripheral demo                                    |
| [ecb-demo](./ecb-demo/README.md)                      | Encryption        | AES electronic codebook mode encryption demo                          |
| [gpiote-demo](./gpiote-demo/README.md)                | Digital Pins      | General-Purpose Input Output Tasks and Events module demo             |
| [i2s-controller-demo](./i2s-controller-demo/README.md)| Audio             | Inter-IC Sound interface "controller mode (aka master mode)" demo     |
| [i2s-peripheral-demo](./i2s-peripheral-demo/README.md)| Audio             | Inter-IC Sound interface "peripheral mode (aka slave mode)" demo      |
| [lpcomp-demo](./lpcomp-demo/README.md)                | Analog Pins       | Low power voltage comparator demo                                     |
| [ppi-demo](./ppi-demo/README.md)                      | Channels          | Programmable peripheral interconnect (PPI) demo                       |
| [pwm-demo](./pwm-demo/README.md)                      | Digital Pins      | Pulse width modulation demo                                           |
| [qdec-demo](./qdec-demo/README.md)                    | Sensor Decoding   | Quadrature sensor decoder (QDEC) demo                                 |
| [rtic-demo](./rtic-demo/README.md)                    | Framework         | The Real-Time Interrupt-driven Concurrency framework demo             |
| [spi-demo](./spi-demo/README.md)                      | Digital Pins      | Serial peripheral interface master (SPIM) with EasyDMA demo           |
| [twi-ssd1306](./twi-ssd1306/README.md)                | Digital Pins      | I2C compatible Two-Wire Interface with the SSD1306 OLED Display demo  |
| [twim-demo](./twim-demo/README.md)                    | Digital Pins      | I2C compatible Two-Wire Interface Master mode demo                    |
| [twis-demo](./twis-demo/README.md)                    | Digital Pins      | I2C compatible Two-Wire Interface Slave mode demo                     |
| [wdt-demo](./wdt-demo/README.md)                      | Timer             | Watchdog timer demo                                                   |


# Running the Demos

Each demo readme should contain instructions on how to run it. However, the information below describes the technologies used and can be used to troubleshoot your system setup. Run the demos from within their respective project directories. E.g. to run `ccm-demo`, you must be in the `nrf-hal/examples/ccm-demo/` directory and run `cargo run`.
> Since the demos are stand-alone projects you would **NOT** typically run them with `cargo run --example xyz-demo` like some cargo projects are configured to do.

There are many ways to setup programming and debugging with an embedded device. Here we will describe how to do this on the nRF52840-DK using the [probe-rs](https://probe.rs/) set of tools. 

## Once off system setup

Install the cross compilation toolchain to target the device.
```console
$ rustup target add thumbv7em-none-eabihf
```
Install the tools to program and run on the device. See [probe-rs](https://github.com/probe-rs/probe-rs) for more details on other required dependencies.
```console
$ cargo install probe-run
```

## For every project (optional)

Optional if you want to use `cargo run` and `cargo check` without extra args. Setup your `.cargo/config` file (create one in the project root if it does not exist. E.g., `nrf-hal/examples/ccm-demo/.cargo/config`). This example will call the prope-run executable configured for the nrf52840 chip when you call `cargo run`:
```
[target.thumbv7em-none-eabihf]
runner = "probe-run --chip nRF52840_xxAA"

[build]
target = "thumbv7em-none-eabihf"
```
Setup the `Cargo.toml` file to use the correct features. Features allow for conditional compilation which is essential for a library like this that supports multiple different devices. Under the `[features]` section add the following line `default = ["52840"]` for the nRF52840-DK device. This is optional but it will allow you to simply call `cargo run` and `cargo build` instead of `cargo run --features 52840` and `cargo build --features 52840` respectively. Note that some demo projects do not have features so this step may not be necessary. If you get a whole bunch of compilation errors then check that you have set the default features correctly. 
> Setting the default features in `Cargo.toml` as well as settings in `.cargo/config`, while not absolutely necessary, will help you use cargo without having to pass long command line arguments to it. However, it will also benefit tools like rust-analyzer (which runs `cargo check`) although you are free to configure these tools manually too.

## To run

Plug in your device (on the nRF52840-DK it is the J2 usb port)
`cargo run`
This will flash the device, reset it and send `rprintln!` debug messages from the device to the terminal automatically.

## An alternative to probe-run. Try cargo embed

You can also use `cargo embed` instead of the probe-run tool to flash the demos to your device. This tool uses the `Embed.toml` file to configure logging and other device characteristics and can be installed by running `cargo install cargo-embed`. The editing of `Cargo.toml` to set the default features still apply but you can pass them into the `cargo embed` tool instead if you wish. Some of the demo readme files show examples of command line arguments to pass to the tool without any additional configuration.
