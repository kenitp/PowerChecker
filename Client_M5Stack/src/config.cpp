#include "config.h"

// Wi-Fi
const char* WIFI_SSID = "XXXXXXXXXXXXXX";
const char* WIFI_PASS = "XXXXXXXX";

// PowerCheck
const char* POWER_CHECKER_URL = "http://192.168.1.2:3000/api/power";
const char* power_img_dir[static_cast<int>(POWER_LEVEL::LvNUM)] = {
    "/img/power/low",        // POWER_LOW  140x184 image
    "/img/power/mid",        // POWER_MID
    "/img/power/high"        // POWER_HIGH
};

// Clock
const char* NTP_SERVER = "ntp.nict.jp";
const long GMT_OFFSET_SEC = 9 * 3600;
const int DAY_LIGHT_OFFSET_SEC = 0;

// FTP Server
const char* FTP_USER = "M5Stack";
const char* FTP_PASS = "M5Stack";
