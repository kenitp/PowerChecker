#!/bin/bash

set -e

echo "=== PowerChecker Docker Installation ==="

# 1. 環境変数ファイルのチェック
if [ ! -f .env ]; then
    echo "ERROR: .env file not found."
    if [ -f .env.example ]; then
        echo "Creating .env from .env.example. Please configure it with your credentials before running this script again."
        cp .env.example .env
    fi
    exit 1
fi

# 2. Dockerコンテナのビルドと起動
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
