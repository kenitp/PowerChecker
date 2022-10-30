#pragma once
#include <M5Stack.h>

enum class POWER_LEVEL : int {
    LvLOW,
    LvMID,
    LvHIGH,
    LvNUM
};

extern const char* WIFI_SSID;
extern const char* WIFI_PASS;
extern const char* POWER_CHECKER_URL;
extern const char* NTP_SERVER;
extern const long GMT_OFFSET_SEC;
extern const int DAY_LIGHT_OFFSET_SEC;

extern const char* power_img[static_cast<int>(POWER_LEVEL::LvNUM)];
