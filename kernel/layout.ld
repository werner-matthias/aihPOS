ENTRY(kernel_start)

SECTIONS
{
    . = 0;
    .text : 
    {
	KEEP(*(.except_table))
        .kernel_main = .;
	KEEP(*(.text.kernel_start))
        *(.text*)
    }
    __text_end = .;
     .data : ALIGN(4096)
    {
      __data_start = .;
	*(.data);
	*(.rodata*)
    }
    __data_end = .;
    /* Zum Start wird das bss-Segment für den provisorischen Stack genutzt */
    .bss : ALIGN(4096)
    {
       __bss_start = .;
       . += 16384;
       __kernel_stack = .; /* 16 k provisorischer Stack sollten erst mal reichen */
    }
  /DISCARD/ :
  {
    *(.ARM.exidx*)
    *(.ARM.extab*)
    *(.note.gnu.build-id*)
    *(.debug*)
  }
}

