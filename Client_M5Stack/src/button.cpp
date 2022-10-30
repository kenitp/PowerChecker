#include "button.h"

static BUTTON_MODE button_mode = MODE_INIT;
static bool button_mode_changed = false;
static bool is_exist_sd = true;

// Button 割込み Handler
void IRAM_ATTR onLeftButton(void) {
    return;
}

 void IRAM_ATTR onMiddleButton(void) {
    int mode = (int)button_mode;
    mode++;

    // If SD card is not mounted, skip the mode which needs SD card.
    if (is_exist_sd == false) {
        if ((mode == MODE_POWER_IMG) || (mode == MODE_PHOTO)) {
            mode++;
        }
    }
    if (mode >= MODE_NUM){
        mode = MODE_POWER;
    }
    button_mode = (BUTTON_MODE)mode;
    button_mode_changed = true;
    return;
}

void IRAM_ATTR onRightButton(void) {
    return;
}


// 公開関数
BUTTON_MODE get_button_mode(void){
    return button_mode;
}

void init_button_mode(void){
    button_mode = MODE_INIT;
    if (SD.open("/")) {
        is_exist_sd = true;
    } else {
        is_exist_sd = false;
    }
    return;
}

bool is_mode_changed(void) {
    bool ret = button_mode_changed;
    if (ret == true) {
        button_mode_changed = false;
        Serial.printf("[Debug] Mode Changed (Mode: %d)\r\n", button_mode);
    }
    return ret;
}