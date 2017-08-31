#![allow(non_snake_case)]
#![no_std]
#![no_main]
#![feature(
    alloc,                    // Nutzung der Alloc-Crate
    //allocator_api,          // Nutzung der Allocator-API
    //abi_unadjusted,         //
    attr_literals,            // Literale in Attributen (nicht nur Strings)
    asm,                      // Assembler in Funktionen...
    associated_type_defaults, // Verknüpfung von Traits mit Typen
    // concat_idents,
    //collections,            // Nutzung des Collection-Crate
    const_fn,                 // const Funktionen (für Constructoren)
    //range_contains,          // Funktion zur Bestimmung, ob eine Range einen Wert enthält
    compiler_builtins_lib,    // Nutzung der Compiler-Buildins-Bibliothek (div, mul, ...)
    core_intrinsics,          // Nutzung der Intrinsics der Co[dependencies.aihpos_process]
    doc_cfg,                  // Plattform-spezifische Dokumentation
    drop_types_in_const,      // Statics dürfen Typen mit Destructoren enthalten
    global_allocator,         // eigener globaler Allocator
    i128_type,                // 128-Bit-Typen
    inclusive_range_syntax,   // Inklusiver Bereich mit "..."   
    iterator_step_by,         // Spezifische Schrittweite bei Iterationen
    lang_items,               // Funktionen interne Funktionen ersetzen (panic)
    linkage,                  // Angaben zum Linktyp (z.B. Sichtbarkeit)
    naked_functions,          // Funktionen ohne Prolog/Epilog
    //nonzero,                  // Werte ohne Null (hier: usize)
    plugin,                   // Nutzung von Compiler-Plugins
    repr_align,               // Alignment
    use_extern_macros,
    //unique,                   // Unique-Pointer
    used,                     // Erlaubt das Verbot, scheinbar toten Code zu eliminieren
)
]
#![plugin(compiler_error)]
#![doc(html_logo_url = "file:///Users/mwerner/Development/aihPOS/aihPOS-docs/logo-128.png")]
/// Benutzte Crates
extern crate alloc;
extern crate bit_field;
extern crate compiler_builtins;

#[macro_use]
mod aux_macros;
#[macro_use] mod debug;
mod hal;
mod panic;
mod data;
mod process;
mod sync;
//use alloc::boxed::Box;

//#[macro_use] mod hal;
use hal::bmc2835::{MemReport,BoardReport,report_board_info,report_memory};
use hal::bmc2835::irq_controller::{IrqController,InterruptPending,BaseInterruptPending};
#[macro_use]
mod entry;
use debug::*;
use entry::syscall;
use hal::cpu::{Cpu,ProcessorMode,MMU};
use core::mem::size_of;
//use sync::no_concurrency::NoConcurrency;
use data::kernel::{KernelData,KERNEL_PID};
mod memory;
use memory::*;

import_linker_symbol!(__text_end);
import_linker_symbol!(__data_start);
import_linker_symbol!(__data_end);
import_linker_symbol!(__bss_start);
import_linker_symbol!(__kernel_stack);

const IRQ_STACK_SIZE: usize = 2048;
const SVC_STACK_SIZE: usize = 64 * 1024;
pub  const INIT_HEAP_SIZE: usize = 25 * 4096; // 25 Seiten = 100 kB

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[no_mangle]      // Name wird für den Export nicht verändert
#[naked]          // Kein Prolog, da dieser den Stack nutzen würde, den wir noch nicht gesetzt haben
#[allow(unreachable_code)]
/// `kernel_start()` ist die Einsprungsstelle in den aihPOS-Code.
/// Der erste Eintrag in der Sprungtabelle hält die Adresse von `kernel_start`, so dass aihPOS sowohl
/// nach dem Einschalten (old_kernel in "config.txt" gesetzt) als auch nach einer Neustart-Ausnahme
/// hier startet.
pub extern fn kernel_start() {
    // Zum Start existiert noch kein Stack. Daher setzen wir einen temporären Stack, der nach
    //  dem Textsegment liegt.
    // Das Symbol ist in "layout.ld" definiert.
    Cpu::set_stack(__kernel_stack as Address);
    // Nun kann die Größe des Speichers und damit die Adresse für den "echten" Stacks bestimmt werden
    Cpu::set_stack(determine_svc_stack());
    kernel_init();
    unreachable!();
}

