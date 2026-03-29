# PowerChecker - Server (BP35C2 / BP35C0)

スマートメーター（Bルート）と SwitchBot 温湿度計からデータを取得し、REST API で配信するサーバー。

## 概要

```
[スマートメーター] --Wi-SUN--> [BP35C2/BP35C0] --USB--> [このサーバー] --HTTP--> [クライアント]
[SwitchBot 温湿度計] ---BLE---> [SwitchBot Cloud API] -----HTTP-----> [このサーバー]
```

| 取得データ | 取得元 | 取得間隔 |
|---|---|---|
| 瞬時電力（W） | スマートメーター（ECHONET Lite） | 60秒 |
| 瞬時電流（A） | スマートメーター（ECHONET Lite） | 60秒 |
| 温度（℃） | SwitchBot 温湿度計 | 30秒 |
| 湿度（%） | SwitchBot 温湿度計 | 30秒 |

## ハードウェア要件

- **Wi-SUN モジュール**: ROHM BP35C2 または BP35C0
  - USB 接続（FTDI FT230X チップ搭載）
- **スマートメーター**: Bルートサービス加入済みのもの
  - 電力会社への申し込みで Bルート ID / パスワードを取得
- **SwitchBot 温湿度計**: SwitchBot Hub 経由でクラウド連携済みのもの

## セットアップ

### 1. udev ルールの設定（USB デバイス名の固定）

```bash
sudo cp 99-com.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

BP35C2 を接続すると `/dev/ttyUSB_power` として認識されます。

### 2. ユーザーをシリアルポートグループに追加

```bash
sudo usermod -aG dialout $USER
# ログアウト・再ログインして反映
```

### 3. 環境変数の設定

`.env.example` をコピーして `.env` を作成し、各自の認証情報を記入します。

```bash
cp .env.example .env
```

`.env` の内容：

```env
# デバイス設定
DEVICE_PATH=/dev/ttyUSB_power

# REST API サーバー設定
SERVER_IP=192.168.1.110
SERVER_PORT=3000

# Bルート認証情報（スマートメーター）
B_ROUTE_ID=your_b_route_id_here
B_ROUTE_PASS=your_b_route_password_here

# SwitchBot 設定
SWITCHBOT_TOKEN=your_switchbot_api_token_here
SWITCHBOT_METER_DEVID=your_switchbot_meter_device_id_here
```

各設定値の取得方法：
- `B_ROUTE_ID` / `B_ROUTE_PASS`: 契約電力会社のBルートサービスに申し込んで取得
- `SWITCHBOT_TOKEN`: SwitchBot アプリ → プロフィール → 開発者向けオプション
- `SWITCHBOT_METER_DEVID`: SwitchBot アプリ → デバイス → 温湿度計のデバイスID

> **注意**: `.env` はリポジトリにコミットしないでください（`.gitignore` で除外済み）。

### 4. ビルド

```bash
cargo build --release
```

### 5. 開発環境での実行

```bash
cargo run
```

または：

```bash
./target/release/power-checker
```

## systemd サービスとしてインストール

`install.sh` を実行するとビルド・インストール・サービス登録を一括で行います。

```bash
./install.sh
```

スクリプトは以下を実行します：
1. `cargo build --release`
2. バイナリを `/usr/local/bin/power-checker` にコピー
3. `.env` を `/etc/power_checker.env` にコピー（パーミッション 600）
4. `power_checker.service` を `/lib/systemd/system/` にコピー
5. サービスを有効化・起動

サービスの操作：

```bash
sudo systemctl status power_checker
sudo systemctl stop power_checker
sudo systemctl start power_checker
sudo journalctl -u power_checker -f   # ログの確認
```

## API

### GET /api/power

現在の電力・環境データを返します。

**レスポンス例：**

```json
{
  "power_w": "350",
  "power_a": "1.5",
  "temperature": "23.5",
  "humidity": "55"
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `power_w` | string | 瞬時電力（W） |
| `power_a` | string | 瞬時電流（A、小数点1桁） |
| `temperature` | string | 温度（℃、小数点1桁） |
| `humidity` | string | 湿度（%） |

**curl による確認：**

```bash
curl http://192.168.1.110:3000/api/power
```

## 参考資料

-   **BP35C2 リファレンス**
    -   [ROHM 公式ページ](https://www.rohm.co.jp/products/wireless-communication/specified-low-power-radio-modules/bp35c0-product#designResources)
-   **ECHONET Lite 通信プロトコル**
    -   [ECHONET 公式ページ](https://echonet.jp/spec_g/)
        -   ECHONET Lite 規格書 Ver.X.XX（日本語版） 第 2 部 ECHONET Lite 通信ミドルウェア仕様
        -   APPENDIX ECHONET 機器オブジェクト詳細規定 Release N
-   **SwitchBot API**
    -   [SwitchBot API v1.0 ドキュメント](https://github.com/OpenWonderLabs/SwitchBotAPI)
