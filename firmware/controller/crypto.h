#ifndef CRYPTO_H
#define CRYPTO_H

#include <Arduino.h>
#include <ArduinoJson.h>
extern "C" {
#include <osapi.h>
}
#include "src/monocypher/monocypher.h"

class Crypto {
    public:
        explicit Crypto(const char *sharedSecret) {
            valid = strlen(sharedSecret) == 64;
            for (size_t i = 0; valid && i < sizeof(key); i++) {
                int high = hexDigit(sharedSecret[2 * i]);
                int low = hexDigit(sharedSecret[2 * i + 1]);
                valid = high >= 0 && low >= 0;
                key[i] = valid ? (high << 4) | low : 0;
            }
        }

        bool isValid() const { return valid; }

        size_t encrypt(DynamicJsonDocument &json, char *outBuffer, size_t outLen) {
            size_t plaintextLen = measureJson(json);
            if (!valid || outLen < OVERHEAD || plaintextLen > outLen - OVERHEAD ||
                plaintextLen > MAX_PACKET - OVERHEAD || json.overflowed()) {
                return 0;
            }
            uint8_t *packet = reinterpret_cast<uint8_t *>(outBuffer);
            packet[0] = 1;
            packet[1] = STATUS;
            if (os_get_random(packet + HEADER, NONCE) != 0) {
                return 0;
            }
            uint8_t *payload = packet + HEADER + NONCE;
            if (serializeJson(json, payload, plaintextLen) != plaintextLen) {
                return 0;
            }
            crypto_aead_lock(payload, payload + plaintextLen, key, packet + HEADER,
                             packet, HEADER, payload, plaintextLen);
            return plaintextLen + OVERHEAD;
        }

        bool decrypt(char *buffer, size_t len, DynamicJsonDocument &json) {
            if (!valid || len < OVERHEAD || len > MAX_PACKET) {
                return false;
            }
            uint8_t *packet = reinterpret_cast<uint8_t *>(buffer);
            if (packet[0] != 1 || packet[1] != COMMAND) {
                return false;
            }
            size_t plaintextLen = len - OVERHEAD;
            uint8_t *payload = packet + HEADER + NONCE;
            if (crypto_aead_unlock(payload, payload + plaintextLen, key, packet + HEADER,
                                   packet, HEADER, payload, plaintextLen) != 0) {
                return false;
            }
            return !deserializeJson(json, static_cast<const uint8_t *>(payload), plaintextLen);
        }

    private:
        static int hexDigit(char c) {
            if (c >= '0' && c <= '9') return c - '0';
            if (c >= 'a' && c <= 'f') return c - 'a' + 10;
            if (c >= 'A' && c <= 'F') return c - 'A' + 10;
            return -1;
        }

        static constexpr size_t HEADER = 2;
        static constexpr size_t NONCE = 24;
        static constexpr size_t OVERHEAD = HEADER + NONCE + 16;
        static constexpr size_t MAX_PACKET = 2048;
        static constexpr uint8_t COMMAND = 1;
        static constexpr uint8_t STATUS = 2;
        uint8_t key[32] = {};
        bool valid;
};

#endif
