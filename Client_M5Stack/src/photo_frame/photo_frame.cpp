#include "photo_frame_int.h"

void taskPhoto(void *args) {
    Serial.println("[Debug] taskPhoto Start");
    while(true) {
        if (ButtonMode::get_mode() == BUTTON_MODE::PHOTO) {
            if (ButtonMode::is_changed()) {
                resetDisplay();
                M5.Lcd.drawJpgFile(SD,"/img/img002.jpg");;
            }
        }
        delay(1000);
    }
    return;
}