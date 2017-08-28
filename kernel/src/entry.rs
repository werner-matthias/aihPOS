#![allow(dead_code)]
use hal::cpu::Cpu;
use ::kernel_start;
/// Sprungtabelle für Ausnahmen (Interrupts, Syscalls, etc.).
#[repr(C)]
pub struct ExceptionTable {
    jmp: [u32; 8],
    dispatch: (
        extern "C" fn(), // (Re-)Start
        extern "C" fn(), // Unbekannter Befehl
        extern "C" fn(u32,u32,u32), // Systemruf
        extern "C" fn(), // Befehl soll von ungültiger Adresse gelesen werden
        extern "C" fn(), // Speicherzugriff mit ungültiger Adresse, z.B. nichtexistent oder unaligned, oder fehlende Zugriffsrechte etc.
        extern "C" fn(), // Reserivert
        extern "C" fn(), // Interrupt
        extern "C" fn(), // Schneller Interrupt
    ),
}

const JMPRPC: u32 = 0xe59ff018;  // ARM-Assembler: ldr pc, [pc, 24]

// Die Tabelle wird fest auf die Adresse 0x0 gelinkt, siehe "layout.ld".
#[link_section=".except_table"]
#[allow(non_upper_case_globals,private_no_mangle_statics)]
#[no_mangle]
#[used] // Sorgt dafür, dass die Tabelle nicht "wegoptimiert" wird.
pub static execption_table: ExceptionTable = ExceptionTable {
    jmp: [JMPRPC; 8],
    dispatch: (
        kernel_start,            // (Re-)Start
        dispatch_undefined,      // Unbekannter Befehl
        dispatch_svc,            // Systemruf 
        dispatch_prefetch_abort, // Befehl soll von ungültiger Adresse gelesen werden 
        dispatch_data_abort,     // Speicherzugriff mit ungültiger Adresse, z.B. nichtexistent oder
                                 //  unaligned, oder fehlende Zugriffsrechte etc.
        kernel_start,            // Reserivert, sollte nie auftreten; falls doch => restart
        dispatch_interrupt,      // Interrupt
        dispatch_fast_interrupt, // Schneller Interrupt
    )
};

pub struct ServiceRoutine{
    undef:      fn(*const u32),
    svc:        fn(u32,u32,u32) -> u32,
    abort:      fn(*const u32),
    data_abort: fn(*const u32),
    irq:        fn(),
    fiq:        fn(),
}

#[allow(non_upper_case_globals)]
pub static service_routine: ServiceRoutine =  ServiceRoutine{
            undef:      undefined_service_routine,
            svc:        ::svc_service_routine,
            abort:      abort_service_routine,
            data_abort: data_abort_service_routine,
            irq:        irq_service_routine,
            fiq:        irq_service_routine,
};

