#include <cstdio>
#include <sstream>
#include "powerCheck_int.h"
#include "power_photo_int.h"

static DynamicJsonDocument doc(4096);

void taskPower(void *args) {
    int count_sec = 0;

    Serial.println("[Debug] taskPower Start");
    std::shared_ptr<PowerPhoto> pp(new PowerPhoto());

    while(true) {
        bool force = false;
        BUTTON_MODE mode = ButtonMode::getMode();
        if ((mode == BUTTON_MODE::POWER) || (mode == BUTTON_MODE::POWER_IMG)) {
            if (ButtonMode::isChanged()) {
                force = true;
                count_sec = 0;
            } else if (ButtonMode::needRefresh() && (mode == BUTTON_MODE::POWER_IMG)) {
                pp = std::shared_ptr<PowerPhoto>(new PowerPhoto());
                force = true;
                count_sec = 0;
            }
            if (count_sec == 0) {
                bool isDrawImg = (mode == BUTTON_MODE::POWER_IMG);
                DrawPower dp = DrawPower(isDrawImg, pp);
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

                            dp.draw(&power_w, &power_a, force);
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
