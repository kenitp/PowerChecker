#include "button.h"

BUTTON_MODE ButtonMode::button_mode;
bool ButtonMode::button_mode_changed;
bool ButtonMode::is_exist_sd;


void IRAM_ATTR ButtonMode::onLeftButton(void) {
    return;
}

void IRAM_ATTR ButtonMode::onMiddleButton(void) {
    int mode = static_cast<int>(ButtonMode::button_mode);
    mode++;

    // If SD card is not mounted, skip the mode which needs SD card.
    if (is_exist_sd == false) {
        if ((mode == static_cast<int>(BUTTON_MODE::POWER_IMG)) || (mode == static_cast<int>(BUTTON_MODE::PHOTO))) {
            mode++;
        }
    }
    if (mode >= static_cast<int>(BUTTON_MODE::NUM)){
        mode = static_cast<int>(BUTTON_MODE::POWER);
    }
    ButtonMode::button_mode = static_cast<BUTTON_MODE>(mode);
    ButtonMode::button_mode_changed = true;
    return;
}

void IRAM_ATTR ButtonMode::onRightButton(void) {
    return;
}

BUTTON_MODE ButtonMode::get_mode(void){
    return ButtonMode::button_mode;
}

void ButtonMode::init_mode(void){
    ButtonMode::button_mode = BUTTON_MODE::INIT;
    ButtonMode::button_mode_changed = false;
    check_sd_exist();
    return;
}

bool ButtonMode::is_changed(void) {
    bool ret = ButtonMode::button_mode_changed;
    if (ret == true) {
        ButtonMode::button_mode_changed = false;
        Serial.printf("[Debug] Mode Changed (Mode: %d)\r\n", ButtonMode::button_mode);
    }
    return ret;
}

void ButtonMode::check_sd_exist(void) {
    if (SD.open("/")) {
        ButtonMode::is_exist_sd = true;
    } else {
        ButtonMode::is_exist_sd = false;
    }
    return;
}
