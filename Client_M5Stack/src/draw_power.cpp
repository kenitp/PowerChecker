#include <sstream>
#include "draw_power.h"

String DrawPower::last_power_w = "0";
String DrawPower::last_power_a = "0.0";

DrawPower::DrawPower(bool isExistImg) {
    is_exist_img = isExistImg;
    titleFont = 2;
    titleSize = 2; 
    valueFont = 4;
    if (is_exist_img == true) {
        valueSize = 2;
        w_offsetX = 20;
        w_offsetY = 40;
        a_offsetX = 30;
        a_offsetY = 20;
        a_unit_offsetX = 7; 
    } else {
        valueSize = 3;
        w_offsetX = 20;
        w_offsetY = 20;
        a_offsetX = 40;
        a_offsetY = 0;
        a_unit_offsetX = 11; 
    }
}

void DrawPower::draw(String *power_w, String *power_a, bool force){
    bool chg = false;

    if (last_power_a != *power_a) {
        last_power_a = *power_a;
        chg = true;
    }
    if (last_power_w != *power_w) {
        last_power_w = *power_w;
        chg = true;
    }

    if ((chg == true) || (force == true)) {
        resetDisplay();
        if (is_exist_img == true) {
            drawImage(power_w);
        }
        drawTitle();
        drawValues(power_w, power_a);
    }
    return;
}

void DrawPower::drawErr(const char *str){
    resetDisplay();
    M5.Lcd.setTextSize(1);
    M5.Lcd.println(str);
    return;
}

void DrawPower::drawTitle(void) {
    M5.Lcd.setTextFont(titleFont);
    M5.Lcd.setTextSize(titleSize);
    M5.Lcd.println("Electricity Usage");
    return;
}

void DrawPower::drawValues(String *power_w, String *power_a) {
    M5.Lcd.setTextFont(valueFont);
    M5.Lcd.setTextSize(valueSize);

    M5.Lcd.setCursor(M5.Lcd.getCursorX()+w_offsetX, M5.Lcd.getCursorY()+w_offsetY);
    M5.Lcd.printf("%4s ", power_w);
    int16_t curW_unit_X = M5.Lcd.getCursorX();
    M5.Lcd.println("W");

    M5.Lcd.setCursor(M5.Lcd.getCursorX()+a_offsetX, M5.Lcd.getCursorY()+a_offsetY);
    M5.Lcd.printf("%4s ", power_a);
    int16_t curA_unit_X = M5.Lcd.getCursorX();
    int16_t cur = max(curW_unit_X, curA_unit_X);
    M5.Lcd.setCursor(cur+a_unit_offsetX, M5.Lcd.getCursorY());
    M5.Lcd.println("A");
    return;
}

void DrawPower::drawImage(String *power_w) {
    int num = 0;
    Serial.printf("[Debug] power_w = %s", power_w);
    std::istringstream iss(power_w->c_str());
    iss >> num;
    Serial.printf("[Debug] power_w = %d", num);
    const char* img_path = power_img[static_cast<int>(POWER_LEVEL::LvLOW)];
    if (num < 300) {
        img_path = power_img[static_cast<int>(POWER_LEVEL::LvLOW)];
    } else if (num < 1200) {
        img_path = power_img[static_cast<int>(POWER_LEVEL::LvMID)];
    } else {
        img_path = power_img[static_cast<int>(POWER_LEVEL::LvHIGH)];
    }
    M5.Lcd.drawJpgFile(SD, img_path, 185, 55);
    return;
}