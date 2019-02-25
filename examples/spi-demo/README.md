# spi-demo
SPIM demonstation code.
Connect a resistor between pin 22 and 23 on the demo board to feed MOSI directly back to MISO
If all tests pass all four Led (Led1 to Led4) will light up, in case of error only at least one of the Led will remain turned off.


## HW connections
Pin     Connecton
P0.24   SPIclk
P0.23   MOSI
P0.22   MISO

This is designed for nRF52-DK board:
https://www.nordicsemi.com/Software-and-Tools/Development-Kits/nRF52-DK
