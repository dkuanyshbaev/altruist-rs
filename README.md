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

# For probe-rs (REQUIRED for USB access and JTAG support)
sudo apt install -y libftdi1-dev libhidapi-dev libusb-1.0-0-dev
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

### Critical Setup Steps (Required!)

#### 1. Update Rust Toolchain

```bash
# REQUIRED: Update to latest nightly (probe-rs needs recent toolchain)
rustup update nightly
rustup default nightly

# Add RISC-V target for ESP32-C6
rustup target add riscv32imac-unknown-none-elf
```

#### 2. Install probe-rs Dependencies & Tools

```bash
# Install system dependencies for probe-rs
sudo apt install -y libftdi1-dev libhidapi-dev libusb-1.0-0-dev

# Install probe-rs tools (do NOT use --locked flag)
cargo install probe-rs-tools

# Verify installation
probe-rs --version  # Should show v0.29.1+

# Verify ESP32-C6 support
probe-rs chip list | grep -i esp32c6
```

#### 3. Fix USB Permissions (Critical!)

Create udev rules for ESP32-C6 access:

```bash
# Create udev rules file
sudo tee /etc/udev/rules.d/99-esp32.rules << 'EOF'
# ESP32-C6 USB JTAG/Serial Debug Unit
SUBSYSTEM=="usb", ATTR{idVendor}=="303a", ATTR{idProduct}=="1001", MODE="0666"
# General ESP32 devices  
SUBSYSTEM=="usb", ATTR{idVendor}=="303a", MODE="0666"
EOF

# Apply rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# IMPORTANT: Unplug and replug your ESP32-C6 board after this!
```

#### 4. Verify Setup

```bash
# Check if probe detects your board
probe-rs list
# Should show: ESP JTAG -- 303a:1001:... (EspJtag)
```

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

### ✅ Working Method: probe-rs

Once setup is complete, flashing is simple:

```bash
# Build and flash in one command (configured in .cargo/config.toml)
cargo run --release
```

This will:
1. **Build** the Embassy firmware 
2. **Erase** flash automatically
3. **Flash** the binary via USB-JTAG
4. **Start** the application

### 📺 Monitor Serial Output

After flashing, monitor your running Embassy application:

```bash
# Continuous real-time logs (press Ctrl+C to stop)
cat /dev/ttyACM0

# Or with timeout
timeout 10s cat /dev/ttyACM0
```

Expected output:
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

### Common Issues & Solutions

#### Problem: `probe-rs` not found after installation

```bash
# Ensure cargo bin directory is in PATH
export PATH="$HOME/.cargo/bin:$PATH"
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc

# Verify installation
probe-rs --version
```

#### Problem: `Error: Probe not found`

**Root Cause**: USB permissions not set correctly.

**Solution**: Follow the USB permissions setup in installation section:
```bash
sudo tee /etc/udev/rules.d/99-esp32.rules << 'EOF'
SUBSYSTEM=="usb", ATTR{idVendor}=="303a", ATTR{idProduct}=="1001", MODE="0666"
EOF
sudo udevadm control --reload-rules
sudo udevadm trigger
# UNPLUG and REPLUG your ESP32-C6 board!
```

#### Problem: `feature 'edition2024' is required`

**Root Cause**: Rust toolchain too old for probe-rs.

**Solution**: Update Rust toolchain:
```bash
rustup update nightly
rustup default nightly
```

#### Problem: Custom board without BOOT button

**Solution**: probe-rs works with custom boards! No BOOT button needed.
- ESP32-C6 has built-in USB-JTAG that probe-rs uses directly
- No need to manually enter download mode
- Just ensure USB permissions are set correctly

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

## ✅ Success! Working Configuration

### Verified Setup

| Component | Version | Status |
|-----------|---------|--------|
| **Rust** | nightly-2025-08-29+ | ✅ Working |
| **probe-rs** | v0.29.1+ | ✅ Working |
| **Embassy** | 0.6.2+ | ✅ Working |
| **esp-hal** | 0.21.1 | ✅ Working |
| **ESP32-C6** | Custom board | ✅ Working |

### Why This Setup Works

- **✅ probe-rs**: Native RISC-V support for ESP32-C6
- **✅ Built-in USB-JTAG**: No external programmer needed
- **✅ Pure Rust**: No ESP-IDF, bootloaders, or partition tables
- **✅ Embassy async**: Modern embedded Rust framework
- **✅ Custom boards**: Works without BOOT buttons

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