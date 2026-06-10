#include "m5_device.h"

uint8_t deviceNormalBrightness(void) {
    switch (M5.getBoard()) {
    case m5::board_t::board_M5StackCore2:
    case m5::board_t::board_M5Tough:
        // Core2 は AXP 経由のバックライト制御のため、Fire 向けの 1 では暗すぎる
        return 64;
    default:
        // Fire: 焼き付き対策のため通常は極暗
        return 1;
    }
}

bool deviceInitSD(void) {
    // M5Stack 系は SD CS = GPIO4。MISO は機種で異なる（Fire=19 / Core2=38）。
    const int cs = GPIO_NUM_4;
    int sck = 18;
    int mosi = 23;
    int miso = 19;

    switch (M5.getBoard()) {
    case m5::board_t::board_M5StackCore2:
    case m5::board_t::board_M5Tough:
        miso = 38;
        break;
    default:
        miso = 19;
        break;
    }

    SPI.begin(sck, miso, mosi, cs);

    // 初回マウントに失敗することがあるため数回リトライする
    for (int i = 0; i < 5; i++) {
        if (SD.begin(cs, SPI, 25000000)) {
            return true;
        }
        Serial.printf("[Debug] SD.begin retry %d\r\n", i + 1);
        delay(200);
    }
    return false;
}
