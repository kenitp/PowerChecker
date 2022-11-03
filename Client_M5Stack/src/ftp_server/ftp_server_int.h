#pragma once

#include <WiFi.h>
#include <WiFiClient.h>
#include "ESP32FtpServer.h"

#include "config.h"

extern void taskFtpServer(void *args);
