#!/bin/bash

# ESP32-C6 Flash and Monitor Script
echo "ESP32-C6 Flash and Monitor Script"
echo "=================================="

# Check if firmware exists
if [ ! -f "target/riscv32imac-unknown-none-elf/release/altruist-rs" ]; then
    echo "Error: Firmware not found. Please run 'cargo build --release' first."
    exit 1
fi

# Try to flash and monitor
echo "Flashing firmware to ESP32-C6..."
espflash flash --monitor target/riscv32imac-unknown-none-elf/release/altruist-rs

# Alternative: Just monitor if already flashed
if [ $? -ne 0 ]; then
    echo ""
    echo "If flashing failed due to permissions, you can either:"
    echo "1. Run: sudo chmod 666 /dev/ttyACM0"
    echo "2. Add your user to dialout group: sudo usermod -a -G dialout $USER"
    echo "3. Run this script with sudo: sudo ./flash_and_monitor.sh"
    echo ""
    echo "To just monitor (if already flashed): espflash monitor"
fi