/// Erledigt alle Initialisierungen:
/// * Setzen der Stacks für Ausnahmemodi
/// * Anlegen des Heaps
/// * Einrichten Paging
/// * Interrups
// Verbietet dem Optimizer, kernel_init() und darin aufgerufene Funktionen mit
// kernel_start() zu verschmelzen. Dies würde wegen #[naked]/keinen Stack schief
// gehen
#[inline(never)]
#[allow(unreachable_code)]
pub(self) fn kernel_init() -> ! {
    KernelData::set_pid(KERNEL_PID);
    report();
    init_mem();
    init_interrupt();
    test();
    loop {}
    unreachable!();
}

#[inline(never)]
fn determine_irq_stack() -> Address {
    report_memory(MemReport::ArmSize) 
}

#[inline(never)]
fn determine_svc_stack() -> Address {
    kprint!("determine stack called.\n";WHITE);
    report_memory(MemReport::ArmSize) - IRQ_STACK_SIZE
}

#[inline(never)]
fn init_mem() {
    kprint!("Init stacks...");
    init_stacks();
    kprint!("done.\nInit heap...");
    memory::init_heap(__bss_start as Address, INIT_HEAP_SIZE);
    kprint!("done.\nInit pagetable...");
    init_paging();
    kprint!("done.\n");
}

