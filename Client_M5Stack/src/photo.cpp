#include "photo.h"

void taskPhoto(void *args) {
    Serial.println("[Debug] taskPhoto Start");
    while(true) {
        if (get_button_mode() == MODE_PHOTO) {
            if (is_mode_changed()) {
                resetDisplay();
                M5.Lcd.drawJpgFile(SD,"/img/img002.jpg");;
            }
        }
        delay(1000);
    }
    return;
}