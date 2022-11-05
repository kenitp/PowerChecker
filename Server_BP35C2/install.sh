#!/bin/bash

cargo build --release
sudo systemctl stop power_checker.service
sudo systemctl disable power_checker.service
sudo cp ./target/release/power-checker /usr/local/bin/
sudo cp power_checker.service /lib/systemd/system/
#sudo cp 99-com.rules /etc/udev/rules.d/
#sudo udevadm control --reload-rules
sudo systemctl enable power_checker.service
sudo systemctl start power_checker.service
