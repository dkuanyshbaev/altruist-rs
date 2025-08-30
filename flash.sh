#!/bin/bash
# Build and flash ESP32C6 firmware with esptool

echo "Building firmware..."
cargo build --release

echo "Flashing bootloader, partition table, and application..."
esptool --chip esp32c6 --port /dev/ttyACM0 --baud 460800 write_flash \
    0x0 bootloader-esp32c6.bin \
    0x8000 partition-table-fixed.bin \
    0x10000 target/riscv32imac-unknown-none-elf/release/altruist-rs

echo "Firmware flashed successfully! To monitor output, run: ./log.sh"