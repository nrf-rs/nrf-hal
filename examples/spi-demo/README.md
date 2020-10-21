# Serial peripheral interface master (SPIM) with EasyDMA demo - Audio

Sends some text out on the SPI peripheral and loops it back on itself to demonstrate full duplex direct-memory-access based SPI data transfer. You'll need a resistor to connect the output to the input. Connect a resistor between pin 22 and 23 on the demo board to feed MOSI directly back to MISO. If all tests pass all four Led (Led1 to Led4) will light up, in case of error only at least one of the Led will remain turned off.

## HW connections

Pin     Connection
P0.24   SPIclk
P0.23   MOSI
P0.22   MISO

This is designed for nRF52-DK board:
https://www.nordicsemi.com/Software-and-Tools/Development-Kits/nRF52-DK
