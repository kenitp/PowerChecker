#pragma once
#include <vector>
#include <M5Stack.h>
#include "config.h"

class PowerPhoto {
public:
    PowerPhoto();
    ~PowerPhoto();

    String* getPowerPhoto(POWER_LEVEL level);

private:
    std::vector<String> photo_list_low;
    std::vector<String> photo_list_mid;
    std::vector<String> photo_list_high;

    void getPhotoList(const char *path, std::vector<String> &list);
    String* getPhotoRandomFromList(std::vector<String> &list);
};
