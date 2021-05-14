/* Linker script for the nRF9160 in Non-secure mode. It assumes you have the
Nordic Secure Partition Manager installed at the bottom of flash and that
the SPM is set to boot a non-secure application from the FLASH origin below. */

MEMORY
{
    /*
     * This is where the Bootloader, Secure Partition Manager or
     * Trusted-Firmware-M lives.
     */
    SECURE_FLASH : ORIGIN = 0x00000000, LENGTH = 256K
    /*
     * This is where your non-secure Rust application lives. Note that SPM must agree this
     * is where your application lives, or it will jump to garbage and crash the CPU.
     */
    FLASH        : ORIGIN = 0x00050000, LENGTH = 768K
    /*
     * This RAM is reserved for the Secure-Mode code located in the `SECURE_FLASH` region.
     */
    SECURE_RAM   : ORIGIN = 0x20000000, LENGTH = 64K
    /*
     * This RAM is available to both the Cortex-M33 and the LTE core (well,
        technically anything between `0x2000_0000` and `0x2001_FFFF` is
        shareable, but we just gave the first 64 KiB to Secure Mode). Shared
        buffers must be placed here.
     */
    SHARED_RAM   : ORIGIN = 0x20010000, LENGTH = 64K
    /*
     * This RAM is available to your non-secure Rust application.
     */
    RAM          : ORIGIN = 0x20020000, LENGTH = 128K
}

SECTIONS
{
    /* This section contains the buffers used by `libnrf_modem` to talk between the Cortex-M33 and the LTE core */
    .shared_ram (NOLOAD) : ALIGN(4)
    {
        . = ALIGN(4);
        *(.shared_ram .shared_ram.*);
        . = ALIGN(4);
    } > SHARED_RAM
}
