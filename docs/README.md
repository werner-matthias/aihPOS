# aihPOS - another incompatible and hackneyed Pi Operating System

aihPOS ist (oder präziser: soll in Zukunft sein :smirk:) das Betriebssystem, das im Bachelorkurs "Betriebssysteme" an der TU Chemnitz eingesetzt wird.

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
