# I2C compatible Two-Wire Interface to OLED Display demo - Digital Pins

This demo uses the TWIM (Two-Wire Interface Master) peripheral along with the embedded_graphics library to draw "Hello Rust!" to an SSD1306 OLED screen. Note that you need to set a default feature to get this to compile.

TWI write example using an SSD1306 OLED display:
https://cdn-shop.adafruit.com/datasheets/SSD1306.pdf

After running it you should see the words "Hello Rust!" on the display.

## HW connections
Pin     Connection
P0.26   SCL
P0.27   SDA

This is designed for the nRF52-DK & the nRF52840-DK board:
https://www.nordicsemi.com/Software-and-Tools/Development-Kits/nRF52-DK
https://www.nordicsemi.com/Software-and-Tools/Development-Kits/nRF52840-DK

The TWI device is a 128x64px SSD1306
https://cdn-shop.adafruit.com/datasheets/SSD1306.pdf

## How to run 

If using `probe-run`, see parent folder readme for setup and then just run

```console
$ cargo run
```
