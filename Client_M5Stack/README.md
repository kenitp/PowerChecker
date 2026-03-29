# PowerChecker Client (M5Stack)

M5Stack Fire 向けの PowerChecker クライアントです。サーバーから電力情報を取得して表示するほか、時計・フォトフレーム・FTP サーバー機能を備えています。

## 機能

| モード | 説明 |
|--------|------|
| POWER | 電力値（W/A）をテキストで表示 |
| POWER_IMG | 電力レベルに応じた画像を表示 |
| CLOCK | NTP から取得した現在時刻を表示 |
| PHOTO | SD カード内の画像をスライドショー表示 |

ボタン操作でモードを切り替えます（左/中/右ボタン）。

FTP サーバー機能により、Wi-Fi 経由で SD カードへ画像を転送できます。

## 必要なもの

- [M5Stack Fire](https://docs.m5stack.com/ja/core/fire)
- [PlatformIO](https://platformio.org/)（CLI または VS Code 拡張）
- Wi-Fi 環境
- PowerChecker サーバー（`/api/power` エンドポイント）

## 設定

プロジェクトルートの `.env.example` をコピーして `.env` を作成し、環境に合わせて編集してください。

```bash
cp .env.example .env
```

```env
WIFI_SSID=your_ssid_here
WIFI_PASS=your_password_here
POWER_CHECKER_URL=http://192.168.1.x:3000/api/power
FTP_USER=M5Stack
FTP_PASS=M5Stack
```

> `.env` はリポジトリに含まれません（`.gitignore` で除外）。認証情報を誤ってコミットしないよう注意してください。

## ビルド・書き込み

### PlatformIO CLI のインストール

```bash
pip install platformio
```

### ビルドのみ

```bash
pio run
```

### M5Stack に書き込む

USB で M5Stack を接続してから実行します。

```bash
pio run --target upload
```

ポートを明示的に指定する場合:

```bash
pio run --target upload --upload-port /dev/ttyUSB0
```

### ビルド＋書き込みをまとめて実行

```bash
pio run --target upload && pio device monitor
```

### シリアルモニター（デバッグ出力確認）

```bash
pio device monitor --baud 115200
```

### ビルド成果物の削除

```bash
pio run --target clean
```

## SD カードへの画像転送

FTP サーバー機能を使って Wi-Fi 経由で画像を転送できます。

- **ホスト**: M5Stack の IP アドレス（起動時に LCD に表示）
- **ユーザー名**: `M5Stack`（`.env` の `FTP_USER`）
- **パスワード**: `M5Stack`（`.env` の `FTP_PASS`）

電力レベル別の画像は SD カードの以下のディレクトリに配置してください（140×184px）。

```
/img/power/low/   # 低電力時
/img/power/mid/   # 中電力時
/img/power/high/  # 高電力時
```

## 依存ライブラリ

| ライブラリ | バージョン |
|------------|------------|
| M5Stack | ^0.4.3 |
| ArduinoJson | ^6.19.4 |
| ESP32FTPServer | ^0.0.2 |

依存関係は `platformio.ini` に記載されており、ビルド時に自動でインストールされます。

## プロジェクト構成

```
Client_M5Stack/
├── src/
│   ├── main.cpp              # エントリーポイント・タスク起動
│   ├── config.cpp            # Wi-Fi / URL / FTP 設定
│   ├── button_mode/          # ボタン割り込み・モード管理
│   ├── clock/                # 時計表示タスク
│   ├── ftp_server/           # FTP サーバータスク
│   ├── photo_frame/          # フォトフレームタスク
│   ├── power_checker/        # 電力取得・表示タスク
│   └── utils/                # Wi-Fi 接続・ディスプレイ共通処理
├── include/                  # 公開ヘッダー
├── lib/ESP32FTPServer/       # FTP サーバーライブラリ（サブモジュール）
└── platformio.ini            # PlatformIO 設定
```
