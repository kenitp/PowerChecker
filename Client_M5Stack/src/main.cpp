#include "main.h"

void setup() {
    M5.begin();
    delay(500);
    adc_power_acquire(); 

    pinMode(GPIO_NUM_39, INPUT);
    pinMode(GPIO_NUM_38, INPUT);
    pinMode(GPIO_NUM_37, INPUT);

    attachInterrupt(digitalPinToInterrupt(GPIO_NUM_39), ButtonMode::onLeftButton, FALLING);
    attachInterrupt(digitalPinToInterrupt(GPIO_NUM_38), ButtonMode::onMiddleButton, FALLING);
    attachInterrupt(digitalPinToInterrupt(GPIO_NUM_37), ButtonMode::onRightButton, FALLING);

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
    init_clock();

    ButtonMode::initMode();
    xTaskCreatePinnedToCore(taskPower, "PowerCheck", 8192, NULL, 3, NULL, 1);
    xTaskCreatePinnedToCore(taskClock, "Clock", 4096, NULL, 4, NULL, 1);
    xTaskCreatePinnedToCore(taskPhoto, "Photo", 8192, NULL, 3, NULL, 1);
    xTaskCreatePinnedToCore(taskFtpServer, "FtpServer", 8192, NULL, 2, NULL, 1);
    delay(3000);
}

void loop() {
    delay(3000);
    ButtonMode::checkSdExist();
}
