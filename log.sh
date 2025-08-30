#!/bin/bash
# Monitor Embassy ESP32-C6 serial output
# Usage: ./log.sh

echo "Monitoring ESP32-C6 Embassy logs on /dev/ttyACM0"
echo "Press Ctrl+C to stop monitoring"
echo "----------------------------------------"

# Configure serial port properly and start monitoring
stty -F /dev/ttyACM0 115200 raw -echo && cat /dev/ttyACM0