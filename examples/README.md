
## Summary of the Examples

Here follows a brief description of each demo for quick reference. For a more in-depth explanation on how the peripherals work please refer to the device reference manuals linked above, the readme for each demo and the comments in the demo code itself.

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
| [spis-demo](./spis-demo/README.md)                    | Digital Pins      | Serial peripheral interface slave (SPIS) demo                         |
| [twi-ssd1306](./twi-ssd1306/README.md)                | Digital Pins      | I2C compatible Two-Wire Interface with the SSD1306 OLED Display demo  |
| [twim-demo](./twim-demo/README.md)                    | Digital Pins      | I2C compatible Two-Wire Interface Master mode demo                    |
| [twis-demo](./twis-demo/README.md)                    | Digital Pins      | I2C compatible Two-Wire Interface Slave mode demo                     |
| [wdt-demo](./wdt-demo/README.md)                      | Timer             | Watchdog timer demo                                                   |


## Running the Examples

Each demo readme should contain instructions on how to run it. However, the information below describes the technologies used and can be used to troubleshoot your system setup. Run the demos from within their respective project directories. E.g. to run `ccm-demo`, you must be in the `nrf-hal/examples/ccm-demo/` directory.
> Since the demos are stand-alone projects you would **NOT** typically run them with `cargo run --example xyz-demo` like some cargo projects are configured to do.

### Once Off System Setup

Install the cross compilation toolchain to target your device. You would typically pass the target as a parameter to cargo or explicitly set it in your cargo config file. If you get compilation errors about `eh_personality` then you have not set the target correctly. Here is an example of the target for a nRF52840 chip:
```console
$ rustup target add thumbv7em-none-eabihf
```
Install the tools to flash the device.
```console
$ cargo install cargo-embed
```

### For Every Project (optional)

Setup the `Cargo.toml` file to use the correct features. Features allow for conditional compilation which is essential for a library like this that supports multiple different devices. Under the `[features]` section add the following line `default = ["52840"]` for the nRF52840-DK device or whatever other feature is applicable for your device. This is optional but it will allow you to simply call `cargo run` and `cargo build` instead of `cargo run --features 52840` and `cargo build --features 52840` respectively. Note that some demo projects do not have features so this step may not be necessary. If you get a whole bunch of compilation errors or plugins like rust-analyzer are not working then check that you have set the chip features correctly.