/// Es werden die Stacks für alle Ausname-Modi gesetzt.
/// Irq, Fiq, Abort und Undef teilen sich einen Stack, der System-Mode nutzt
/// den User-Mode-Stack und muss nicht gesetzt werden.
#[inline(never)]
fn init_stacks() {
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
    MMU::set_page_dir(PageDirectory::addr());
    let page_directory: &mut PageDirectory = KernelData::page_directory();
    let frame_allocator: &mut FrameManager = KernelData::frame_allocator();
    
    // Standard ist Seitenfehler
    for section in Section::iter(0 .. MAX_ADDRESS) {
        page_directory[section.nr()] = MemoryBuilder::<DirectoryEntry>::new_entry(DirectoryEntry::Fault)
            .entry();
    }
    // Für den Kernel richten wir zwei Seitentabellen ein:
    // * Code (Textsegment), Daten und Heap => kpage_table
    // * Stacks                             => spage_table
    //
    // # Anmerkung:
    // Der Kernel ist relativ klein, daher reicht eine Seitentabelle für Code und Daten.
    // Sollte sich dies mal ändern, müsste zunächst die Anzahl der benötigten Tabellen bestimmt
    // werden. In diesem Fall sollten die Tabellen auf dem Heap angelegt werden.
    let kpage_table: &mut PageTable = &mut KernelData::kpages();
    kpage_table.invalidate();
    let spage_table: &mut PageTable = &mut KernelData::spages();
    spage_table.invalidate();
    // Die Seitentabellen werden in das Seitenverzeichnis eingetragen
    page_directory[0] = MemoryBuilder::<DirectoryEntry>::new_entry(DirectoryEntry::CoarsePageTable)
        .base_addr(kpage_table.addr())
        .entry();
    page_directory[Section::from_addr(determine_svc_stack() - 65556 ).nr()] =
        MemoryBuilder::<DirectoryEntry>::new_entry(DirectoryEntry::CoarsePageTable)
        .base_addr(spage_table.addr())
        .entry();
    // Der Kernel-Bereich wird auf sich selbst gemappt:
    // Dabei ist der Code ausführbar, Daten nicht.
    // Code
    for frm in Frame::iter(0 .. __text_end as usize) {
        kpage_table[frm.rel()] = MemoryBuilder::<TableEntry>::new_entry(TableEntry::SmallPage)
            .base_addr(frm.start())
            .rights(MemoryAccessRight::SysRwUsrNone)
            .mem_type(MemType::NormalWT)
            .domain(0)
            .entry();
        frame_allocator.reserve(frm).expect("frame allocator failed");
    }
    // Kernel-Daten + BSS
    for frm in Frame::iter(__data_start as usize .. __bss_start as usize + INIT_HEAP_SIZE) {
        kpage_table[frm.rel()] = MemoryBuilder::<TableEntry>::new_entry(TableEntry::SmallPage)
            .base_addr(frm.start())
            .rights(MemoryAccessRight::SysRwUsrNone)
            .mem_type(MemType::NormalWB)
            .no_execute(true)
            .domain(0)
            .entry();
        frame_allocator.reserve(frm).expect("frame allocator failed");
    }
    // Stacks
    for frm in  Frame::iter(determine_svc_stack() - SVC_STACK_SIZE .. determine_irq_stack() -1) {
        spage_table[frm.rel()] = MemoryBuilder::<TableEntry>::new_entry(TableEntry::SmallPage)
            .base_addr(frm.start())
            .rights(MemoryAccessRight::SysRwUsrNone)
            .mem_type(MemType::NormalWT)
            .no_execute(true)
            .domain(0)
            .entry();
        frame_allocator.reserve(frm).expect("frame allocator failed");
    }
    // Der Rest des Speichers (Geräte) wird auf sich selbst gemappt
    // TODO: nur die tatsächlichen Geräte mappen
    for section in Section::iter(determine_irq_stack() .. MAX_ADDRESS) {
        let pde = MemoryBuilder::<DirectoryEntry>::new_entry(DirectoryEntry::Section)
            .base_addr(section.start())
            .rights(MemoryAccessRight::SysRwUsrNone)
            .mem_type(MemType::NormalUncashed)
            .domain(0)
            .no_execute(true)
            .entry();
        page_directory[section.nr()] = pde;
    }
    MMU::set_domain_access(0,DomainAccess::Manager);
    kprint!("Vorbereitungen für MMU-Aktivierung abgeschlossen.\n");
    unsafe{ MMU::start(); }
    kprint!("MMU aktiviert.\n");
}

fn init_interrupt() {
    let irq_controller = IrqController::get();
    irq_controller.enable(InterruptPending::Basic(BaseInterruptPending::ARMtimer));
    let timer = hal::bmc2835::arm_timer::ArmTimer::get()
        .resolution(hal::bmc2835::arm_timer::ArmTimerResolution::Counter23Bit)
        .predivider(250)
        .count(1000000)
        .enable(true)
        .activate_interrupt(true);
    kprint!("Timer: {:?}\n",timer;WHITE);
    kprint!("Interrupt aktiviert.\n";BLUE);
}
 
