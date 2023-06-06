#include <IRremote.h>
#include <ArduinoJson.h>
#include "logger.h"

#ifndef APPLICATION_H
#define APPLICATION_H

class Application {
    public:
        Application(int ledPin) {
            irsend.begin(ledPin);
        }

        void consumeCommand(DynamicJsonDocument &json) {
            int number = json["sequence"];
            Logger.print("Consuming command: ");
            Logger.println(number);

            if (number > lastCommandId) {
                lastCommandId = number;
                size_t commandLen = jsonArrayIntoCommandBuffer(json["data"], commandBuffer, sizeof(commandBuffer));
                executeCommand(commandBuffer, commandLen);
            }
        }

        void reportStatus(DynamicJsonDocument &json) {
            Logger.print("reportStatus... ");
            json.clear();
            json["last_command_sequence_number"] = lastCommandId;
        }

    private:
        size_t jsonArrayIntoCommandBuffer(JsonArray array, uint16_t *buffer, size_t bufferLen) {
            int i = 0;
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
        int lastTimeStatusSent = 0;
        int lastCommandId = 0;
};

#endif