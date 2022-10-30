#include "config.h"

// Wi-Fi
const char* WIFI_SSID = "XXXXXXXXXXXXXX";
const char* WIFI_PASS = "XXXXXXXX";

// PowerCheck WEB API
const char* POWER_CHECKER_URL = "http://192.168.1.2:3000/api/power";

// Clock
const char* NTP_SERVER = "ntp.nict.jp";
const long GMT_OFFSET_SEC = 9 * 3600;
const int DAY_LIGHT_OFFSET_SEC = 0;

const char* power_img[static_cast<int>(POWER_LEVEL::LvNUM)] = {
    "/img/power/img_low.jpg",        // POWER_LOW
    "/img/power/img_mid.jpg",        // POWER_MID
    "/img/power/img_high.jpg"        // POWER_HIGH
};
