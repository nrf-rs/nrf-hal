# `adafruit-nrf52-bluefruit-le`

**WORK IN PROGRESS!**

Board support for the Adafruit NRF52 Bluefruit LE
https://www.adafruit.com/product/3406

Follow instructions to update the bootloader (step 3 on
https://learn.adafruit.com/bluefruit-nrf52-feather-learning-guide/arduino-bsp-setup)
to bring the SoftDevice up-to-date.

Memory layout in linker.x is set up for SoftDevice S132 6.1.1. May
require changes for other revisions.

# Examples
There's a simple blinky example which writes to the USB serial console
and flashes the red/blue LEDs as it does so.

# Flashing the firmware
I use adafruit-nrfutil (cf the above Learning Guide URL for
installation), as I don't have access to a hardware debugger. nrfutil
requires intel hex input files to generate its firmware package, but
LLVM's objcopy (used by cargo binutils) doesn't support that format,
so you'll need the GNU binutils collection for it.

## Steps:

1) Generate ELF:
`% cargo build -p adafruit-nrf52-bluefruit-le --example blinky --release`
2) Generate Intel hex:
`% arm-none-eabi-objcopy -O ihex target/thumbv7em-none-eabihf/release/examples/blinky blinky.hex`
3) Generate zip firmware:
`% adafruit-nrfutil dfu genpkg --dev-type 0x0052 --sd-req 0x00b7 --application blinky.hex blinky.zip`
4) Upload firmware:
`% adafruit-nrfutil dfu serial -pkg blinky.zip -p $SERIALPORT -b 115200 --singlebank`

# TODO

Bluetooth support. (Pull in s136 stuff? myNewt?). SPI. I²C.
