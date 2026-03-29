#include "wifi_connect.h"

void connect_wifi(const char* ssid, const char* pass) {
    WiFi.begin(ssid, pass);
}

wl_status_t get_wifi_status() {
    return WiFi.status();
}
