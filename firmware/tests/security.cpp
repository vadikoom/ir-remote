#include <cassert>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <vector>
#define LOGGER_H
struct TestLogger {
    template<class T> void print(T) {}
    template<class T> void println(T) {}
    void println() {}
} Logger;
#include "../controller/application.h"
#include "../controller/crypto.h"

size_t transmissions = 0;
size_t transmittedLength = 0;
static bool randomFails = false;
extern "C" int os_get_random(unsigned char *buffer, size_t length) {
    for (size_t i = 0; i < length; i++) buffer[i] = i;
    return randomFails ? -1 : 0;
}
const char *secret = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

std::vector<char> commandPacket(const std::string &plaintext) {
    std::vector<char> packet(plaintext.size() + 42);
    uint8_t key[32], nonce[24];
    for (size_t i = 0; i < 32; i++) key[i] = i;
    for (size_t i = 0; i < 24; i++) nonce[i] = i;
    packet[0] = 1; packet[1] = 1;
    memcpy(packet.data() + 2, nonce, 24);
    auto bytes = reinterpret_cast<uint8_t *>(packet.data());
    crypto_aead_lock(bytes + 26, bytes + 26 + plaintext.size(), key, nonce, bytes, 2,
                     reinterpret_cast<const uint8_t *>(plaintext.data()), plaintext.size());
    return packet;
}

std::string hex(const char *bytes, size_t length) {
    std::ostringstream output;
    for (size_t i = 0; i < length; i++)
        output << std::hex << std::setfill('0') << std::setw(2) << unsigned(uint8_t(bytes[i]));
    return output.str();
}

int main(int argc, char **argv) {
    assert(argc >= 2);
    Crypto crypto(secret);
    assert(crypto.isValid());
    assert(!Crypto("password").isValid());
    assert(!Crypto(std::string(64, 'g').c_str()).isValid());
    DynamicJsonDocument json(32768);
    const std::string command = "{\"sequence\":1,\"data\":[100,200]}";
    auto packet = commandPacket(command);
    auto input = packet;
    assert(crypto.decrypt(input.data(), input.size(), json));
    Application application(2);
    assert(application.consumeCommand(json));
    assert(transmissions == 1 && transmittedLength == 2);
    assert(application.consumeCommand(json));
    assert(transmissions == 1);

    for (size_t i = 0; i < packet.size(); i++) {
        input = packet; input[i] ^= 1;
        assert(!crypto.decrypt(input.data(), input.size(), json));
    }
    for (size_t i = 0; i < packet.size(); i++) {
        input = packet;
        assert(!crypto.decrypt(input.data(), i, json));
    }
    input = commandPacket("invalid JSON");
    assert(!crypto.decrypt(input.data(), input.size(), json));
    input.assign(2049, 0);
    assert(!crypto.decrypt(input.data(), input.size(), json));
    input = packet;
    assert(!Crypto(std::string(64, 'f').c_str()).decrypt(input.data(), input.size(), json));

    for (const char *invalid : {"{}", "{\"sequence\":2,\"data\":[]}",
        "{\"sequence\":2,\"data\":[65536]}", "{\"sequence\":2,\"data\":[-1]}",
        "{\"sequence\":2,\"data\":[0]}", "{\"sequence\":2,\"data\":[1.5]}",
        "{\"sequence\":2,\"data\":[\"100\"]}", "{\"sequence\":-1,\"data\":[100]}",
        "{\"sequence\":4294967295,\"data\":[100]}"}) {
        assert(!deserializeJson(json, invalid));
        assert(!application.consumeCommand(json));
    }
    json.clear(); json["sequence"] = 2;
    auto data = json.createNestedArray("data");
    for (size_t i = 0; i < 301; i++) data.add(65535);
    assert(!application.consumeCommand(json));
    application.reportStatus(json);
    assert(json["last_command_sequence_number"].as<uint32_t>() == 1);
    json.clear(); json["sequence"] = 2; data = json.createNestedArray("data");
    for (size_t i = 0; i < 300; i++) data.add(65535);
    assert(application.consumeCommand(json));
    assert(transmissions == 2 && transmittedLength == 300);

    json.clear(); json["last_command_sequence_number"] = 42;
    char status[2048];
    size_t length = crypto.encrypt(json, status, sizeof(status));
    assert(length == 77);
    status[77] = 99;
    assert(crypto.encrypt(json, status, 77) == 77);
    assert(status[77] == 99);
    assert(!crypto.decrypt(status, length, json));
    assert(crypto.encrypt(json, status, 41) == 0);
    randomFails = true;
    assert(crypto.encrypt(json, status, sizeof(status)) == 0);
    randomFails = false;
    length = crypto.encrypt(json, status, sizeof(status));

    const std::string directory = argv[1];
    if (argc == 3 && std::string(argv[2]) == "--write-fixtures") {
        std::ofstream(directory + "/command.hex") << hex(packet.data(), packet.size()) << '\n';
        std::ofstream(directory + "/status.hex") << hex(status, length) << '\n';
    } else {
        std::string expected;
        std::ifstream(directory + "/command.hex") >> expected;
        assert(expected == hex(packet.data(), packet.size()));
        std::ifstream(directory + "/status.hex") >> expected;
        assert(expected == hex(status, length));
    }
    std::cout << "Firmware security checks passed: authentication, tampering, bounds, schema, deduplication, RNG failure\n";
}
