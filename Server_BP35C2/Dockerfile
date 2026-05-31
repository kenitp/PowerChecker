# ビルドステージ
FROM rust:slim-bookworm AS builder

# serialport クレートや openssl のコンパイルに必要なパッケージをインストール
RUN apt-get update && apt-get install -y \
    pkg-config \
    libudev-dev \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/power-checker
COPY . .

# リリースビルドの作成
RUN cargo build --release

# 実行ステージ
FROM debian:bookworm-slim

# 実行時に必要な共有ライブラリと証明書をインストール
RUN apt-get update && apt-get install -y \
    libudev1 \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# ビルドしたバイナリをコピー
COPY --from=builder /usr/src/power-checker/target/release/power-checker /usr/local/bin/power-checker

# アプリケーションを実行するディレクトリ
WORKDIR /app

# ポートの公開 (デフォルトは 3000)
EXPOSE 3000

CMD ["power-checker"]
