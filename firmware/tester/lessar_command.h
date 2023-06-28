#include "result.h"
#include "debug.h"

#define abs(x) ((x)>0?(x):-(x))

class LessarCommand {
public:
    LessarCommand() {
        m_command[0] = 0;
        m_command[1] = 0;
        m_command[2] = 0;
    }

    LessarCommand(int command[3]) {
        m_command[0] = command[0];
        m_command[1] = command[1];
        m_command[2] = command[2];
    }

    LessarCommand(const LessarCommand &other) {
        m_command[0] = other.m_command[0];
        m_command[1] = other.m_command[1];
        m_command[2] = other.m_command[2];
    }

    void printBits(char * buffer) {
        // prints the command in binary format MSB first, with spaces between bytes
        for (int i = 0; i < 3; i++) {
            for (int j = 7; j >= 0; j--) {
                buffer[9*i + 7 - j] = (m_command[i] & (1 << j)) ? '1' : '0';
            }
            buffer[9*i + 8] = ' ';
        }

        buffer[9*3] = '\0';
    }

    static Result<LessarCommand> decodeFromRaw(uint16_t *buffer, int length) {
        int commandBytes[12] = {0, 0, 0, 0, 0, 0, 0, 0, 0};
        // offset as they appear in input stream
        int bitOffset = 0;
        int prelude = 0;

        for(int i = 0; i < length; i++) {
            int val = closestValue(buffer[i]);
            //Serial.print("closest value ");
            //Serial.print(buffer[i]);
            //Serial.print(" -> ");
            //Serial.println(val);

            if (val == NEC_INITIATOR || val == NEC_FILLER) {
                // only valid on byte boundaries
                if (bitOffset % 8 != 0) {
                    return Result<LessarCommand>::error("NEC_INITIATOR or NEC_FILLER not on byte boundary");
                }

                prelude = 0;
                continue;
            }

            if (prelude == 0) {
                if (val != NEC_SHORT) {
                    return Result<LessarCommand>::error("NEC_SHORT expected when prelude is 0");
                }
            } else {
                int bit = 0;

                if (val == NEC_SHORT) {
                    bit = 0;
                } else if (val == NEC_LONG) {
                    bit = 1;
                } else {
                    return Result<LessarCommand>::error("NEC_SHORT or NEC_LONG expected when prelude is 1");
                };

                if (bitOffset >= sizeof(commandBytes) / sizeof(commandBytes[0]) * 8) {
                    //Serial.println("command too long. we only support 12 bytes");
                    //Serial.println(bitOffset);
                    //Serial.println(i);
                    return Result<LessarCommand>::error("comand too long. we only support 12 bytes");
                }

                commandBytes[bitOffset / 8] |= (bit << (7 - bitOffset % 8));
                bitOffset++;
            }

            prelude = 1 - prelude;
        }

        debug("command bytes: %x %x %x %x %x %x %x %x %x %x %x %x\n",
            commandBytes[0], commandBytes[1], commandBytes[2], commandBytes[3],
            commandBytes[4], commandBytes[5], commandBytes[6], commandBytes[7],
            commandBytes[8], commandBytes[9], commandBytes[10], commandBytes[11]);

        /// validating that command has format a !a b !b c !c a !a b !b c !c
        for (int i = 0; i < 6; i+=2) {
           if (commandBytes[i] != commandBytes[i + 6]) {
               return Result<LessarCommand>::error("INVALID COMMAND ( a != repeat(a) )");
           }

           if (commandBytes[i+1] != commandBytes[i + 7]) {
               return Result<LessarCommand>::error("INVALID COMMAND ( a != repeat(a) )");
           }

           if (commandBytes[i] != (~commandBytes[i + 1] & 0xFF)) {
               return Result<LessarCommand>::error("INVALID COMMAND (a != !a)");
           }
        }

        int command[3] = {commandBytes[0], commandBytes[2], commandBytes[4]};
        return Result<LessarCommand>::ok(LessarCommand(command));
    }

private:
    static const int NEC_SHORT = 562;
    static const int NEC_LONG = 1687;
    static const int NEC_INITIATOR = 4300;
    static const int NEC_FILLER = 9000 - NEC_INITIATOR;

    static int closestValue(int x) {
        const int SIZE = 4;
        int candidates[SIZE] = {NEC_SHORT, NEC_LONG, NEC_INITIATOR, NEC_FILLER};
        int closest = candidates[0];
        for (int i = 0; i < SIZE; i++) {
            if (abs(candidates[i] - x) < abs(closest - x)) {
                closest = candidates[i];
            }
        }

        return closest;
    }

    int m_command[3];
};