fn report() {
    kprint!("aihPOS"; BLUE);
    kprint!(" Version {}\n",VERSION; BLUE);
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
    kprint!("0x{:08x} ({:10}): Anfang Kernelcode\n",kernel_start as usize,kernel_start as usize; WHITE);
    kprint!("0x{:08x} ({:10}): Ende Kernelcode\n",__text_end as usize,__text_end as usize; WHITE);
    kprint!("0x{:08x} ({:10}): Anfang Kerneldaten\n",__data_start as usize, __data_start as usize; WHITE);
    kprint!("0x{:08x} ({:10}): Anfang Pagedirectory\n",PageDirectory::addr(), PageDirectory::addr(); WHITE);
    kprint!("0x{:08x} ({:10}): Ende Pagedirectory\n",PageDirectory::addr() + size_of::<PageDirectory>(), PageDirectory::addr() + size_of::<PageDirectory>(); WHITE);
    kprint!("0x{:08x} ({:10}): Ende Kerneldaten\n",__data_end as usize,__data_end as usize; WHITE);
    kprint!("0x{:08x} ({:10}): Anfang Kernelheap\n",__bss_start as usize, __bss_start as usize; WHITE);
    kprint!("0x{:08x} ({:10}): Initiales Ende Kernelheap\n",__bss_start as usize + INIT_HEAP_SIZE, __bss_start as usize + INIT_HEAP_SIZE; WHITE);
    kprint!("0x{:08x} ({:10}): TOS System\n",determine_svc_stack() as usize, determine_svc_stack() as usize; WHITE);
    kprint!("0x{:08x} ({:10}): TOS Interrupt\n",determine_irq_stack() as usize, determine_irq_stack() as usize; WHITE);
    debug::kprint::deb_info();
}

fn test() {
    kprint!("Calling system.\n");
    let ret=syscall!(23,1,2);
    //let stack: [u32;20] = [0u32;20];
    //let sp: *const u32 = &stack[19];
    kprint!("Returned from system call: {}.\n",ret);
    Cpu::enable_interrupts();
    /*
    unsafe{
        let mut ptr: *const u32 = sp;
        asm!("mov r9, sp":::"r9":"volatile");
        asm!("mov sp, $0"::"r"(sp):"sp":"volatile");
        asm!("mov r0, 0":::"r0":"volatile");
        asm!("mov r1, #0x1":::"r1":"volatile");
        asm!("mov r2, #0x2":::"r2":"volatile");
        asm!("mov r3, #0x3":::"r3":"volatile");
        asm!("mov r4, #0x4":::"r4":"volatile");
        asm!("mov r5, #0x5":::"r5":"volatile");
        asm!("mov r6, #0x6":::"r6":"volatile");
        asm!("mov r7, #0x7":::"r7":"volatile");
        //asm!("mov r11, #0x11":::"r11":"volatile");
        //asm!("mov r12, #0x12":::"r12":"volatile");
        asm!("stmfd sp!, {r0-r10}":::"sp":"volatile");
        asm!("mov sp, r9":::"r9":"volatile");
        kprint!("Inspect stack\n"; BLUE);
        let mut l: u8 = 20;
        while l>0 {
            kprint!("{:08x}: {:08x}\n",ptr as u32, *ptr; WHITE);
            ptr = ptr.offset(-1);
            l -= 1;
        }
    }
     */

    
    /*
    {
        let v1 = Box::new(0);
        let v2 = Box::new((23,42));
        let v3 = Box::new(1); 
        kprint!("v1 = {}, v2 = {:?}, v3 = {}.\n",*v1,*v2,*v3);
        drop(v1);
    }
     */
    /*
    // Das folgende sollte eine Schutzverletzung geben
    unsafe{
        let pt: *mut u32 = 0x1000000 as *mut u32;
        *pt = 42;
    }
    kprint!("Ich lebe noch.");
     */
    //Cpu::enable_interrupts();
    let timer = hal::bmc2835::arm_timer::ArmTimer::get();
    loop {
       kprint!(".";WHITE);
       // if timer.interrupt_occured() {
            //kprint!("Interrupt should have occured!\n";RED);
         //   timer.reset_interrupt();
        //}
        debug::blink::blink_once(debug::blink::BS_HI);
    }
    debug::blink::blink(debug::blink::BS_HI);
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


pub fn syscall_yield() {
    syscall!(1);
}

pub fn process_a() {
    loop {
        let n: u32 = 0;
        kprint!("Ich bin A: {}\n",n);
        n.wrapping_add(1);
        syscall_yield();
    }
}

pub fn process_b() {
    loop {
        let n: u32 = 0;
        kprint!("Ich bin B: {}\n",n);
        n.wrapping_add(1);
        syscall_yield();
    }
}
