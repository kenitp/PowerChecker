# PowerChecker - Server

Home Assistant のスマートメーターセンサーと SwitchBot 温湿度計からデータを取得し、REST API で配信するサーバー。

## 概要

```
[スマートメーター] --Wi-SUN--> [BP35C2] --USB--> [Home Assistant] --HTTP--> [このサーバー] --HTTP--> [クライアント]
[SwitchBot 温湿度計] ---BLE---> [SwitchBot Cloud API] --------------HTTP--------------> [このサーバー]
```

| 取得データ | 取得元 | 取得間隔 |
|---|---|---|
| 瞬時電力（W） | Home Assistant | 60秒 |
| 瞬時電流（A） | Home Assistant | 60秒 |
| 温度（℃） | SwitchBot 温湿度計 | 30秒 |
| 湿度（%） | SwitchBot 温湿度計 | 30秒 |

Wi-SUN モジュールとの通信は Home Assistant の [Smart Meter B Route](https://www.home-assistant.io/integrations/route_b_smart_meter/) 統合が担当します。本サーバーは統合が公開する以下のエンティティを参照します。

| エンティティ | 内容 |
|---|---|
| `sensor.smart_meter_power` | 瞬時電力（W） |
| `sensor.smart_meter_current_r` | R相の瞬時電流（A） |
| `sensor.smart_meter_current_t` | T相の瞬時電流（A） |

統合の既定ポーリング間隔は5分のため、値の取得前に `homeassistant.update_entity` を呼んで再計測させています。これにより Home Assistant 側の履歴も60秒間隔で更新されます。

`power_a` は R相とT相の平均値です。履歴は Home Assistant の recorder が保持します。

## 前提

- Home Assistant に Smart Meter B Route 統合が設定済みであること
  - 設定 → デバイスとサービス → 統合を追加 → Smart Meter B Route
  - 生成されたエンティティ ID を上表の名称にリネームする
- Home Assistant の長期アクセストークン
  - プロフィール → セキュリティ → 長期アクセストークンを作成
- SwitchBot 温湿度計が SwitchBot Hub 経由でクラウド連携済みであること

## セットアップ

`.env.example` をコピーして `.env` を作成し、各自の認証情報を記入します。

```bash
cp .env.example .env
```

```env
# Home Assistant
HA_BASE_URL=http://host.docker.internal:8123
HA_TOKEN=your_home_assistant_long_lived_access_token_here

# SwitchBot API credentials
SWITCHBOT_METER_DEVID=your_switchbot_meter_device_id_here
SWITCHBOT_TOKEN=your_switchbot_token_here
SWITCHBOT_SECRET=your_switchbot_secret_here
```

各設定値の取得方法：
- `HA_BASE_URL`: Home Assistant が host ネットワークで動作している場合は既定値のままでよい
- `SWITCHBOT_TOKEN` / `SWITCHBOT_SECRET`: SwitchBot アプリ → プロフィール → 設定 → 開発者向けオプション
- `SWITCHBOT_METER_DEVID`: SwitchBot アプリ → デバイス → 温湿度計のデバイスID

> **注意**: `.env` はリポジトリにコミットしないでください（`.gitignore` で除外済み）。

## 起動

```bash
./install.sh
```

`docker compose` でのビルドと起動を行います。ログの確認は `docker compose logs -f`。

開発環境で直接実行する場合：

```bash
HA_BASE_URL=http://127.0.0.1:8123 cargo run
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

```bash
curl http://192.168.1.110:3000/api/power
```

## 参考資料

- [Home Assistant Smart Meter B Route 統合](https://www.home-assistant.io/integrations/route_b_smart_meter/)
- [Home Assistant REST API](https://developers.home-assistant.io/docs/api/rest/)
- [SwitchBot API v1.1 ドキュメント](https://github.com/OpenWonderLabs/SwitchBotAPI)
