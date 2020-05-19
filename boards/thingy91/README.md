# `thingy91-nrf9160-bsp`

Board support crate for the nRF9160 on the Nordic Thingy:91.

https://www.nordicsemi.com/Software-and-tools/Prototyping-platforms/Nordic-Thingy-91

This crate assumes you:

* are using an external ARM Cortex-Debug programmer (e.g. SEGGER J-Link) connected to the 10-pin Cortex-Debug header on the Thingy:91,
* are not attempting to upload firmware over USB using the Thingy:91's built-in `mcuboot` DFU bootloader, and
* have used your ARM Cortex-Debug programmer to load the Nordic Secure Partition Manager, which expects a non-secure application at address 0x0004_0000.

If you want to use the Nordic-supplied `mcuboot` bootloader, your application's `.cargo/config` file is going to need to specify a custom linker script, which contains the correct flash layout.

The Thingy:91 also contains a nRF52840. This crate assumes you are running the default nRF52840 firmware, as supplied out of the box, which acts as a USB to UART adaptor for UART0 (at 115,200 bps) and UART1 (at 1,000,000 bps) on the nRF9160.

This crate is in early development.
