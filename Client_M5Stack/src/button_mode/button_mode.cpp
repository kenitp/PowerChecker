#include "button_mode_int.h"

BUTTON_MODE ButtonMode::buttonMode;
bool ButtonMode::buttonModeChanged;
bool ButtonMode::isExistSD;
bool ButtonMode::isRefresh;


void IRAM_ATTR ButtonMode::onLeftButton(void) {
    return;
}

void IRAM_ATTR ButtonMode::onMiddleButton(void) {
    int mode = static_cast<int>(ButtonMode::buttonMode);
    mode++;

    // If SD card is not mounted, skip the mode which needs SD card.
    if (isExistSD == false) {
        if ((mode == static_cast<int>(BUTTON_MODE::POWER_IMG)) || (mode == static_cast<int>(BUTTON_MODE::PHOTO))) {
            mode++;
        }
    }
    if (mode >= static_cast<int>(BUTTON_MODE::NUM)){
        mode = static_cast<int>(BUTTON_MODE::POWER);
    }
    ButtonMode::buttonMode = static_cast<BUTTON_MODE>(mode);
    ButtonMode::buttonModeChanged = true;
    return;
}

void IRAM_ATTR ButtonMode::onRightButton(void) {
    ButtonMode::isRefresh = true; 
    return;
}

BUTTON_MODE ButtonMode::getMode(void){
    return ButtonMode::buttonMode;
}

bool ButtonMode::needRefresh(void){
    bool ret = ButtonMode::isRefresh;
    ButtonMode::isRefresh = false; 
    return ret;
}

void ButtonMode::initMode(void){
    ButtonMode::buttonMode = BUTTON_MODE::INIT;
    ButtonMode::buttonModeChanged = false;
    ButtonMode::isRefresh = true; 
    ButtonMode::isExistSD = true;
    ButtonMode::checkSdExist();
    return;
}

bool ButtonMode::isChanged(void) {
    bool ret = ButtonMode::buttonModeChanged;
    if (ret == true) {
        ButtonMode::buttonModeChanged = false;
        Serial.printf("[Debug] Mode Changed (Mode: %d)\r\n", ButtonMode::buttonMode);
    }
    return ret;
}

void ButtonMode::checkSdExist(void) {
    if (ButtonMode::isExistSD == true) {
        if (SD.open("/")) {
            ButtonMode::isExistSD = true;
        } else {
            ButtonMode::isExistSD = false;
            Serial.println("[WARN] SD is not mounted!");
        }
    }
    return;
}
