use core::fmt::Arguments;
use debug::kprint;

#[repr(u32)]
pub enum SysCall {
    Exit,
    Fork,
    Yield,
    Send,
    Receive,
    Write,
    Read,
}

impl SysCall {
    

    #[inline(never)]
    #[no_mangle]
    #[allow(private_no_mangle_fns,unused_variables)]
    #[linkage="weak"] // Verhindert, dass der Optimierer die Funktion eliminiert
    pub fn svc_service_routine(nr: SysCall, arg1: u32, arg2: u32, arg3: u32)  -> u32
    {
        match nr {
            SysCall::Exit =>  {},
            SysCall::Write => {
                let out: Arguments = unsafe{ *(arg2 as * const Arguments)};
                kprint::fkprint(out);
        },
            _             => {
            }
        }
        0
    }

}
