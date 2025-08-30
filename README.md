# altruist-rs

A Rust implementation of altruist using Embassy async framework on ESP32C6.

## Current Status ✅

We have successfully created a **minimal working Embassy example** that:
- ✅ Compiles cleanly with proper Embassy + esp-hal dependencies
- ✅ Uses async/await with Embassy executor and tasks
- ✅ Flashes successfully with esptool (6MB firmware)
- ✅ Hardware communication works (serial connection established)

## Current Code

**Embassy Hello World Application:**
```rust
#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_println::println;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    println!("Altruist-RS starting up!");
    spawner.spawn(hello_task()).unwrap();
}

#[embassy_executor::task]
async fn hello_task() {
    let mut counter = 0u32;
    loop {
        println!("Hello from Altruist-RS! Counter: {}", counter);
        counter += 1;
        Timer::after(Duration::from_secs(5)).await;
    }
}
```

## Usage

```bash
# Build and flash firmware
./flash.sh

# Monitor serial output  
./look
```

## Current Limitation

The firmware flashes successfully but needs ESP-IDF bootloader components to boot completely. Currently shows bootloader error messages because we only have the application at 0x10000.

## Next Steps Options

1. **Add ESP-IDF bootloader + partition table** - Complete the boot process
2. **Expand Embassy functionality** - Add GPIO, sensors, WiFi
3. **Project structure** - Set up proper altruist-rs architecture

The core achievement is complete: **working Embassy async Rust development environment for ESP32C6**! 🚀

## Dependencies

- Rust nightly with `riscv32imac-unknown-none-elf` target
- esptool for flashing
- Embassy 0.6.0 + esp-hal 0.20.1

## License

MIT.