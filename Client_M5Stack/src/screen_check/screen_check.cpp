#include "screen_check_int.h"
#include "m5_device.h"

// 焼き付き確認：暗〜中間の均一な単色をバックライト最大で表示する。
// 残像は画素の透過率のわずかな差として現れるため、暗めのグレーや紺だと
// 真っ白・真っ黒よりも明暗差が浮き上がって見つけやすい。
// 右ボタンでテスト色を順番に切り替えられる。

// RGB565 のグレーを生成（v は 0-255）
static constexpr uint16_t gray565(uint8_t v) {
    return ((v >> 3) << 11) | ((v >> 2) << 5) | (v >> 3);
}

struct CheckColor {
    uint16_t color;
    const char* name;
};

// 確認用テスト色（暗めを優先的に並べる）
static const CheckColor CHECK_COLORS[] = {
    { gray565(32),  "Dark Gray" },
    { gray565(64),  "Gray" },
    { gray565(128), "Mid Gray" },
    { TFT_NAVY,     "Navy" },
    { TFT_MAROON,   "Dark Red" },
    { TFT_DARKGREEN,"Dark Green" },
    { TFT_WHITE,    "White" },
    { TFT_BLACK,    "Black" },
};
static const uint8_t CHECK_COLOR_NUM = sizeof(CHECK_COLORS) / sizeof(CHECK_COLORS[0]);

// 通常時の輝度（setup と揃える）
static const uint8_t CHECK_BRIGHTNESS_MAX = 255;

static uint8_t normalBrightness(void) {
    return deviceNormalBrightness();
}

static void drawCheckColor(uint8_t index) {
    M5.Display.fillScreen(CHECK_COLORS[index].color);
    Serial.printf("[Debug] ScreenCheck color: %s\r\n", CHECK_COLORS[index].name);
}

void taskScreenCheck(void *args) {
    Serial.println("[Debug] taskScreenCheck Start");
    bool active = false;
    uint8_t color_index = 0;

    while (true) {
        if (ButtonMode::getMode() == BUTTON_MODE::CHECK) {
            if (!active) {
                // 確認モードに入った：輝度を上げ、最初の色を表示
                active = true;
                color_index = 0;
                ButtonMode::isChanged();
                ButtonMode::needRefresh();
                M5.Display.setBrightness(CHECK_BRIGHTNESS_MAX);
                drawCheckColor(color_index);
            }

            // 右ボタンで次のテスト色へ
            if (ButtonMode::needRefresh()) {
                color_index = (color_index + 1) % CHECK_COLOR_NUM;
                drawCheckColor(color_index);
            }
            delay(100);
        } else {
            if (active) {
                // 確認モードを抜けた：輝度と画面を元に戻す
                active = false;
                M5.Display.setBrightness(normalBrightness());
                M5.Display.fillScreen(TFT_BLACK);
            }
            delay(500);
        }
    }
    return;
}
