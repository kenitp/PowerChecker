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

    static BUTTON_MODE get_mode(void);
    static void init_mode(void);
    static bool is_changed(void);
    static void check_sd_exist(void);

private:
    static BUTTON_MODE button_mode;
    static bool button_mode_changed;
    static bool is_exist_sd;
};
