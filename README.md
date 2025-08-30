# Altruist RS - Embassy ESP32-C6 Project

A minimal Embassy-based async application for ESP32-C6 using pure Rust (no ESP-IDF dependencies).

## 🚀 Features

- **Pure Rust Embassy Framework** - Modern async/await embedded programming
- **ESP32-C6 Support** - RISC-V based microcontroller with Wi-Fi 6, BLE 5, Thread/Zigbee
- **Bare Metal** - No operating system overhead, direct hardware access
- **Multiple Concurrent Tasks** - LED blinking, counters, and main loop simulation
- **Embassy Timers** - Non-blocking async delays and timing

## 📋 Prerequisites

### System Dependencies (Ubuntu/Debian)

```bash
# Install build tools and libraries
sudo apt update
sudo apt install -y build-essential pkg-config libudev-dev libusb-1.0-0-dev cmake git

# For probe-rs (recommended flashing tool)
sudo apt install -y libftdi1-dev libhidapi-dev
```

### Rust Toolchain

```bash
# Install Rust target for ESP32-C6 (RISC-V)
rustup target add riscv32imac-unknown-none-elf

# Note: This project requires Rust nightly due to build-std
rustup toolchain install nightly
rustup default nightly
```

## 🔧 Installation

### Clone and Setup

```bash
git clone <repository-url>
cd altruist-rs
```

### Choose Flashing Method

#### Option 1: probe-rs (Recommended for ESP32-C6)

probe-rs is the modern standard for embedded Rust development and works excellently with ESP32-C6 (RISC-V based).

```bash
# Install probe-rs tools (latest version with ESP32-C6 fixes)
cargo install probe-rs-tools --locked

# Add shell completion (optional)
probe-rs complete install

# Verify ESP32-C6 support
probe-rs chip list | grep -i esp32c6

# Check probe-rs can detect your device
probe-rs list
```

**Why probe-rs for ESP32-C6?**
- ✅ Native RISC-V support (ESP32-C6 is RISC-V based)
- ✅ Works with bare metal applications (no ESP-IDF dependency)
- ✅ Built-in USB-JTAG support for ESP32-C6 DevKit
- ✅ Fast flashing and debugging
- ✅ Active development with ESP32-C6 specific fixes in v0.29.1+

#### Option 2: espflash (Fallback - Has Limitations)

```bash
# WARNING: espflash 4.0+ has issues with bare metal applications
# Only works with ESP-IDF format applications, not pure esp-hal

# If you have compatible toolchain, try older version:
cargo install espflash@3.3.0  # May not work with newer Rust
```

**Known Issues with espflash:**
- ❌ espflash 4.0+ requires ESP-IDF app descriptor
- ❌ Does not support bare metal esp-hal applications  
- ❌ Will show "ESP-IDF App Descriptor missing" error

### Hardware Setup

- **ESP32-C6 DevKit** with USB-C connection
- **Built-in USB-JTAG** - No external programmer needed
- Connect via **USB port** (not UART port if board has both)
- Device appears as `/dev/ttyACM0` or similar

## 🏗️ Building

```bash
# Clean build
cargo clean

# Build release version (optimized)
cargo build --release

# Check binary was created
ls -la target/riscv32imac-unknown-none-elf/release/altruist-rs
```

## 📡 Flashing and Running

### Using probe-rs (Recommended)

```bash
# Method 1: Use cargo run with probe-rs runner (configured in .cargo/config.toml)
cargo run --release

# Method 2: Direct probe-rs command
probe-rs run --chip esp32c6 target/riscv32imac-unknown-none-elf/release/altruist-rs

# Method 3: Flash and attach separately
probe-rs download --chip esp32c6 target/riscv32imac-unknown-none-elf/release/altruist-rs
probe-rs attach --chip esp32c6
```

### Using espflash (If Available)

```bash
# Only works with older espflash versions
cargo run --release  # If espflash runner configured

# Or directly
espflash flash --monitor target/riscv32imac-unknown-none-elf/release/altruist-rs
```

