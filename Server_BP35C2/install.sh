#!/bin/bash

cargo build --release
sudo systemctl stop power_checker.service
sudo systemctl disable power_checker.service
sudo cp ./target/release/power-checker /usr/local/bin/
sudo cp power_checker.service /lib/systemd/system/
if [ -f .env ]; then
    sudo cp .env /etc/power_checker.env
    sudo chmod 600 /etc/power_checker.env
else
    echo "WARNING: .env file not found. Create /etc/power_checker.env manually before starting the service."
fi
#sudo cp 99-com.rules /etc/udev/rules.d/
#sudo udevadm control --reload-rules
sudo systemctl enable power_checker.service
sudo systemctl start power_checker.service
