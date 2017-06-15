# aihPOS - another incompatible and hackneyed Pi Operating System

aihPOS ist (oder präziser: soll in Zukunft sein :smirk: ) das Betriebssystem, das im Bachelorkurs "Betriebssysteme" an der TU Chemnitz eingesetzt wird.

Verzeichnisse:
- `bin/`: nötige Tools zum Bauen, z.Z: lediglich `cargo-kernel` 
- `jtag/`: kleiner Kernel der die Nutzung von JTAG ermöglicht
- `kernel/`: Microkernel für aihPOS

## Den Kern bauen ##

Es müssen installiert sein 
- Nightly Rust
- Cargo
- Xargo

`cargo-kernel` (aus `bin`) sollte im Suchpfad sein. Dann kann die Übersetzung mit
```
cargo kernel --target=arm-none-eabihf 
```
gestartet werden.

{% github_sample werner-matthias/aihPOS/blob/master/kernel/Cargo.toml %}
