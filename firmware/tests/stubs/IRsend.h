#pragma once
#include <cstddef>
#include <cstdint>
extern size_t transmissions;
extern size_t transmittedLength;
class IRsend {
public:
    IRsend(int, bool) {}
    void begin() {}
    void sendRaw(uint16_t *, size_t length, int) { transmissions++; transmittedLength = length; }
};
