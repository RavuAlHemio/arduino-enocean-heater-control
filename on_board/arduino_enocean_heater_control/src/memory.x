/* Arduino Due memory layout (SAM3X8E) */

MEMORY {
    /* K = 1024 bytes */
    FLASH : ORIGIN = 0x00080000, LENGTH = 256K
    /* TODO: second flash at 0x000C0000? */
    RAM : ORIGIN = 0x20000000, LENGTH = 64K
    /* TODO: second SRAM at 0x20080000? */
}
