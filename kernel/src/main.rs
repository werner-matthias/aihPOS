#![no_std]
#![no_main]
#![feature(
    //abi_unadjusted,         //
    alloc,                  // Nutzung der Alloc-Crate
    attr_literals,          // Literale in Attributen (nicht nur Strings)
    asm,                    // Assembler in Funktionen...
    // concat_idents,
    collections,            // Nutzung des Collection-Crate
    const_fn,               // const Funktionen (für Constructoren)
    compiler_builtins_lib,  // Nutzung der Compiler-Buildins-Bibliothek (div, mul, ...)
    core_intrinsics,        // Nutzung der Intrinsics der Core-Bibliothek
    i128_type,              // 128-Bit-Typen
    lang_items,             // Funktionen interne Funktionen ersetzen (panic)
    linkage,                // Angaben zum Linktyp (z.B. Sichtbarkeit)
    naked_functions,        // Funktionen ohne Prolog/Epilog
    plugin,                 // Nutzung von Compiler-Plugins
    repr_align,             // Alignment
    step_by,                // Spezifische Schrittweite bei Iterationen
    // use_extern_macros,
    used,                   // Verbot, scheinbar toten Code zu eliminieren
)
]
#![plugin(compiler_error)]

/// Benutzte Crates
//#[macro_use]
extern crate alloc;
extern crate bit_field;
#[macro_use] extern crate collections;
extern crate compiler_builtins;
extern crate kalloc;
#[allow(unused_imports)]  
#[macro_use] extern crate lazy_static;
extern crate spin;

const IRQ_STACK_SIZE: u32 = 2048;

// 
extern {
    static mut __page_directory: [PageDirectoryEntry;4096];
    static __text_end: u32;
    static __kernel_stack: u32;
    static __data_end: u32;
    static __shared_begin: u32;
    static __shared_end:   u32;
}

#[macro_use] mod debug;
#[macro_use] mod hal;
mod panic;
mod sync;
mod mem;

use hal::board::{MemReport,BoardReport,report_board_info,report_memory};
use hal::entry::syscall;
use hal::cpu::{Cpu,ProcessorMode,MMU};
use mem::{PdEntryType,PageDirectoryEntry,PdEntry,DomainAccess,MemoryAccessRight,MemType};
use mem::frames::FrameManager;
pub use mem::heap::{aihpos_allocate,aihpos_deallocate,aihpos_usable_size,aihpos_reallocate_inplace,aihpos_reallocate};

use collections::vec::Vec;
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[no_mangle]      // Name wird für den Export nicht verändert
#[naked]          // Kein Prolog, da dieser den Stack nutzen würde, den wir noch nicht gesetzt haben
#[allow(unreachable_code)]
pub extern fn kernel_start() {
    // Zum Start existiert noch kein Stack. Daher setzen wir einen temporären Stack, der nach dem Textsegment liegt.
    // Das Symbol ist in "layout.ld" definiert.
    Cpu::set_stack(unsafe {&__kernel_stack}  as *const u32 as u32);
    // Nun kann die Größe des Speichers und damit die Adresse für den "echten" Stacks bestimmt werden
    Cpu::set_stack(determine_svc_stack());
    kernel_init();
    unreachable!();
}

#[inline(never)] // Verbietet dem Optimizer, kernel_init() und darin aufgerufene Funktionen mit kernel_start()
                 // zu verschmelzen. Dies würde wegen #[naked]/keinen Stack schief gehen
#[allow(unreachable_code)]
fn kernel_init() -> ! {
    report();
    init_mem();
    test();
    loop {}
    unreachable!();
}

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
    init_stacks();
    init_paging();
}

fn init_stacks() {
    // Stack für die anderen Ausnahme-Modi.  Irq, Fiq, Abort und Undef teilen sich einen Stack, der System-Mode nutzt
    // den User-Mode-Stack und muss nicht gesetzt werden.
    let adr = determine_irq_stack();
    Cpu::set_mode(ProcessorMode::Irq);
    Cpu::set_stack(adr);
    Cpu::set_mode(ProcessorMode::Fiq);
    Cpu::set_stack(adr);
    Cpu::set_mode(ProcessorMode::Abort);
    Cpu::set_stack(adr);
    Cpu::set_mode(ProcessorMode::Undef);
    Cpu::set_stack(adr);
    // ...und zurück in den Svc-Mode
    Cpu::set_mode(ProcessorMode::Svc);
}

fn init_paging() {
    unimplemented!();
}

/*
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
}*/

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
    let mut frame_manager = FrameManager::new();
    frame_manager.mark_not_available(0..0x0002ffff);
    //kprint!("ff: {}\n",frame_manager.first_free);
    for _ in 0..17 {
        let adr: u32 = frame_manager.allocate();
        kprint!("Neuer Frame @ {:08x}\n",adr);
    }
    frame_manager.release(0x00090000);
    for _ in 0..2 {
        let adr: u32 = frame_manager.allocate();
        kprint!("Neuer Frame @ {:08x}\n",adr);
    }
    {
        let v = vec![1,2,3];
        for i in v {
            kprint!("{} ",i);
        }
    }
    /*
    // Das folgende sollte eine Schutzverletzung geben
    unsafe{
        let pt: *mut u32 = 0x1000000 as *mut u32;
        *pt = 42;
    }
    kprint!("Ich lebe noch.");
     */
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
