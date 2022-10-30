#pragma once

#include <M5Stack.h>
#include "display.h"
#include "config.h"

class DrawPower {
public:
    DrawPower(bool isExistImg);
    ~DrawPower(){};

    void draw(String *power_w, String *power_a, bool force);
    void drawErr(const char *str);
    void display_power_values_image(String *power_w, String *power_a);
    void displayValues_image(String *power_w, String *power_a);

private:
    bool is_exist_img;
    uint8_t titleFont;
    uint8_t titleSize;
    uint8_t valueFont;
    uint8_t valueSize;

    int16_t w_offsetX;
    int16_t w_offsetY;
    int16_t a_offsetX;
    int16_t a_offsetY;
    int16_t a_unit_offsetX;    

    static String last_power_w;
    static String last_power_a;

    void drawTitle(void);
    void drawValues(String *power_w, String *power_a);
    void drawImage(String *power_w);
};