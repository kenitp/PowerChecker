#!/bin/bash

set -e

echo "=== PowerChecker Docker Installation ==="

# 1. udevルールのセットアップ（未適用または変更がある場合のみ実行）
if [ ! -f /etc/udev/rules.d/99-com.rules ] || ! cmp -s 99-com.rules /etc/udev/rules.d/99-com.rules; then
    echo "Setting up udev rules for serial port..."
    sudo cp 99-com.rules /etc/udev/rules.d/
    echo "Reloading udev rules..."
    sudo udevadm control --reload-rules
    sudo udevadm trigger
    echo "udev rules applied. (/dev/ttyUSB_power will be created when the device is connected)"
else
    echo "udev rules are already up to date. (Skipping sudo udev configuration)"
fi

# 2. 環境変数ファイルのチェック
if [ ! -f .env ]; then
    echo "ERROR: .env file not found."
    if [ -f .env.example ]; then
        echo "Creating .env from .env.example. Please configure it with your credentials before running this script again."
        cp .env.example .env
    fi
    exit 1
fi

# 3. Dockerコンテナのビルドと起動
echo "Starting Docker containers..."
if docker compose version &> /dev/null; then
    docker compose down
    docker compose up --build -d
elif command -v docker-compose &> /dev/null; then
    docker-compose down
    docker-compose up --build -d
else
    echo "ERROR: Docker Compose is not installed. Please install docker-compose or docker-compose-plugin."
    exit 1
fi

echo "=== Installation Completed Successfully! ==="
echo "You can check the logs using: docker compose logs -f"
