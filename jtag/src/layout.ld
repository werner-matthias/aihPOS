ENTRY(kernel_main)
 
SECTIONS
{
    /* Starts at LOADER_ADDR. */
    . = 0x8000;
    .text :
    {
        KEEP(*(.text.kernel_main))
        *(.text)
    }	
    .bss :
    {
        bss = .;
        *(.bss)
    }
}

