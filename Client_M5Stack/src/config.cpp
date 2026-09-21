#include "config.h"

// Wi-Fi
const char* WIFI_SSID = CFG_WIFI_SSID;
const char* WIFI_PASS = CFG_WIFI_PASS;

// PowerCheck
// Home Assistant のテンプレート API で必要な値だけを 1 リクエストで受け取る
const char* HA_TEMPLATE_URL = CFG_HA_BASE_URL "/api/template";
const char* HA_TOKEN = CFG_HA_TOKEN;
const char* HA_POWER_TEMPLATE =
    "{\"template\": \"{\\\"power_w\\\": \\\"{{ states('sensor.smart_meter_power') }}\\\", "
    "\\\"power_a\\\": \\\"{{ ((states('sensor.smart_meter_current_r')|float(0) + "
    "states('sensor.smart_meter_current_t')|float(0)) / 2) | round(1) }}\\\"}\"}";
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
const char* FTP_USER = CFG_FTP_USER;
const char* FTP_PASS = CFG_FTP_PASS;