#[naked]
#[inline(never)]
pub extern "C" fn dispatch_undefined() {
    Cpu::save_context();
    //    
    unsafe{
        asm!("mov r0, lr\n
              sub r0, r0, #4\n
              bl undefined_service_routine");
        //\n"::"r"(undefined_service_routine):"r0":"alignstack","volatile"
        //);
    }
    Cpu::restore_context_and_return();
}

#[naked]
#[inline(never)]
#[allow(unused_variables)]
// Achtung, diese Funktion wird nur bei einem Opt-Level >= 1 korrekt übersetzt. Da #[naked] im
// unoptimierten Fall buggy^H^H^H^H unerwartete Ergebnisse erzeugt (vgl.https://stackoverflow.com/
// questions/42238878/why-are-there-extra-asm-instructions-in-a-naked-rust-function),
// würde r0 bei Opt-Level=0 überschrieben oder der Stack korrumpiert werden.
pub extern "C" fn dispatch_svc(nr: u32, arg1: u32, arg2: u32){
    unsafe{
        // r0 braucht (zunächst) nicht gerettet zu werden; das wird bei der Rückgabe
        // sowieso überschrieben
        asm!("stmfd sp!, {r1-r11, lr}":::"memory");
        asm!("blx $0"::"r"(service_routine.svc):"r0","r1","r2","r3","r4","r5",
             "r6","r7","r8","r9","r10","r11","memory":"alignstack","volatile");
        asm!("ldmfd sp!, {r1-r11, pc}^":::"memory");
    };
}

#[naked]
pub extern "C" fn dispatch_prefetch_abort() {
    //Cpu::save_context();
    unsafe{
        asm!("mov lr, r0":::"memory");
        asm!("blx $0"::"r"(service_routine.abort):"r0","r1","r2","r3","r4","r5",
             "r6","r7","r8","r9","r10","r11","memory":"alignstack","volatile");
    }
    // abort_service_routine();
    //Cpu::restore_context_and_return();
}

#[naked]
pub extern "C" fn dispatch_data_abort() {
    //Cpu::save_context();
    unsafe {
        asm!("mov lr, r0":::"memory");
        asm!("blx $0"::"r"(service_routine.data_abort):"r0","r1","r2","r3","r4","r5",
             "r6","r7","r8","r9","r10","r11","memory":"alignstack","volatile");
    }
    //data_abort_service_routine();
    //Cpu::restore_context_and_return();
}

#[naked]
pub extern "C" fn dispatch_interrupt() {
    Cpu::save_context();
    Cpu::restore_context_and_return();
}

#[naked]
pub extern "C" fn dispatch_fast_interrupt() {
    Cpu::save_context();
    Cpu::restore_context_and_return();
}

#[inline(never)]
#[no_mangle]
#[allow(private_no_mangle_fns)]
#[linkage="weak"] // Verhindert, dass der Optimierer die Funktion eliminiert
pub fn irq_service_routine() 
{
    kprint!("Interrupt!\n");
}

#[inline(never)]
#[no_mangle]
#[allow(private_no_mangle_fns)]
#[linkage="weak"] // Verhindert, dass der Optimierer die Funktion eliminiert
pub fn undefined_service_routine(ptr: *const u32) {
    let addr = unsafe{ ptr.offset(-1)};
    let cmd  = unsafe{ *addr };
    // Sobald eine Prozessabstraktion existiert, sollte dies angepasst werden.
    kprint!("Unbekannter Befehl 0{:08x} @ 0{:08x}\n",cmd, addr as usize);
    panic!("Unbehandelt");
}

#[inline(never)]
#[no_mangle]
#[allow(private_no_mangle_fns)]
#[linkage="weak"] // Verhindert, dass der Optimierer die Funktion eliminiert
pub fn abort_service_routine(adr: *const u32) {
    // Im Moment wird das Windows-3.X-Verhalten simuliert.
    // Sobald eine Prozessabstraktion existiert, sollte dies angepasst werden.
    kprint!("Allgemeine Schutzverletzung @ {:?}\n",adr);
    panic!("Unbehandelt");
}

#[inline(never)]
#[no_mangle]
#[allow(private_no_mangle_fns)]
#[linkage="weak"] // Verhindert, dass der Optimierer die Funktion eliminiert
pub fn data_abort_service_routine(adr: *const u32) {
    // Im Moment wird das Windows-3.X-Verhalten simuliert.
    // Sobald eine Prozessabstraktion existiert, sollte dies angepasst werden.
    kprint!("Allgemeine Schutzverletzung bei Datenzugriff @ {:?}\n",adr);
    panic!("Unbehandelt");
}

// Als Systemruf wird der Softwareinterrupt 42 genutzt. Der erste Parameter ist der Rufselector,
// die anderen je nach Bedarf.
#[inline(never)]
// Die Angabe der Linkage sorgt dafür, dass die Funktion als zu exportieren markiert 
// und damit nicht vom Optimizer global verändert wird.
#[linkage="weak"]
#[allow(unused_variables)]
pub fn syscall(nr: u32, arg1: u32, arg2: u32) -> u32 {
    #[allow(unused_assignments)]
    let ret: u32;// = nr;
    unsafe{
        // Die Parameter (nach ARM Calling Konvention in den Registern r0-r2) werden durchgereicht.
        // Sicherheitshalber werden diese Register dem Compiler als "vermint" gemeldet.
        asm!("":::"r0","r1","r2");
        // Der Optimizer erkennt keinen Ruf und rettet das Link-Register nicht, daher
        // muss dies manuell geschehen
        asm!("push {lr}");   
        asm!("svc #42");
        asm!("pop {lr}");
        asm!("mov $0,r0":"=r"(ret)::"r0");
    }
    ret
}

// Zur Bequemlichkeit gibt es ein Macro, das Systemrufe mit 0 bis 2 Argumenten zulässt.
// Ungenutzte Argumente werden auf Null gesetzt.
macro_rules! syscall{
    ($e:expr) => { syscall($e,0,0)};
    ($e:expr,$a1:expr) => { syscall($e,$a1,0)};
    ($e:expr,$a1:expr,$a2:expr) => { syscall($e,$a1,$a2)};
}
