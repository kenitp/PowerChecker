#include <sstream>
#include "draw_power_int.h"
#include "pixel_shift.h"

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
    M5.Display.setTextSize(1);
    M5.Display.println(str);
    return;
}

void DrawPower::drawTitle(void) {
    M5.Display.setTextFont(titleFont);
    M5.Display.setTextSize(titleSize);
    M5.Display.setTextColor(TFT_WHITE, TFT_BLACK);
    M5.Display.println("Electricity Usage");
    return;
}

void DrawPower::drawValues(String *power_w, String *power_a) {
    // 電力値をパース
    int num = 0;
    std::istringstream iss(power_w->c_str());
    iss >> num;

    // 電力レベルに応じた色（低:緑 / 中:オレンジ / 高:赤）
    uint16_t wColor;
    if (num < 300) {
        wColor = 0x07E0;   // グリーン
    } else if (num < 1200) {
        wColor = TFT_ORANGE;
    } else {
        wColor = TFT_RED;
    }

    M5.Display.setTextFont(valueFont);
    M5.Display.setTextSize(valueSize);

    M5.Display.setTextColor(wColor, TFT_BLACK);
    M5.Display.setCursor(M5.Display.getCursorX()+w_offsetX, M5.Display.getCursorY()+w_offsetY);
    M5.Display.printf("%4s ", power_w);
    int16_t curW_unit_X = M5.Display.getCursorX();
    M5.Display.println("W");

    M5.Display.setTextColor(0xBDF7, TFT_BLACK);  // ライトグレー
    M5.Display.setCursor(M5.Display.getCursorX()+a_offsetX, M5.Display.getCursorY()+a_offsetY);
    M5.Display.printf("%4s ", power_a);
    int16_t curA_unit_X = M5.Display.getCursorX();
    int16_t cur = max(curW_unit_X, curA_unit_X);
    M5.Display.setCursor(cur+a_unit_offsetX, M5.Display.getCursorY());
    M5.Display.println("A");
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
        M5.Display.drawJpgFile(SD, img_path->c_str(), 185 + PixelShift::getX(), 55 + PixelShift::getY());
    } else {
        Serial.printf("[Debug] drawPng = %s\r\n", img_path->c_str());
        M5.Display.drawPngFile(SD, img_path->c_str(), 185 + PixelShift::getX(), 55 + PixelShift::getY());
    }
    return;
}