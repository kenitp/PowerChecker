#pragma once
#include <Arduino.h>

// ピクセルシフト：焼き付き防止のため一定時間ごとに表示位置を微小移動する
class PixelShift {
public:
    // ループから定期的に呼ぶ。5分ごとにシフト位置を更新する
    static void tick(void);
    static int8_t getX(void);
    static int8_t getY(void);

    static const uint32_t SHIFT_INTERVAL_MS = 5UL * 60UL * 1000UL; // 5分

private:
    static const int8_t SHIFTS[][2];
    static const uint8_t NUM_SHIFTS;
    static uint8_t current_index;
    static unsigned long last_update_ms;
};
