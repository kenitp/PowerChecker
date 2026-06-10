#include "button_mode_int.h"

void taskButton(void *args) {
    Serial.println("[Debug] taskButton Start");
    while (true) {
        ButtonMode::update();
        delay(10);
    }
}

BUTTON_MODE ButtonMode::buttonMode;
bool ButtonMode::buttonModeChanged;
bool ButtonMode::isExistSD;
bool ButtonMode::isRefresh;


void ButtonMode::cycleMode(void) {
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

void ButtonMode::requestRefresh(void) {
    ButtonMode::isRefresh = true;
    return;
}

void ButtonMode::update(void) {
    M5.update();

    // [一時デバッグ] タッチの状態を毎秒出力して切り分ける
    static unsigned long lastTouchDbg = 0;
    unsigned long nowDbg = millis();
    if (nowDbg - lastTouchDbg >= 1000) {
        lastTouchDbg = nowDbg;
        auto d = M5.Touch.getDetail();
        Serial.printf("[Debug] touch en=%d cnt=%d x=%d y=%d state=%d\r\n",
            (int)M5.Touch.isEnabled(), (int)M5.Touch.getCount(),
            d.x, d.y, (int)d.state);
    }

    // 画面下のボタン（Core2 はベゼルのタッチボタン / Fire は物理ボタン）
    // BtnB=中央(モード切替) / BtnC=右(更新)
    if (M5.BtnB.wasPressed()) {
        cycleMode();
    }
    if (M5.BtnC.wasPressed()) {
        requestRefresh();
    }

    // タッチデバイス（Core2 等）は画面タップでも操作できる
    if (M5.Touch.isEnabled()) {
        updateTouch();
    }
    return;
}

void ButtonMode::updateTouch(void) {
    // タップ（押して離した）瞬間を1回の操作として検出する。
    // wasClicked() は指を離した時に発火するため getCount() でガードしてはいけない
    // （離した時点では getCount() が 0 になるため）。
    auto detail = M5.Touch.getDetail();
    if (!detail.wasClicked()) {
        return;
    }
    // 画面下のボタン領域(y >= 表示高さ)は BtnB/BtnC 側で処理するため除外する
    if (detail.y >= M5.Display.height()) {
        return;
    }
    Serial.printf("[Debug] Touch tap x=%d y=%d\r\n", detail.x, detail.y);
    // 画面左半分=モード切替 / 右半分=更新
    if (detail.x < (M5.Display.width() / 2)) {
        cycleMode();
    } else {
        requestRefresh();
    }
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
