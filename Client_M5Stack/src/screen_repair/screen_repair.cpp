#include "screen_repair_int.h"

// 焼き付き（残像）修復：液晶分子の固定化を解消するため、
// 全画面を様々なパターンで高速に駆動して画素を揺さぶる。
// 1サイクルは「原色サイクル」→「白黒高速反転」の2フェーズで構成する。

// フェーズ1：塗りつぶしに使用する色の並び
static const uint16_t REPAIR_COLORS[] = {
    TFT_WHITE,
    TFT_RED,
    TFT_GREEN,
    TFT_BLUE,
    TFT_BLACK,
};
static const uint8_t REPAIR_COLOR_NUM = sizeof(REPAIR_COLORS) / sizeof(REPAIR_COLORS[0]);

// フェーズ1：各色を表示する時間（ミリ秒）
static const uint32_t COLOR_INTERVAL_MS = 500;
// フェーズ1：原色サイクルを繰り返す回数
static const uint8_t COLOR_CYCLE_REPEAT = 3;

// フェーズ2：白黒反転の切替間隔（ミリ秒）
static const uint32_t FLASH_INTERVAL_MS = 120;
// フェーズ2：白黒反転の回数（白＋黒で1回）
static const uint16_t FLASH_REPEAT = 30;

// 修復中の輝度（残像解消のため最大にする）
static const uint8_t REPAIR_BRIGHTNESS = 255;
// 通常時の輝度（setup と揃える）
static const uint8_t NORMAL_BRIGHTNESS = 1;

// モード継続中のみ true。途中でモードが変わったら false を返す
static bool stillRepairing(void) {
    return (ButtonMode::getMode() == BUTTON_MODE::REPAIR);
}

// 指定時間 delay しつつ、モードが変わったら即座に false を返す
static bool repairDelay(uint32_t ms) {
    const uint32_t step = 20;
    for (uint32_t elapsed = 0; elapsed < ms; elapsed += step) {
        if (!stillRepairing()) {
            return false;
        }
        delay(step);
    }
    return true;
}

// フェーズ1：原色を順番に塗りつぶす
static bool runColorPhase(void) {
    for (uint8_t cycle = 0; cycle < COLOR_CYCLE_REPEAT; cycle++) {
        for (uint8_t i = 0; i < REPAIR_COLOR_NUM; i++) {
            if (!stillRepairing()) {
                return false;
            }
            M5.Lcd.fillScreen(REPAIR_COLORS[i]);
            if (!repairDelay(COLOR_INTERVAL_MS)) {
                return false;
            }
        }
    }
    return true;
}

// フェーズ2：白と黒を高速に反転させる
static bool runFlashPhase(void) {
    for (uint16_t i = 0; i < FLASH_REPEAT; i++) {
        if (!stillRepairing()) {
            return false;
        }
        M5.Lcd.fillScreen(TFT_WHITE);
        if (!repairDelay(FLASH_INTERVAL_MS)) {
            return false;
        }
        M5.Lcd.fillScreen(TFT_BLACK);
        if (!repairDelay(FLASH_INTERVAL_MS)) {
            return false;
        }
    }
    return true;
}

void taskScreenRepair(void *args) {
    Serial.println("[Debug] taskScreenRepair Start");
    bool active = false;

    while (true) {
        if (stillRepairing()) {
            if (!active) {
                // 修復モードに入った：輝度を最大に上げる
                active = true;
                ButtonMode::isChanged();
                M5.Lcd.setBrightness(REPAIR_BRIGHTNESS);
            }

            // 強化修復シーケンスを実行（途中でモードが変わったら中断）
            if (runColorPhase()) {
                runFlashPhase();
            }
        } else {
            if (active) {
                // 修復モードを抜けた：輝度と画面を元に戻す
                active = false;
                M5.Lcd.setBrightness(NORMAL_BRIGHTNESS);
                M5.Lcd.fillScreen(TFT_BLACK);
            }
            delay(500);
        }
    }
    return;
}
