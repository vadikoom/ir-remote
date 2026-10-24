#include <IRsend.h>
#include <ArduinoJson.h>
#include "logger.h"

#ifndef APPLICATION_H
#define APPLICATION_H

class Application {
    public:
        Application(int pinNumber) : irsend(pinNumber, true) {
            irsend.begin();
        }

        bool consumeCommand(DynamicJsonDocument &json) {
            if (!json["sequence"].is<uint32_t>() || !json["data"].is<JsonArray>()) {
                return false;
            }
            uint32_t number = json["sequence"];
            JsonArray data = json["data"];
            if (number == 0 || number == UINT32_MAX || data.size() == 0 ||
                data.size() > sizeof(commandBuffer) / sizeof(commandBuffer[0])) {
                return false;
            }
            for (JsonVariant value : data) {
                if (!value.is<uint16_t>() || value.as<uint16_t>() == 0) {
                    return false;
                }
            }
            Logger.print("Consuming command: ");
            Logger.println(number);

            if (number > lastCommandId) {
                size_t commandLen = jsonArrayIntoCommandBuffer(data, commandBuffer,
                    sizeof(commandBuffer) / sizeof(commandBuffer[0]));
                executeCommand(commandBuffer, commandLen);
                lastCommandId = number;
            }
            return true;
        }

        void reportStatus(DynamicJsonDocument &json) {
            Logger.print("reportStatus... ");
            json.clear();
            json["last_command_sequence_number"] = lastCommandId;
        }

    private:
        size_t jsonArrayIntoCommandBuffer(JsonArray array, uint16_t *buffer, size_t bufferLen) {
            size_t i = 0;
            for (JsonVariant v : array) {
                if (i >= bufferLen) {
                    Logger.println("Command buffer overflow!");
                    return 0;
                }
                buffer[i] = v.as<uint16_t>();
                i++;
            }

            return i;
        }

        void executeCommand(uint16_t *commandBuffer, size_t commandLen) {
            Logger.print("Executing command: ");
            for (int i = 0; i < commandLen; i++) {
                Logger.print(commandBuffer[i]);
                Logger.print(" ");
            }
            Logger.println();
            irsend.sendRaw(commandBuffer, commandLen, 38);  
        }

    private:
        IRsend irsend;

        uint16_t commandBuffer[300];
        uint32_t lastCommandId = 0;
};

#endif
