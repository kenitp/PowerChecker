#include <SD.h>
#include "power_photo_int.h"

PowerPhoto::PowerPhoto(){
    this->photo_list_low.clear();
    this->photo_list_mid.clear();
    this->photo_list_high.clear();
    this->getPhotoList(power_img_dir[static_cast<int>(POWER_LEVEL::LvLOW)], this->photo_list_low);
    this->getPhotoList(power_img_dir[static_cast<int>(POWER_LEVEL::LvMID)], this->photo_list_mid);
    this->getPhotoList(power_img_dir[static_cast<int>(POWER_LEVEL::LvHIGH)], this->photo_list_high);
}

PowerPhoto::~PowerPhoto(){
    this->photo_list_low.clear();
    this->photo_list_low.shrink_to_fit();
    this->photo_list_mid.clear();
    this->photo_list_mid.shrink_to_fit();
    this->photo_list_high.clear();
    this->photo_list_high.shrink_to_fit();
}

void PowerPhoto::getPhotoList(const char* path, std::vector<String> &list) {
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

String* PowerPhoto::getPowerPhoto(POWER_LEVEL level) {
    String *ret;
    if (level == POWER_LEVEL::LvLOW) {
        ret = this->getPhotoRandomFromList(this->photo_list_low);
    } else if (level == POWER_LEVEL::LvMID) {
        ret = this->getPhotoRandomFromList(this->photo_list_mid);
    } else {
        ret = this->getPhotoRandomFromList(this->photo_list_high);
    }
    return ret;
}

String* PowerPhoto::getPhotoRandomFromList(std::vector<String> &list) {
    int index = random(list.size());
    return &list[index];
}
