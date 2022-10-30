#pragma once
#include <M5Stack.h>
#include <SD.h>

enum BUTTON_MODE {
    MODE_INIT,
    MODE_POWER = MODE_INIT,
    MODE_POWER_IMG,
    MODE_CLOCK,
    MODE_PHOTO,
    MODE_NUM
};

extern void IRAM_ATTR onLeftButton(void);
extern void IRAM_ATTR onMiddleButton(void);
extern void IRAM_ATTR onRightButton(void);

extern BUTTON_MODE get_button_mode(void);
extern void init_button_mode(void);
extern bool is_mode_changed(void);