macro_rules! import_linker_address {
    ($i:ident) => {extern "C" {fn $i();}}
}
