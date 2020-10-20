## Running the Demos

All the demos can be found in the examples folder and are completely independent cargo projects. Run them from within the respective project directories. E.g. to run `ccm-demo`, you must be in the `nrf-hal/examples/ccm-demo/` directory and run `cargo run`.

There are many ways to setup programming and debugging with a device. Here we will describe how to do this on the nRF52840-DK using [probe-rs](https://github.com/probe-rs/probe-rs). 

### Once off system setup

Install the cross compilation toolchain to target the device.
```
rustup target add thumbv7em-none-eabihf
```
Install the tools to program and run on the device. See [probe-rs](https://github.com/probe-rs/probe-rs) for more details on other required dependencies.
```
cargo install probe-run
```

### For every project

Setup your `.cargo/config` file (create one in the project root if it does not exist. E.g., `nrf-hal/examples/ccm-demo/.cargo/config`). This example will call the prope-run executable configured for the nrf52840 chip when you call `cargo run`:
```
[target.thumbv7em-none-eabihf]
runner = "probe-run --chip nRF52840_xxAA"

[build]
target = "thumbv7em-none-eabihf"
```
Setup the `Cargo.toml` file to use the correct features. Features allow for conditional compilation which is essential for a library like this that supports multiple different devices. Under the `[features]` section add the following line `default = ["52840"]` for the nRF52840-DK device. This is optional but it will allow you to simply call `cargo run` and `cargo build` instead of `cargo run --features 52840` and `cargo build --features 52840` respectively. Note that some demo projects do not have features so this step may not be necessary. If you get a whole bunch of compilation errors then check that you have set the default features correctly.

### To run

Plug in your device (on the nRF52840-DK it is the J2 usb port)
`cargo run`
This will flash the device, reset it and send `rprintln!` debug messages from the device to the terminal automatically.

## Summary of the Demos

Here follows a brief description of each demo for quick reference. For a more in-depth explanation on how the peripherals work please refer to the device reference manuals above and the comments in the demo code itself.

### blinky-button-demo (Hello World)

The blinky button demo. This demonstrates a simple hello world blinky application targeted at the ***nrf52832 chip*** which has different pinouts to the nRD52840-DK board. It is a useful exercise to experiment with what you need to change to get the same functionality working on the nrf52840 chip as this project does not use features to support multiple chips. This demo also introduces the cargo-embed tool which is an alternative to probe-run but part of the same family.

### ccm-demo (Encryption)

The cipher block chaining - message authentication code (CCM) mode demo. This demo initialises a text message of the maximum size of 251 bytes and encrypts and decrypts it, measuring the time it takes. It then repeats the process with smaller and smaller chunks of data to demonstrate how long smaller packets take to process.

### comp-demo (Analog Pins)

The comparator peripheral demo. This demo uses the comp peripheral to compare the differential voltages between two pins. If the voltage on Pin 30 is higher than Pin 31 (reference voltage) the built in LED will switch off otherwise it will switch on.

### ecb-demo (Encryption)

The AES electronic codebook mode encryption demo. Blocking 128-bit AES encryption of 16 bytes of data using a 16 byte key. Encryption only, no decryption.

### gpiote-demo (Digital Pins)

The General-Purpose Input Output Tasks and Events module demo. This demo targets the nRF52840-DK in particular because of the 4 available hardware buttons on the board itself. The demo shows how you can use the `cortex-m-rtic` crate to easily debounce some buttons without blocking the CPU. GPIO pin state changes fire events which can be used to carry out tasks. This showcases the PPI (programmable peripheral interconnect) system for which there is also a dedicated demo.

### i2s-controller-demo (Audio)

The Inter-IC Sound interface 'controller mode (aka master mode)' demo. This demo generates Morse code audio signals from text received over UART and plays them back over I2S. Tested with nRF52840-DK and a UDA1334a DAC. 

### i2s-peripheral-demo (Audio)

The Inter-IC Sound interface 'peripheral mode (aka slave mode)' demo. This demonstrates full duplex communication between a controller and peripheral mode I2S peripheral using the EasyDMA capabilities of the chip. 

### lpcomp-demo (Analog Pins)

The low power comparator demo. This demo shows how you would keep the device in low power mode and power it up when an analog voltage on a pin changes with respect to a voltage on a reference pin.

### ppi-demo (Channels)

The programmable peripheral interconnect (PPI) demo. The PPI allows peripherals to interact with each other without having to go through the CPU. Note that you need to choose a default feature in order for this demo to build. See above. This demo uses the Bluetooth RADIO peripheral as an example but does nothing special with Bluetooth itself so this is not the demo to learn about that capability.

### pwm-demo (Digital Pins)

The pulse width modulation demo. This demonstrates various PWM use cases by allowing the user to press buttons to change demo modes. This outputs PWM signals to the built in LEDs on the nRF52840-DK.

### qdec-demo (Sensor Decoding)

The quadrature decoder (QDEC) demo. This peripheral supports buffered decoding of quadrature-encoded sensor signals (typically used for mechanical and optical sensors). The demo shows how to use a rotary encoder to trigger an interrupt and update a variable by the amount of rotation. (nRF52840 + Bourns PEC11R rotary encoder)

### rtic-demo (Concurrency Framework)

The Real-Time Interrupt-driven Concurrency framework demo. Many of the demos in this project use RTIC and demonstrate its capabilities in more detail but this is a bare-bones default template for you to build off. RTIC is not unique to the nRF series but very useful for a program that requires concurrency. Unfortunately, this demo does not appear to use rtt for logging so it crashes when used with probe-run. However, it will work with other debuggers. See other demos for rtt logging setup.

### spi-demo (Digital Pins)

The serial peripheral interface master (SPIM) with EasyDMA demo. Sends some text out on the SPI peripheral and loops it back on itself to demonstrate full duplex direct-memory-access based SPI data transfer. You'll need a resistor to connect the output to the input. 

### twi-ssd1306 (Digital Pins)

I2C compatible Two-Wire Interface with the SSD1306 OLED Display demo. This demo uses the TWIM (Two-Wire Interface Master) peripheral along with the embedded_graphics library to draw "Hello Rust!" to an OLED screen. Note that you need to set a default feature to get this to compile (see "Running the demos" section).

### twim-demo (Digital Pins)

I2C compatible Two-Wire Interface Master mode demo. This demo uses the TWIM peripheral to read and write 8 bytes of data to arbitrary addresses on a device that is connected to the I2C pins p0_30 and p0_31. It demonstrates error handling if the device does not respond properly (or it is not connected).

### twis-demo (Digital Pins)

I2C compatible Two-Wire Interface Slave mode demo. This demo uses the twis peripheral with rtic to listen for I2C signals which are exposed as events. The event handler reads data from the peripheral at the address specified.

### wdt-demo (Timer)

Watchdog timer demo. This demonstrates the how the entire device can be set to automatically reset if certain conditions are not met within a certain period of time. In this case you have to press all 4 buttons at least once within a 5 second period to prevent a reset. If you mash the buttons for a while you will encounter an 'attempt to subtract with overflow' panic at `main.rs:205` which is ultimately cleared by the watchdog timer. This is intended demo behaviour ;)