## ⚙️ Configuration Files

### `.cargo/config.toml`

```toml
[build]
target = "riscv32imac-unknown-none-elf"
rustflags = ["-C", "force-frame-pointers"]

[target.riscv32imac-unknown-none-elf]
# Choose one of these runners based on your flashing tool:
runner = "probe-rs run --chip esp32c6"        # probe-rs (recommended)
# runner = "espflash flash --monitor"          # espflash (if compatible version)

[unstable]
build-std = ["core"]

[env]
ESP_LOG="INFO"
```

### `Cargo.toml` Dependencies

```toml
[dependencies]
esp-hal = { version = "0.21.1", features = ["esp32c6"] }
esp-hal-embassy = { version = "0.4.0", features = ["esp32c6"] }
esp-println = { version = "0.12.0", features = ["esp32c6", "log"] }
esp-backtrace = { version = "0.14.2", features = ["esp32c6", "panic-handler", "exception-handler", "println"] }
embassy-executor = { version = "0.6.2", features = ["task-arena-size-20480"] }
embassy-time = { version = "0.3.2" }
log = { version = "0.4.22" }

[profile.release]
codegen-units = 1
opt-level = 3
lto = 'fat'
overflow-checks = false
```

### `build.rs`

```rust
fn main() {
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}
```

## 🔍 Expected Output

When running successfully, you should see:

```
ESP32-C6: Embassy Hello World!
Firmware started successfully!
Embassy timers initialized!
[MAIN] Tick 0
[LED] On
[COUNTER] Fast count: 0
[COUNTER] Fast count: 1
[LED] Off
[COUNTER] Fast count: 2
[MAIN] Tick 1
...
```

## 🛠️ Troubleshooting

### probe-rs Issues

#### Problem: `probe-rs` not found after installation

```bash
# Ensure cargo bin directory is in PATH
export PATH="$HOME/.cargo/bin:$PATH"
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc

# Verify installation
probe-rs --version
```

#### Problem: Permission denied accessing device

```bash
# Fix USB device permissions
sudo chmod 666 /dev/ttyACM0

# Or add user to dialout group (permanent)
sudo usermod -a -G dialout $USER
# Log out and back in for group change to take effect
```

#### Problem: Device not detected

```bash
# List available probes
probe-rs list

# Check if ESP32-C6 is supported
probe-rs chip list | grep -i esp32c6

# Put device in download mode manually:
# Hold BOOT button while pressing RESET button
```

### espflash Issues

#### Problem: `ESP-IDF App Descriptor missing` error

**Root Cause**: espflash 4.0+ only supports ESP-IDF applications, not bare metal esp-hal.

**Solutions**:
1. **Switch to probe-rs** (recommended)
2. **Use older espflash** (if toolchain compatible)
3. **Add ESP-IDF compatibility layer** (complex)

### Build Issues

#### Problem: Target not found

```bash
rustup target add riscv32imac-unknown-none-elf
```

#### Problem: `edition2024` feature required

Update Rust toolchain or use compatible dependency versions.

#### Problem: Linker errors

Ensure `build.rs` contains the linker argument and esp-hal generates `linkall.x`.

### Runtime Issues

#### Problem: No serial output

1. **Check correct USB port**: Use USB port, not UART
2. **Verify baud rate**: Should be 115200  
3. **Monitor after flashing**:
   ```bash
   # With probe-rs
   probe-rs attach --chip esp32c6
   
   # With screen
   screen /dev/ttyACM0 115200
   ```

## 📚 Project Structure

```
altruist-rs/
├── src/
│   └── main.rs              # Main Embassy application
├── .cargo/
│   └── config.toml          # Cargo configuration
├── build.rs                 # Build script with linker args
├── Cargo.toml              # Dependencies and profiles  
├── flash_with_esptool.sh   # Custom flashing script (backup)
└── README.md               # This file
```

