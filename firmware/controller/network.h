#include <Arduino.h>
#include <ESP8266WiFi.h>
#include <WiFiUdp.h>
#include "logger.h"

#ifndef NETWORK_H
#define NETWORK_H

class Network {
    public:
        Network(const char *ssid, const char *pass, uint16_t localPort, uint16_t remotePort, const char *remoteDomain) {
            this->ssid = ssid;
            this->pass = pass;
            this->localPort = localPort;
            this->remotePort = remotePort;
            this->remoteDomain = remoteDomain;
        }

        void Connect() {
            WiFi.mode(WIFI_STA);


            // Begin WiFi
            WiFi.begin(this->ssid, this->pass);

            // Connecting to WiFi...
            Logger.print("Connecting to ");
            Logger.println(this->ssid);
            // Loop continuously while WiFi is not connected

            while (WiFi.status() != WL_CONNECTED) {
                delay(100);
                Logger.print(".");
            }

            // Connected to WiFi
            Logger.print("Connected! IP address: ");
            Logger.println(WiFi.localIP());

            WiFi.setAutoReconnect(true);
            WiFi.persistent(true);

            if (this->remoteIP.fromString(this->remoteDomain)) {
                Logger.print("Resolved remote IP Address: ");
                Logger.println(this->remoteIP);
            } else {
                Logger.print("Resolving remote host.... ");
                while (!WiFi.hostByName(this->remoteDomain, this->remoteIP)) {
                    delay(100);
                    Logger.print(".");
                }

                Logger.print("Resolved remote host: ");
                Logger.println(this->remoteIP);
            }

            // Begin listening to UDP port
            this->udp.begin(this->localPort);
            Logger.print("Listening on UDP port ");
            Logger.println(this->localPort);
        }

        void Send(char *buffer, size_t len) {
            Logger.print("Sedning packet... len: ");
            Logger.println(len);
            Logger.print("Current local IP: ");
            Logger.println(WiFi.localIP());

            this->udp.beginPacket(this->remoteIP, this->remotePort);
            this->udp.write(buffer, len);
            this->udp.endPacket();
        }

        int Receive(char *buffer, size_t len) {
            int packetSize = this->udp.parsePacket();
            if (packetSize) {
                Logger.print("Received packet of size: ");
                Logger.println(packetSize);
                int bytesRead = this->udp.read(buffer, len);
                Logger.println(bytesRead);
                return bytesRead;
            }
            return 0;
        }

    private:
        WiFiUDP udp;
        const char *ssid;
        const char *pass;
        uint16_t localPort;
        const char *remoteDomain;
        IPAddress remoteIP;
        uint16_t remotePort;
};

#endif
