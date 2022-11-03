#include <M5Stack.h>

void resetDisplay(void) {
    M5.Lcd.setCursor(0, 15);
    M5.Lcd.clear(BLACK);
    // M5.Lcd.setTextDatum(TL_DATUM);
    return;
}
