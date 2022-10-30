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

const char* power_img[POWER_NUM] = {
    "/img/power/moomin_low.jpg",        // POWER_LOW
    "/img/power/moomin_mid.jpg",        // POWER_MID
    "/img/power/moomin_high.jpg"        // POWER_HIGH
};
