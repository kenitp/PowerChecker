#include "clock_int.h"
#include "pixel_shift.h"

static bool get_time_from_ntp(void);
static void display_clock(struct tm &timeinfo);

// 時間関連
static struct tm timeinfo;
uint8_t secLastReport = 0;
const char* week[7] = {"Sun", "Mon", "Tue", "wed", "Thu", "Fri", "Sat"};

void taskClock(void *args) {
    Serial.println("[Debug] taskClock Start");
    while(true) {
        if (ButtonMode::getMode() == BUTTON_MODE::CLOCK) {
            if (ButtonMode::isChanged()) {
                resetDisplay();
            }

            getLocalTime(&timeinfo);
            // 毎日午前2時に時刻取得。時刻取得に失敗しても動作継続
            if((timeinfo.tm_hour == 2)&&(timeinfo.tm_min == 0)&&(timeinfo.tm_sec == 0)) {
                get_time_from_ntp();
            }
            if(secLastReport != timeinfo.tm_sec) { //秒が更新されたら、表示をupdate
                secLastReport = timeinfo.tm_sec;
                display_clock(timeinfo);
            }
        }
        delay(100);
    }
    return;
}

static bool get_time_from_ntp(void) {
    //NTPによる時刻取得
    configTime(GMT_OFFSET_SEC, DAY_LIGHT_OFFSET_SEC, NTP_SERVER);
    if (!getLocalTime(&timeinfo)) {
        M5.Lcd.println("Failed to get time");
        return false;
    }
    return true;
}

void init_clock(void) {
    get_time_from_ntp();
    return;
}

static void display_clock(struct tm &timeinfo){
    static int8_t last_dx = 0, last_dy = 0;
    int8_t dx = PixelShift::getX();
    int8_t dy = PixelShift::getY();
    int16_t posX = 0, posY = 0;

    // シフト位置が変わったら画面をクリアして再描画
    if (dx != last_dx || dy != last_dy) {
        last_dx = dx;
        last_dy = dy;
        M5.Lcd.clear(BLACK);
    }

    // 日付（白）
    M5.Lcd.setTextFont(4);
    M5.Lcd.setTextSize(1);
    M5.Lcd.setTextColor(TFT_WHITE, TFT_BLACK);
    M5.Lcd.setCursor(0 + dx, 25 + dy);
    M5.Lcd.printf("%02d/%02d/%02d(%s)\r\n",
    timeinfo.tm_year + 1900, timeinfo.tm_mon + 1, timeinfo.tm_mday, week[timeinfo.tm_wday]);
    posX = M5.Lcd.getCursorX();
    posY = M5.Lcd.getCursorY();

    // セパレーターライン
    M5.Lcd.drawLine(0 + dx, posY + 2, 319 + dx, posY + 2, 0x4208);

    // 時刻（オレンジ/アンバー）
    M5.Lcd.setTextFont(7);
    M5.Lcd.setTextSize(2);
    M5.Lcd.setTextColor(TFT_ORANGE, TFT_BLACK);
    M5.Lcd.setCursor(posX+15, posY+10);
    M5.Lcd.printf("%02d:%02d\r\n",timeinfo.tm_hour, timeinfo.tm_min);
    posX = M5.Lcd.getCursorX();
    posY = M5.Lcd.getCursorY();

    // 秒（グレー）
    M5.Lcd.setTextFont(7);
    M5.Lcd.setTextSize(1);
    M5.Lcd.setTextColor(0x7BEF, TFT_BLACK);
    M5.Lcd.setCursor(230 + dx, posY+15);
    M5.Lcd.printf("%02d",timeinfo.tm_sec);
    return;
}