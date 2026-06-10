#include "m5_device.h"
#include "pixel_shift.h"

void resetDisplay(void) {
    M5.Display.fillScreen(TFT_BLACK);
    M5.Display.setCursor(0 + PixelShift::getX(), 15 + PixelShift::getY());
    // M5.Display.setTextDatum(TL_DATUM);
    return;
}
