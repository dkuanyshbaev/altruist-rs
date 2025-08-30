# altruist-rs

A Rust implementation of altruist.

## Prerequisites

```bash
# Install RISC-V target for ESP32-C6
rustup target add riscv32imac-unknown-none-elf

# Install flashing tools
cargo install espflash cargo-espflash
```

## Building

```bash
cargo build
```

## Flashing

1. Connect your ESP32-C6 device
2. Run the flash script:
```bash
./flash.sh
```

This will flash the firmware and start monitoring serial output.

## License

MIT.
