macro_rules! import_linker_symbol {
    ($i:ident) => {extern "C" {fn $i();}}
}
