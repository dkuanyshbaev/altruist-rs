#!/bin/bash

echo "Altruist logs on /dev/ttyACM0"
echo "Press Ctrl+C to stop"
echo "----------------------------------------"

cat /dev/ttyACM0 | sed -E \
    -e 's/\[SDS011\]/\x1b[34m[SDS011]\x1b[0m/g' \
    -e 's/\[BME280\]/\x1b[32m[BME280]\x1b[0m/g' \
    -e 's/\[ME2-CO\]/\x1b[36m[ME2-CO]\x1b[0m/g' \
    -e 's/\[AGGREGATOR\]/\x1b[35m[AGGREGATOR]\x1b[0m/g' \
    -e 's/.*(error|timeout|Error|ERROR|backing off).*/\x1b[31m&\x1b[0m/g'
