#!/bin/bash
# Build and flash with system esptool

# Build the firmware
echo "Building firmware..."
cargo build --release

# Flash with system esptool
echo "Flashing firmware..."
esptool --chip esp32c6 --port /dev/ttyACM0 write_flash 0x10000 target/riscv32imac-unknown-none-elf/release/altruist-rs

echo "Flashing complete! To monitor output, run: cat /dev/ttyACM0"