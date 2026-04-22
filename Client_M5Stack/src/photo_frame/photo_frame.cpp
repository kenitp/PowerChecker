#include "photo_frame_int.h"
#include "pixel_shift.h"

void taskPhoto(void *args) {
    Serial.println("[Debug] taskPhoto Start");
    unsigned long last_draw_ms = 0;
    while(true) {
        if (ButtonMode::getMode() == BUTTON_MODE::PHOTO) {
            bool force = ButtonMode::isChanged();
            unsigned long now = millis();
            // モード切替時、またはシフト更新間隔ごとに再描画
            if (force || (now - last_draw_ms >= PixelShift::SHIFT_INTERVAL_MS)) {
                last_draw_ms = now;
                resetDisplay();
                M5.Lcd.drawJpgFile(SD, "/img/img002.jpg",
                    PixelShift::getX(), PixelShift::getY());
            }
        }
        delay(1000);
    }
    return;
}