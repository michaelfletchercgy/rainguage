#!/bin/sh
PORT=/dev/ttyACM0
stty -F /dev/ttyACM0 raw
cargo run