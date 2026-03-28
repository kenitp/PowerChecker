# Power Checker Client

M5Stack Fire（ESP32）向けの電力表示クライアントです。元の C++ / PlatformIO プロジェクトを **Rust** と **Slint** で書き直したものです。

## 機能

- **電力モード**: Power Checker サーバの HTTP API から電力（W）などを取得して表示
- **時計モード**: NTP で時刻同期し、デジタル時計を表示
- **フォトモード**: SD カードの画像を表示（フォトフレーム）

表示は ILI9341（320×240）を想定しています。

## ハードウェア

| 項目 | 内容 |
|------|------|
| ボード | M5Stack Fire（ESP32） |
| ディスプレイ | ILI9341 320×240（SPI） |
| ボタン | A/B/C（いずれもアクティブ Low） |

### ボタン割り当て

| ボタン | GPIO | 役割 |
|--------|------|------|
| A | 39 | 左（予約） |
| B | 38 | 表示モードの切り替え |
| C | 37 | データの強制再取得 |

## 前提条件

1. **ESP32（Xtensa）用 Rust ツールチェーン**  
   ターゲット `xtensa-esp32-none-elf` でビルドできる環境（例: [esp-rs book](https://docs.esp-rs.org/book/) の手順に従った `espup` など）。

2. **書き込み・モニタ**  
   `espflash` が利用できること（`.cargo/config.toml` で `runner = "espflash flash --monitor"` を使用）。

3. **Slint**  
   [crates.io の `slint` / `slint-build`](https://crates.io/crates/slint) から取得します（`Cargo.toml` のバージョンに従う）。ローカルクローンは不要です。

4. **`build-std`**  
   `.cargo/config.toml` で `build-std = ["alloc", "core"]` を使っているため、対応した nightly / esp ツールチェーンが必要です。

## 設定

ビルド時に次の環境変数が埋め込まれます（`env!`）。

| 変数 | 説明 |
|------|------|
| `WIFI_SSID` | 接続する Wi-Fi の SSID |
| `WIFI_PASS` | Wi-Fi のパスワード |
| `POWER_CHECKER_URL` | 電力 API の完全 URL（**実行時はリテラル IPv4 のみ**。DNS は使わない） |

### `.env` で設定する（推奨）

1. `.env.example` を `.env` にコピーする。  
2. 値を編集する。  
3. `build.rs` が `dotenv` クレートで `.env` を読み、`cargo:rustc-env` でビルドに渡す。

```bash
cp .env.example .env
# .env を編集
cargo build --release
```

### `.cargo/config.toml` の `[env]` で設定する

`WIFI_SSID` などを `[env]` に書いても同様にビルドに反映できます（`main.rs` 先頭のコメント参照）。

## ビルドと書き込み

プロジェクトルートで:

```bash
cargo build --release
cargo run --release   # espflash でフラッシュ＆モニタ（runner 設定時）
```

デフォルトターゲットは `.cargo/config.toml` の `xtensa-esp32-none-elf` です。

## プロジェクト構成（抜粋）

| パス | 内容 |
|------|------|
| `src/main.rs` | ESP32 初期化、Wi-Fi、HTTP、NTP、Slint プラットフォーム実装 |
| `ui/app.slint` | UI 定義（モード切替・テーマなど） |
| `build.rs` | `dotenv` で `.env` 読み込み、Slint のコンパイル |

## 依存クレート（概要）

- **UI**: Slint（ソフトウェアレンダラ、`no_std` 向け設定）
- **MCU**: esp-hal、esp-wifi、esp-alloc など
- **表示**: mipidsi（ILI9341）、embedded-graphics
- **通信**: smoltcp（TCP 等）、serde / serde-json-core

詳細は `Cargo.toml` を参照してください。
