FROM ubuntu:20.04
RUN apt-get update && apt-get install -y \
    curl \
    git \
    make \
    python3 \
    vim \
    g++

RUN curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | sh
RUN arduino-cli config init --additional-urls https://arduino.esp8266.com/stable/package_esp8266com_index.json
RUN arduino-cli core update-index
RUN arduino-cli core install arduino:avr
RUN arduino-cli core install esp8266:esp8266

RUN arduino-cli lib install IRremoteESP8266
#RUN arduino-cli lib install IRremote
RUN arduino-cli lib install ArduinoJson
