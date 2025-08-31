# Altruist

Smart home system.

## Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Rust RISC-V target
rustup target add riscv32imac-unknown-none-elf

# Install system dependencies for probe-rs (Ubuntu/Debian)
sudo apt install -y libusb-1.0-0-dev libftdi1-dev libudev-dev

# Install probe-rs
cargo install probe-rs-tools

# Set USB permissions for ESP32
sudo tee /etc/udev/rules.d/99-esp32.rules << 'EOF'
SUBSYSTEM=="usb", ATTR{idVendor}=="303a", ATTR{idProduct}=="1001", MODE="0666"
EOF
sudo udevadm control --reload-rules
sudo udevadm trigger

# Unplug and replug your ESP32-C6 after this
```

## Run

```bash
# Flash and run
cargo run --release

# Monitor serial output (in another terminal)
./log.sh
```
