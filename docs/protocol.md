# Protocol and security

Set `FIRMWARE_SHARED_SECRET` to a random 32-byte key encoded as 64 hexadecimal characters
(`openssl rand -hex 32`). The tasks pass the same key to the bot as `IR_SHARED_SECRET`.
Changing the key requires rebuilding/flashing the firmware and restarting the bot with the matching key.
The key is embedded in the firmware; physical access to flash can reveal it.

Both directions use XChaCha20-Poly1305 with a fresh cryptographically random 24-byte nonce for each packet.
The binary format is version (1 byte, currently 1), message type (1 byte: command=1, status=2), nonce (24 bytes),
encrypted UTF-8 JSON, and authentication tag (16 bytes). The version and message type are authenticated as additional
data. Packets exceeding 2048 bytes, plaintext packets, and failed authentication are rejected before changing state.
There is no handshake or persistent cryptographic session state. Firmware uses the ESP8266 SDK random source after
connecting Wi-Fi; the bot uses the operating system random source. Firmware crypto is vendored Monocypher 4.0.2.

Command sequences suppress duplicate IR execution during device uptime. Packet freshness is not guaranteed:
captured commands can execute after device restart or if they were never received, and captured status packets can
affect connectivity, acknowledgement, and routing state. This is an accepted limitation of this personal project.
IR acknowledgement means the transmitter emitted the signal; the appliance does not acknowledge reception.
