#pragma once
#include <M5Stack.h>
#include <WiFi.h>
#include <time.h>
#include "config.h"
#include "button_mode.h"
#include "display.h"

extern void taskClock(void *args);
extern void init_clock(void);
