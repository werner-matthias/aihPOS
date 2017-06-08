#![no_std]
#![no_main]
#![feature(asm,                    // Assembler in Funktionen...
           //global_asm,           // ... und außerhalb
           //abi_unadjusted,         //  
           lang_items,             // Funktionen interne Funktionen ersetzen (panic)
           naked_functions,        // Funktionen ohne Prolog/Epilog
           linkage,                // Angaben zum Linktyp (z.B. Sichtbarkeit)
           const_fn,               // const Funktionen (für Constructoren)
           i128_type,              // 128-Bit-Typen
           repr_align,             // Alignment
           attr_literals,          // Literale in Attributen (nicht nur Strings)
//           use_extern_macros,
           core_intrinsics,        // Nutzung der Intrinsics der Core-Bibliothek
           compiler_builtins_lib,  // Nutzung der Compiler-Buildins-Bibliothek (div, mul, ...)
           //           concat_idents,
           used,                   // Verbot, scheinbar toten Code zu eliminieren
           closure_to_fn_coercion, // Zuweisung von Closures zu Funktionen
          )
  ]

const IRQ_STACK_SIZE: u32 = 2048;
extern {
    static mut __page_directory: [PageDirectoryEntry;4096];
    static __text_end: u32;
    static __data_end: u32;
    static __shared_begin: u32;
    static __shared_end:   u32;
}

extern crate compiler_builtins;
extern crate bit_field;

#[macro_use] mod debug;
#[macro_use] mod hal;
mod panic;
mod sync;
use hal::board::{MemReport,BoardReport,report_board_info,report_memory};
use hal::entry::syscall;
use hal::cpu::mmu::{MMU,PdEntryType,PageDirectoryEntry,PdEntry,DomainAccess,MemoryAccessRight,MemType};


const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[no_mangle]   // Name wird für den Export nicht verändert
#[naked]       // Kein Prolog, da dieser den Stack nutzen würde, den wir noch nicht gesetzt haben
pub extern fn kernel_init() {
    unsafe {
        // Zum Start existiert noch kein Stack. Da alle Variablenoperationen über den Stack
        // laufen, setzen wir einen temporären Stack, der nach dem Textsegment liegt.
        // Danach holen wir uns Informationen über die Speichergröße und setzen die Stacks
        // an das Ende des Speichers. Alle Ausnahmen teilen sich einen Stack (der System-Mode nutzt
        // den User-Mode-Stack).
        // Das Ganze ist natürlich absolut kein safe Rust.
        // Temporärer Stack
        asm!("ldr sp, =__kernel_stack");
        // Nun kann die Größe des Speichers und damit die Adressen für die Stacks bestimmt werden
        determine_irq_stack();
        // Register r5 wird als temporäre Variable benutzt
        asm!("mov r5, r0":::"r5"); 
        determine_svc_stack();
        // Umschalten in den Irq-Mode...
        asm!("cps 0x12");
        // ...und Setzen des Irq-Stacks
        asm!("mov sp, r5");
        // Umschalten in den Irq-Mode...
        asm!("cps 0x12");
        // ...und Setzen des Irq-Stacks
        asm!("mov sp, r5");
        // Umschalten in den Abort-Mode...
        asm!("cps 0x17");
        // ...und Setzen des Abort-Stacks
        asm!("mov sp, r5");
        // Umschalten in den Undefined-Mode...
        asm!("cps 0x1B");
        // ...und Setzen des Undefined-Stacks
        asm!("mov sp, r5");
        // Zurück in den Svc-Mode...
        asm!("cps 0x13");
        // ...und Setzen der "richtigen" Stackadresse
        asm!("mov sp, r0");
    }
    report();
    init_mem();
    test();
}
#[inline(never)]
fn determine_irq_stack() -> u32 {
    let addr = (report_memory(MemReport::ArmSize) - 3) & 0xFFFFFFFC;
    addr
}

#[inline(never)]
fn determine_svc_stack() -> u32 {
    let addr = ((report_memory(MemReport::ArmSize) - 3) & 0xFFFFFFFC) - IRQ_STACK_SIZE;
    addr
}

fn init_mem() {
    let mmu = MMU::new(unsafe{ &mut __page_directory});
    // Standard ist Seitenfehler
    for page in 0..4096{ 
        let pde: PageDirectoryEntry;
        pde = PdEntry::new(PdEntryType::Fault).entry;
        mmu.page_directory[page as usize] = pde;
    }
    // Das erste MB wird jetzt auf sich selbst gemappt
    mmu.page_directory[0] = PdEntry::new(PdEntryType::Section).base_addr(0).rights(MemoryAccessRight::SysRwUsrNone).mem_type(MemType::NormalUncashed).entry;  // Identitätsmapping;
    // Den Stack und alles drüber (eigentlich nur die HW) brauchen wir auch:
    for page in 447..4096 {
        let pde: PageDirectoryEntry;
        pde = PdEntry::new(PdEntryType::Section).base_addr(page << 20).rights(MemoryAccessRight::SysRwUsrNone).mem_type(MemType::NormalUncashed).entry;  // Identitätsmapping
        mmu.page_directory[page as usize] = pde;
    }
    MMU::set_domain_access(0,DomainAccess::Manager);
    mmu.start();
    kprint!("MMU aktiviert.\n");
}

fn report() {
    kprint!("aihPOS"; RED);
    kprint!(" Version {}\n",VERSION; RED);
    let  (firmware_version, board_model, board_revision,serial) = (report_board_info(BoardReport::FirmwareVersion),
                                                                   report_board_info(BoardReport::BoardModel),
                                                                   report_board_info(BoardReport::BoardRevision),
                                                                   report_board_info(BoardReport::SerialNumber));
    if board_model == 0 {
        kprint!("Raspberry Pi");
    } else {
        kprint!("Unbekanntes Board");
    }
    let (serial_high,serial_low) = (serial >> 16, serial & 0xFFFF);
    kprint!(", Version {:#0x}, Seriennummer {:04x}.{:04x}\n",board_revision,serial_high,serial_low);
    kprint!("Firmwareversion {:0x}\n",firmware_version);
    kprint!("Speicherlayout:\n");
    kprint!(" IRQ-Stack: 0x{:08x}, System-Stack: 0x{:08x}, Seitendirectory: 0x{:08x}\n",determine_irq_stack(),determine_svc_stack(), unsafe{ __page_directory.as_ptr()} as *const
            u32 as u32; WHITE);
    kprint!(" Shared: 0x{:08x} - 0x{:08x}, Ende des Kernels: 0x{:08x}\n",
            unsafe{ &__shared_begin} as *const u32 as u32,
            unsafe{&__shared_end} as *const u32 as u32,
            unsafe{&__data_end} as *const u32 as u32; WHITE); 
}

fn test() {
    kprint!("Calling system.\n");
    let ret=syscall!(23,1,2);
    kprint!("Returned from system call: {}.\n",ret);
    // Das folgende sollte eine Schutzverletzung geben
    unsafe{
        let pt: *mut u32 = 0x1000000 as *mut u32;
        *pt = 42;
    }
    kprint!("Ich lebe noch.");
    debug::blink(debug::BS_HI);
}

#[inline(never)]
#[no_mangle]
#[allow(private_no_mangle_fns,unused_variables)]
#[linkage="weak"] // Verhindert, dass der Optimierer die Funktion eliminiert
pub fn svc_service_routine(nr: u32, arg1: u32, arg2: u32)  -> u32
{
    kprint!("System Call #{:X} with parameter {} and {}\n",nr,arg1,arg2);
    42
}