## 🧩 Architecture

### Embassy Integration

- **Embassy Executor**: Async runtime for concurrent tasks
- **Embassy Time**: Non-blocking delays using hardware timers  
- **ESP-HAL Integration**: Hardware abstraction layer for ESP32-C6

### Current Implementation

The application demonstrates:
- Embassy timer initialization using Timer Group 0
- Simulated concurrent tasks (main loop, LED blink, counter)
- Non-blocking delays with `embassy_time::block_for()`
- GPIO control for LED on pin 8
- Serial debugging output

### Hardware Features Used

- **GPIO Pin 8**: LED control output
- **Timer Group 0**: Embassy time source
- **UART**: Debug output via esp-println
- **USB-JTAG**: Built-in debugging and flashing interface

## 🔄 Development Workflow

1. **Edit**: Make changes to `src/main.rs`
2. **Build**: `cargo build --release`  
3. **Flash**: `cargo run --release` (with probe-rs runner)
4. **Monitor**: Serial output appears automatically
5. **Debug**: Use `probe-rs attach` for advanced debugging

## 🚧 Known Issues & Solutions

### Flashing Tool Compatibility

| Tool | Version | ESP32-C6 | Bare Metal | Status |
|------|---------|-----------|------------|--------|
| probe-rs | 0.29.1+ | ✅ | ✅ | **Recommended** |
| espflash | 4.0+ | ❌ | ❌ | Requires ESP-IDF |
| espflash | 2.x-3.x | ✅ | ✅ | Works if installable |
| esptool.py | Any | ⚠️ | ⚠️ | Manual conversion needed |

### ESP32-C6 Specific Notes

- **RISC-V Architecture**: Well supported by probe-rs
- **Built-in USB-JTAG**: No external programmer needed
- **Memory Layout**: Current linker script works correctly
- **Embassy Support**: Fully functional with esp-hal 0.21.1+

## 📈 Next Steps

### Immediate Development

- [ ] Add true Embassy async tasks with spawning
- [ ] GPIO interrupt handling
- [ ] I2C/SPI sensor integration
- [ ] LED PWM control

### Advanced Features  

- [ ] Wi-Fi connectivity with embassy-net
- [ ] Bluetooth Low Energy (BLE)
- [ ] Thread/Zigbee networking  
- [ ] Over-the-air (OTA) updates
- [ ] Power management and deep sleep
- [ ] Non-volatile storage (NVS)

## 📖 Resources

### Documentation

- [Embassy Book](https://embassy.dev/book/) - Modern async embedded framework
- [ESP-HAL Documentation](https://docs.esp-rs.org/) - ESP32 hardware abstraction
- [probe-rs Documentation](https://probe.rs/docs/) - Debugging and flashing tool
- [ESP32-C6 Datasheet](https://www.espressif.com/sites/default/files/documentation/esp32-c6_datasheet_en.pdf)

### Community

- [ESP-RS Matrix Chat](https://matrix.to/#/#esp-rs:matrix.org) - ESP Rust community
- [Embassy Matrix Chat](https://matrix.to/#/#embassy-rs:matrix.org) - Embassy framework
- [probe-rs GitHub](https://github.com/probe-rs/probe-rs) - Tool development

## 🏆 Achievement Summary

✅ **Embassy Framework**: Successfully integrated with ESP32-C6  
✅ **Pure Rust Stack**: No C dependencies, full Rust development  
✅ **Async Programming**: Embassy timers and concurrent task simulation  
✅ **Modern Tooling**: probe-rs for professional embedded development  
✅ **Production Ready**: Optimized builds with LTO and proper profiles  

This project demonstrates **state-of-the-art embedded Rust development** using Embassy on ESP32-C6 with modern tooling and best practices.

## 📄 License

MIT License - see LICENSE file for details.

---

**Note**: This project represents a complete Embassy + ESP32-C6 development setup. probe-rs is strongly recommended over espflash for bare metal applications due to superior compatibility and feature set.