#!/bin/sh
sudo apt-get install libusb-dev
rustup target add thumbv6m-none-eabi
cargo install cargo-hf2

# Setup udev rules and access to modem
#wget https://github.com/adafruit/Trinket_Arduino_Linux/raw/master/99-adafruit-boards.rules
#sudo cp ./99-adafruit-boards.rules /etc/udev/rules.d/
#sudo service udev restart
#sudo addgroup $USER adm

# Grab the GCC/ARM tools
#cargo install cargo-binutils
#rustup component add llvm-tools-preview
#mkdir $HOME/opt
#wget "https://developer.arm.com/-/media/Files/downloads/gnu-rm/8-2018q4/gcc-arm-none-eabi-8-2018-#q4-major-linux.tar.bz2" -O gcc.tar.bz2
#bunzip2 gcc.tar.bz2
#tar xf gcc.tar $HOME/opt/gcc-arm

echo ""
echo Manual Steps
echo ""
echo "1. Install arduino (www.arduino.cc)"
echo "2. In arduino, File, Preferences, add this board support"
echo "       https://adafruit.github.io/arduino-board-index/package_adafruit_index.json "
echo "3. in arduino, Tools, Boards, Board Manager"
echo "       Add Adafruit SAMD Board"
echo "99. logout/login for group access"
