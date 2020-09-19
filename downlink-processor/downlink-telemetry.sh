#!/bin/bash
source .env
stty -F $SERIAL_PORT raw
cargo run