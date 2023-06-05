#include <Arduino.h>
#include <Print.h>

#ifndef LOGGER_H
#define LOGGER_H

class LoggerType : public Print {
public:
    LoggerType() {
        Serial.begin(115200);
    }

    size_t write(uint8_t character) override {
        Serial.write(character);
        return 1; // Indicate success
    }

    size_t write(const uint8_t *buffer, size_t size) override {
        Serial.write(buffer, size);
        return size; // Indicate success
    }
};

LoggerType Logger;

#endif