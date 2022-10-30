#pragma once

enum POWER_LEVEL {
    POWER_LOW,
    POWER_MID,
    POWER_HIGH,
    POWER_NUM
};

extern const char* WIFI_SSID;
extern const char* WIFI_PASS;
extern const char* POWER_CHECKER_URL;
extern const char* NTP_SERVER;
extern const long GMT_OFFSET_SEC;
extern const int DAY_LIGHT_OFFSET_SEC;

extern const char* power_img[POWER_NUM];
