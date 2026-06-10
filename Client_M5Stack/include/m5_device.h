#pragma once

// SD 画像描画 API を使うため M5Unified より先に SD.h を include する
#include <SD.h>
#include <SPI.h>
#include <M5Unified.h>

uint8_t deviceNormalBrightness(void);

// デバイスごとの正しい SPI ピンで SD カードを初期化する。
// マウント成功で true を返す。
bool deviceInitSD(void);
