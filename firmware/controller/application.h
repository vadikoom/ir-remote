#include <ArduinoJson.h>
#include "logger.h"

#ifndef APPLICATION_H
#define APPLICATION_H

class Application {
    public:
        Application() {
        }

        void consumeCommand(DynamicJsonDocument &json) {
            int number = json["sequence"];
            Logger.print("Consuming command: ");
            Logger.println(number);
        }

        void reportStatus(DynamicJsonDocument &json) {
            Logger.print("reportStatus... ");
            json.clear();
            json["last_command_sequence_number"] = lastCommandId;
        }

    private:
        int lastTimeStatusSent = 0;
        int lastCommandId = 0;
};

#endif