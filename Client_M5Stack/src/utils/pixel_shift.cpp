#include "pixel_shift.h"

// シフトパターン: (dx, dy) を順番に適用する
const int8_t PixelShift::SHIFTS[][2] = {
    { 0,  0},
    { 2,  0},
    { 2,  2},
    { 0,  2},
    {-2,  2},
    {-2,  0},
    {-2, -2},
    { 0, -2},
    { 2, -2},
};
const uint8_t PixelShift::NUM_SHIFTS = 9;
uint8_t PixelShift::current_index = 0;
unsigned long PixelShift::last_update_ms = 0;

void PixelShift::tick(void) {
    unsigned long now = millis();
    if (now - last_update_ms >= SHIFT_INTERVAL_MS) {
        last_update_ms = now;
        current_index = (current_index + 1) % NUM_SHIFTS;
    }
}

int8_t PixelShift::getX(void) {
    return SHIFTS[current_index][0];
}

int8_t PixelShift::getY(void) {
    return SHIFTS[current_index][1];
}
