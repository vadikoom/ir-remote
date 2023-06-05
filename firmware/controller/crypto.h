#include <Arduino.h>
#include <ArduinoJson.h>
#include "logger.h"

#ifndef CRYPTO_H
#define CRYPTO_H

class Crypto {
    public:
        Crypto(const char *sharedSecret) {
            this->sharedSecret = sharedSecret;
        }

        size_t encrypt(DynamicJsonDocument &json, char *outBuffer, size_t outLen) {
            Logger.print("Encrypting JSON document... ");
            int resultSize = serializeJson(json, outBuffer, outLen);
            Logger.print("Encrypted: ");
            Logger.print(resultSize);
            Logger.print(". bytes: ");
            outBuffer[resultSize] = '\0';
            Logger.println(outBuffer);
            return resultSize;
        }

        bool decrypt(char *buffer, size_t len, DynamicJsonDocument &json) {
            Logger.print("Decrypting JSON document... ");
            deserializeJson(json, buffer, len);
            Logger.print("Decrypted: ");
            Logger.println(json.size());
            return true;
        }
    
    private:
        const char *sharedSecret;
};

#endif