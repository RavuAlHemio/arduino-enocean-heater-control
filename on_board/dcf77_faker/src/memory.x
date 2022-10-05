/* Arduino Due memory layout (SAM3X8E) */

MEMORY {
    /* K = 1024 bytes */

    /* the following is FLASH0 only */
    /* FLASH : ORIGIN = 0x00080000, LENGTH = 256K */

    /* FLASH0 and FLASH1 are contiguous */
    FLASH : ORIGIN = 0x00080000, LENGTH = 512K

    /* the following is SRAM0 only */
    /* RAM : ORIGIN = 0x20000000, LENGTH = 64K */

    /* 0x20000000 is repeated at 0x20070000, which can be used to make SRAM0+SRAM1 contiguous */
    RAM : ORIGIN = 0x20070000, LENGTH = 96K
}
