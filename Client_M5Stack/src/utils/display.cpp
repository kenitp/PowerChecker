#include <M5Stack.h>
#include "pixel_shift.h"

void resetDisplay(void) {
    M5.Lcd.clear(BLACK);
    M5.Lcd.setCursor(0 + PixelShift::getX(), 15 + PixelShift::getY());
    // M5.Lcd.setTextDatum(TL_DATUM);
    return;
}
