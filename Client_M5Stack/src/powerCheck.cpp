#include "powerCheck.h"

static void displayTitle(void);
static void displayValues(String *power_w, String *power_a);
static void displayValues_image(String *power_w, String *power_a);
static void display_power_values(String *power_w, String *power_a);
static void display_power_values_image(String *power_w, String *power_a);

static DynamicJsonDocument doc(4096);

void taskPower(void *args) {
    static String last_power_w = "0";
    static String last_power_a = "0.0";
    int count_sec = 0;

    Serial.println("[Debug] taskPower Start");

    while(true) {
        bool chg = false;
        BUTTON_MODE mode = get_button_mode();
        if ((mode == MODE_POWER) || (mode == MODE_POWER_IMG)) {
            if (is_mode_changed()) {
                chg = true;
                count_sec = 0;
            }
            if (count_sec == 0) {
                Serial.println("[Debug] Update Power Values");
                if (WiFi.status() != WL_CONNECTED) {
                    resetDisplay();
                    M5.Lcd.println("WiFi not connected!");
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

                            if (last_power_a != power_a) {
                                last_power_a = power_a;
                                chg = true;
                            }
                            if (last_power_w != power_w) {
                                last_power_w = power_w;
                                chg = true;
                            }
                            if (chg == true) {
                                if (mode == MODE_POWER_IMG) {
                                    display_power_values_image(&power_w, &power_a);
                                } else {
                                    display_power_values(&power_w, &power_a);
                                }
                            }
                        }
                    } else {
                        resetDisplay();
                        M5.Lcd.setTextSize(1);
                        M5.Lcd.printf("[HTTP] GET... failed, error: %s\n", http.errorToString(httpCode).c_str());
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

static void display_power_values(String *power_w, String *power_a){
    resetDisplay();
    displayTitle();
    displayValues(power_w, power_a);
    return;
}

static void displayTitle(void) {
    M5.Lcd.setTextFont(2);
    M5.Lcd.setTextSize(2);
    M5.Lcd.println("Electricity Usage");
    return;
}

static void displayValues(String *power_w, String *power_a) {
    M5.Lcd.setTextFont(4);
    M5.Lcd.setTextSize(3);

    M5.Lcd.setCursor(M5.Lcd.getCursorX()+20, M5.Lcd.getCursorY()+20);
    M5.Lcd.printf(" %4s ", power_w);
    int16_t curW_unit_X = M5.Lcd.getCursorX();
    M5.Lcd.println("W");

    M5.Lcd.setCursor(M5.Lcd.getCursorX()+40, M5.Lcd.getCursorY());
    M5.Lcd.printf(" %4s ", power_a);
    int16_t curA_unit_X = M5.Lcd.getCursorX();
    int16_t cur = max(curW_unit_X, curA_unit_X);
    M5.Lcd.setCursor(cur+11, M5.Lcd.getCursorY());
    M5.Lcd.println("A");
    return;
}

static void display_power_values_image(String *power_w, String *power_a){
    // std::istringstream ss;
    int num = atoi((*power_w).c_str());
    const char* img_path = power_img[POWER_LOW];
    if (num < 300) {
        img_path = power_img[POWER_LOW];
    } else if (num < 1200) {
        img_path = power_img[POWER_MID];
    } else {
        img_path = power_img[POWER_HIGH];
    }
    resetDisplay();
    M5.Lcd.drawJpgFile(SD, img_path, 185, 55);
    displayTitle();
    displayValues_image(power_w, power_a);
    return;
}

static void displayValues_image(String *power_w, String *power_a) {
    M5.Lcd.setTextFont(4);
    M5.Lcd.setTextSize(2);

    M5.Lcd.setCursor(M5.Lcd.getCursorX()+20, M5.Lcd.getCursorY()+40);
    M5.Lcd.printf("%4s ", power_w);
    int16_t curW_unit_X = M5.Lcd.getCursorX();
    M5.Lcd.println("W");

    M5.Lcd.setCursor(M5.Lcd.getCursorX()+30, M5.Lcd.getCursorY()+20);
    M5.Lcd.printf("%4s ", power_a);
    int16_t curA_unit_X = M5.Lcd.getCursorX();
    int16_t cur = max(curW_unit_X, curA_unit_X);
    M5.Lcd.setCursor(cur+7, M5.Lcd.getCursorY());
    M5.Lcd.println("A");
    return;
}

