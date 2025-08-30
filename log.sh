#!/bin/bash
# Monitor ESP32C6 serial output

echo "Monitoring ESP32C6 logs on /dev/ttyACM0..."
echo "Press Ctrl+C to stop monitoring"  
echo "----------------------------------------"

# Configure the serial port and read from it
stty -F /dev/ttyACM0 115200 cs8 -cstopb -parenb
cat /dev/ttyACM0