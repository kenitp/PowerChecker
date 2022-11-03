#pragma once
#include <vector>
#include <M5Stack.h>
#include "config.h"

class PowerPhoto {
public:
    PowerPhoto();
    ~PowerPhoto();

    static String* get_power_photo(POWER_LEVEL level);

private:
    static std::vector<String> photo_list_low;
    static std::vector<String> photo_list_mid;
    static std::vector<String> photo_list_high;

    static void get_photo_list(const char *path, std::vector<String> &list);
    static String* get_photo_random_from_list(std::vector<String> &list);
};