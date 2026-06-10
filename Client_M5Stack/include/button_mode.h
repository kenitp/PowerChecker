#pragma once
#include "m5_device.h"

enum class BUTTON_MODE : int {
    INIT,
    POWER = INIT,
    POWER_IMG,
    CLOCK,
    PHOTO,
    REPAIR,
    CHECK,
    NUM
};

class ButtonMode {
public:
    ButtonMode(){};
    ~ButtonMode(){};

    static void update(void);

    static BUTTON_MODE getMode(void);
    static bool needRefresh(void);
    static void initMode(void);
    static bool isChanged(void);
    static void checkSdExist(void);

private:
    static void cycleMode(void);
    static void requestRefresh(void);
    static void updateTouch(void);

    static BUTTON_MODE buttonMode;
    static bool buttonModeChanged;
    static bool isExistSD;
    static bool isRefresh;
};

// ボタン/タッチ入力を監視するタスク
void taskButton(void *args);
