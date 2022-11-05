#include <sstream>
#include "draw_power_int.h"

String DrawPower::last_power_w = "0";
String DrawPower::last_power_a = "0.0";

DrawPower::DrawPower(bool isExistImg, std::shared_ptr<PowerPhoto> ins_pp) {
    this->is_exist_img = isExistImg;
    this->ins_pp = ins_pp;
    this->titleFont = 2;
    this->titleSize = 2; 
    this->valueFont = 4;
    if (this->is_exist_img == true) {
        this->valueSize = 2;
        this->w_offsetX = 20;
        this->w_offsetY = 40;
        this->a_offsetX = 30;
        this->a_offsetY = 20;
        this->a_unit_offsetX = 7; 
    } else {
        this->valueSize = 3;
        this->w_offsetX = 20;
        this->w_offsetY = 20;
        this->a_offsetX = 40;
        this->a_offsetY = 0;
        this->a_unit_offsetX = 11; 
    }
}

void DrawPower::draw(String *power_w, String *power_a, bool force){
    bool chg = false;

    if (DrawPower::last_power_a != *power_a) {
        DrawPower::last_power_a = *power_a;
        chg = true;
    }
    if (DrawPower::last_power_w != *power_w) {
        DrawPower::last_power_w = *power_w;
        chg = true;
    }

    if ((chg == true) || (force == true)) {
        resetDisplay();
        if (this->is_exist_img == true) {
            this->drawImage(power_w);
        }
        this->drawTitle();
        this->drawValues(power_w, power_a);
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
    std::istringstream iss(power_w->c_str());
    iss >> num;
    String* img_path;

    if (num < 300) {
        img_path = this->ins_pp->getPowerPhoto(POWER_LEVEL::LvLOW);
        Serial.printf("[Debug] image_low = %s\r\n", img_path->c_str());
    } else if (num < 1200) {
        img_path = this->ins_pp->getPowerPhoto(POWER_LEVEL::LvMID);
        Serial.printf("[Debug] image_mid = %s\r\n", img_path->c_str());
    } else {
        img_path = this->ins_pp->getPowerPhoto(POWER_LEVEL::LvHIGH);
        Serial.printf("[Debug] image_high = %s\r\n", img_path->c_str());
    }
    if (img_path->indexOf(".jpg") != -1) {
        Serial.printf("[Debug] drawJpg = %s\r\n", img_path->c_str());
        M5.Lcd.drawJpgFile(SD, img_path->c_str(), 185, 55);
    } else {
        Serial.printf("[Debug] drawPng = %s\r\n", img_path->c_str());
        M5.Lcd.drawPngFile(SD, img_path->c_str(), 185, 55);
    }
    return;
}
