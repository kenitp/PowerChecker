#include <M5Stack.h>
#include <WiFi.h>
#include <WiFiMulti.h>
#include <HTTPClient.h>
#include <ArduinoJson.h>
#include "config.h"

WiFiMulti wifiMulti;

void resetDisplay(void);

DynamicJsonDocument doc(4096);

void setup() {
  M5.begin();

  // M5.Lcd.setCursor(20, 60);
  M5.Lcd.setBrightness(10);
  M5.Lcd.setTextFont(2);
  M5.Lcd.setTextSize(1);
  WiFi.begin(WIFI_SSID, WIFI_PASS);
  while (WiFi.status() != WL_CONNECTED){
      delay(500);
      M5.Lcd.print('.');
  }

  M5.Lcd.print("\r\nWiFi connected\r\nIP address: ");
  M5.Lcd.println(WiFi.localIP());
  delay(3000);
}

void loop() {
    static String last_power_w = "0";
    static String last_power_a = "0.0";

    if (WiFi.status() != WL_CONNECTED) {
        resetDisplay();
        M5.Lcd.println("WiFi not connected!");
        delay(1000);
        return;
    }

    if ((WiFi.status() == WL_CONNECTED)) {
        HTTPClient http;
        http.begin(POWER_CHECKER_URL); // (3)
        int httpCode = http.GET();

        if (httpCode > 0) {
            if (httpCode == HTTP_CODE_OK) {
                String payload = http.getString();
                deserializeJson(doc, payload);
                String power_a = doc["power_a"];
                String power_w = doc["power_w"];

                bool chg = false;
                if (last_power_a != power_a) {
                    last_power_a = power_a;
                    chg = true;
                }
                if (last_power_w != power_w) {
                    last_power_w = power_w;
                    chg = true;
                }

                if (chg == true) {
                    resetDisplay();
                    M5.Lcd.setTextSize(2);
                    M5.Lcd.println("Electricity Usage");
                    M5.Lcd.setTextSize(5);
                    M5.Lcd.printf(" %4s W\n", &power_w);
                    M5.Lcd.printf("  %4s A\n", &power_a);
                }
            }
        } else {
            resetDisplay();
            M5.Lcd.setTextSize(1);
            M5.Lcd.printf("[HTTP] GET... failed, error: %s\n", http.errorToString(httpCode).c_str());
        }

        http.end();
    }

    delay(30000);
}

void resetDisplay(void) {
    M5.Lcd.setCursor(0, 20);
    M5.Lcd.clear(BLACK);
    return;
}