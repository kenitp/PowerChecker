#include <SD.h>
#include "power_photo_int.h"

std::vector<String> PowerPhoto::photo_list_low;
std::vector<String> PowerPhoto::photo_list_mid;
std::vector<String> PowerPhoto::photo_list_high;

PowerPhoto::PowerPhoto(){
    get_photo_list(power_img_dir[static_cast<int>(POWER_LEVEL::LvLOW)], photo_list_low);
    get_photo_list(power_img_dir[static_cast<int>(POWER_LEVEL::LvMID)], photo_list_mid);
    get_photo_list(power_img_dir[static_cast<int>(POWER_LEVEL::LvHIGH)], photo_list_high);
}

PowerPhoto::~PowerPhoto(){
    photo_list_low.clear();
    photo_list_low.shrink_to_fit();
    photo_list_mid.clear();
    photo_list_mid.shrink_to_fit();
    photo_list_high.clear();
    photo_list_high.shrink_to_fit();
}

void PowerPhoto::get_photo_list(const char* path, std::vector<String> &list) {
    File root = SD.open(path);
    if (root) {
        File file = root.openNextFile();
        while (file) {
            if (file.isDirectory()) {
                // Dir skip
            } else {
                // File
                String filepath = file.path();
                Serial.printf("Img = %s\r\n", filepath.c_str());
                if ((filepath.indexOf(".jpg") != -1) || (filepath.indexOf(".png") != -1)) {
                    // Find
                    list.push_back(filepath);
                }
            }
            file = root.openNextFile();
        }
    }
}

String* PowerPhoto::get_power_photo(POWER_LEVEL level) {
    String *ret;
    if (level == POWER_LEVEL::LvLOW) {
        ret = get_photo_random_from_list(photo_list_low);
    } else if (level == POWER_LEVEL::LvMID) {
        ret = get_photo_random_from_list(photo_list_mid);
    } else {
        ret = get_photo_random_from_list(photo_list_high);
    }
    return ret;
}

String* PowerPhoto::get_photo_random_from_list(std::vector<String> &list) {

    int index = random(list.size());
    return &list[index];
}
