/* Linker script for the nRF5340 in Non-secure mode. It assumes you have the
Nordic Secure Partition Manager installed at the bottom of flash and that
the SPM is set to boot a non-secure application from the FLASH origin below. */

MEMORY
{
    /*
     * This is where the Bootloader, Secure Partition Manager or
     * Trusted-Firmware-M lives.
     */
    /*
    SECURE_FLASH : ORIGIN = 0x00000000, LENGTH = 256K
     * This is where your non-secure Rust application lives. Note that SPM must agree this
     * is where your application lives, or it will jump to garbage and crash the CPU.
     SIG          : ORIGIN =  0x00050000, LENGTH = 1K
     */
      FLASH        : ORIGIN = 0x00050000, LENGTH = 767K 
      /* FLASH        : ORIGIN = 0x00000000, LENGTH = 767K  */
    /*
     * This RAM is reserved for the Secure-Mode code located in the `SECURE_FLASH` region.
     */
     SECURE_RAM   : ORIGIN = 0x20000000, LENGTH = 64K
     /*
     * This RAM is available to your non-secure Rust application.
     */
    RAM          : ORIGIN = 0x20020000, LENGTH = 128K
}
