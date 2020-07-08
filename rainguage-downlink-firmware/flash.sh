#!/bin/sh
set -e
cargo objcopy --bin rainguage_downlink_firmware --release -- -O binary target/rainguage_downlink_firmware.bin
#stty -F /dev/ttyACM0 ospeed 1200
~/.arduino15/packages/arduino/tools/bossac/1.7.0/bossac -i -d --port=ttyACM0 -U true -i -e -w -v target/rainguage_downlink_firmware.bin -R
