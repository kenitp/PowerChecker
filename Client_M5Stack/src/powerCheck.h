#pragma once
#include <M5Stack.h>
#include <WiFi.h>
#include <HTTPClient.h>
#include <ArduinoJson.h>
#include "config.h"
#include "display.h"
#include "button.h"

extern void taskPower(void *args);
extern void taskPower_image(void *args);