#include "main.h"
#include "pixel_shift.h"
#include "m5_device.h"

void setup() {
    auto cfg = M5.config();
    // M5Unified はデフォルトで Serial.begin を実行しないため、明示的に有効化する
    cfg.serial_baudrate = 115200;
    M5.begin(cfg);
    delay(500);
    adc_power_acquire();

    M5.Display.setBrightness(deviceNormalBrightness());
    M5.Display.setTextFont(2);
    M5.Display.setTextSize(1);

    if (deviceInitSD()) {
        Serial.println("[Debug] SD mounted");
    } else {
        Serial.println("[WARN] SD mount failed");
    }
    connect_wifi(WIFI_SSID, WIFI_PASS);
    while (get_wifi_status() != WL_CONNECTED){
        delay(500);
        M5.Display.print('.');
    }

    M5.Display.print("\r\nWiFi connected\r\nIP address: ");
    M5.Display.println(WiFi.localIP());
    init_clock();

    ButtonMode::initMode();
    xTaskCreatePinnedToCore(taskButton, "Button", 4096, NULL, 4, NULL, 1);
    xTaskCreatePinnedToCore(taskPower, "PowerCheck", 8192, NULL, 3, NULL, 1);
    xTaskCreatePinnedToCore(taskClock, "Clock", 4096, NULL, 4, NULL, 1);
    xTaskCreatePinnedToCore(taskPhoto, "Photo", 8192, NULL, 3, NULL, 1);
    xTaskCreatePinnedToCore(taskScreenRepair, "ScreenRepair", 4096, NULL, 3, NULL, 1);
    xTaskCreatePinnedToCore(taskScreenCheck, "ScreenCheck", 4096, NULL, 3, NULL, 1);
    xTaskCreatePinnedToCore(taskFtpServer, "FtpServer", 8192, NULL, 2, NULL, 1);
    delay(3000);
}

void loop() {
    // ボタン/タッチ入力は taskButton で処理する
    ButtonMode::checkSdExist();
    PixelShift::tick();
    delay(3000);
}
