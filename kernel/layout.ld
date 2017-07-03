ENTRY(kernel_start)

SECTIONS
{
    .text : 
    {
	. = 0x0;
	KEEP(*(.except_table))
        .kernel_main = .;
	KEEP(*(.text.kernel_start))
        *(.text*)
    }
    __text_end = .;
     .data : ALIGN(128)
    {
	*(.data);
	*(.rodata*)
    }
    __data_end = .;
    /* Zum Start wird der Speicher für den provisorischen Stack genutzt,
       später für das den gemeinsamen Datenbereich von Kernel und Prozessen */
    .shared :  ALIGN(4096)
    {
        __shared_begin = .;
	. = __shared_begin + 16384;
    }
    __shared_end  = .;
    __kernel_stack = .; /* 16 k provisorischer Stack sollten erst mal reichen */
    .pagetables :  ALIGN(16384)
    {
        __page_directory = .;
    }
    . = . + 16384;
    .bss : ALIGN(4096)
    {
       __bss_start = .;
    }
  /DISCARD/ :
  {
    *(.ARM.exidx*)
    *(.ARM.extab*)
    *(.note.gnu.build-id*)
    *(.debug*)
  }
}

