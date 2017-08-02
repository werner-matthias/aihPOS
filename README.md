# aihPOS - another incompatible and hackneyed Pi Operating System

aihPOS is (better: shell be in the future :smirk:) the operating system used in the undergraded course "Operating Systems" at the TU Chemnitz.

Directories:
- `bin/`: tools for building; currently `cargo-kernel` only
- `jtag/`: small kernel to allow development with use of JTAG
- `kernel/`: micro kernel for aihPOS

## How to build the kernel ##
Prerequisites: 
- Nightly Rust
- Cargo
- Xargo

Put `cargo-kernel` in your path. Then run
```
cargo kernel --target=arm-none-eabihf 
```

## Remark
Since aihPOS should support an undergraduate course at a German university, all
source comments are in German. Possibly, I will create a branch with translations
once the code is stable.
