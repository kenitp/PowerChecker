#include <cstdio>
#include <sstream>
#include "powerCheck_int.h"
#include "power_photo_int.h"

static DynamicJsonDocument doc(4096);

void taskPower(void *args) {
    int count_sec = 0;

    Serial.println("[Debug] taskPower Start");
    PowerPhoto pp = PowerPhoto();

    while(true) {
        bool force = false;
        BUTTON_MODE mode = ButtonMode::get_mode();
        if ((mode == BUTTON_MODE::POWER) || (mode == BUTTON_MODE::POWER_IMG)) {
            bool isDrawImg = (mode == BUTTON_MODE::POWER_IMG);
            DrawPower dp = DrawPower(isDrawImg);
            if (ButtonMode::is_changed()) {
                force = true;
                count_sec = 0;
            }
            if (count_sec == 0) {
                Serial.println("[Debug] Update Power Values");
                if (WiFi.status() != WL_CONNECTED) {
                    const char* errStr = "WiFi not connected!";
                    dp.drawErr(errStr);
                    delay(1000);
                }

                if ((WiFi.status() == WL_CONNECTED)) {
                    HTTPClient http;
                    http.begin(POWER_CHECKER_URL);
                    int httpCode = http.GET();

                    if (httpCode > 0) {
                        if (httpCode == HTTP_CODE_OK) {
                            String payload = http.getString();
                            deserializeJson(doc, payload);
                            String power_a = doc["power_a"];
                            String power_w = doc["power_w"];

                            dp.draw(&power_w, &power_a, force, pp);
                        }
                    } else {
                        char str[200];
                        (void)sprintf (str, "[HTTP] GET... failed, error: %s\n", http.errorToString(httpCode).c_str());
                        dp.drawErr(str);
                    }
                    http.end();
                }
            }
            count_sec++;
            if (30  <= count_sec){
                count_sec = 0;
            }
        }
        delay(1000);
    }
    return;
}
