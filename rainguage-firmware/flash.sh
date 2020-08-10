#!/bin/sh
set -e
cargo objcopy --bin rainguage_downlink_firmware --release -- -O binary target/rainguage_downlink_firmware.bin
~/.arduino15/packages/arduino/tools/bossac/1.7.0/bossac -i -d --port=ttyACM1 -U true -i -e -w -v target/rainguage_downlink_firmware.bin -R
