#pragma once
#include <M5Stack.h>

enum class BUTTON_MODE : int {
    INIT,
    POWER = INIT,
    POWER_IMG,
    CLOCK,
    PHOTO,
    NUM
};

class ButtonMode {
public:
    ButtonMode(){};
    ~ButtonMode(){};

    // Button 割込み Handler
    static void IRAM_ATTR onLeftButton(void);
    static void IRAM_ATTR onMiddleButton(void);
    static void IRAM_ATTR onRightButton(void);

    static BUTTON_MODE getMode(void);
    static bool needRefresh(void);
    static void initMode(void);
    static bool isChanged(void);
    static void checkSdExist(void);

private:
    static BUTTON_MODE buttonMode;
    static bool buttonModeChanged;
    static bool isExistSD;
    static bool isRefresh;
};
