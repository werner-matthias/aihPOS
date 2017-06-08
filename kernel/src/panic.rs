use core;
    
#[lang = "eh_personality"] extern fn eh_personality() {}

#[allow(private_no_mangle_fns)]
#[no_mangle]
pub extern fn __aeabi_unwind_cpp_pr0() -> ()
{
    loop {}
}

#[allow(private_no_mangle_fns)]
#[no_mangle]
pub extern fn __aeabi_unwind_cpp_pr1() -> ()
{
    loop {}
}

#[allow(private_no_mangle_fns)]
#[allow(non_snake_case)] #[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}

#[allow(private_no_mangle_fns)]
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(msg: core::fmt::Arguments,
                               file: &'static str,
                               line: u32) -> ! {
    ::debug::fkprintc(format_args!("\nKernel Panic!\n"),::debug::kprint::RED);
    ::debug::fkprint(msg);
    ::debug::fkprint(format_args!(" @{}, Zeile: {}",file,line));
    loop{}
}
