#include "Arduino.h"
#include <ESP8266WiFi.h>
#include <WiFiUdp.h>
#include "logger.h"
#include "application.h"
#include "network.h"
#include "crypto.h"

#ifndef FIRMWARE_WIFI_SSID
    #error "FIRMWARE_WIFI_SSID macro is not defined!"
#endif

#ifndef FIRMWARE_WIFI_PASS
    #error "FIRMWARE_WIFI_PASS macro is not defined!"
#endif

#ifndef FIRMWARE_REMOTE_HOST
    #error "FIRMWARE_REMOTE_HOST macro is not defined!"
#endif

#ifndef FIRMWARE_SHARED_SECRET
    #error "FIRMWARE_SHARED_SECRET macro is not defined!"
#endif

#define LOCAL_UDP_PORT 4944
#define REMOTE_UDP_PORT 4944

#define LED_PIN 2 
#define IDLE_PING_INTERVAL (5 * 1000) // 5 seconds

Network network(FIRMWARE_WIFI_SSID, FIRMWARE_WIFI_PASS, LOCAL_UDP_PORT, REMOTE_UDP_PORT, FIRMWARE_REMOTE_HOST);
Application application(LED_PIN);
Crypto crypto(FIRMWARE_SHARED_SECRET);

char iobuffer[2048];
DynamicJsonDocument json(8 * 1024);
uint64 nextTimeSendStatus = 0;


void setup() {
    pinMode(LED_PIN, OUTPUT);
    delay(10000);

    Logger.println("Starting up...");
    Logger.println("WIFI_SSID: " + String(FIRMWARE_WIFI_SSID));
    Logger.println("WIFI_PASS: " + String(FIRMWARE_WIFI_PASS));
    Logger.println("LOCAL_UDP_PORT: " + String(LOCAL_UDP_PORT));
    Logger.println("REMOTE_UDP_PORT: " + String(REMOTE_UDP_PORT));
    Logger.println("remoteHost: " + String(FIRMWARE_REMOTE_HOST));
    
    network.Connect();
}

void loop() {   
    uint64 now = millis();
    size_t len = network.Receive(iobuffer, sizeof(iobuffer));
    
    if (len > 0) {
        bool successfullDecrypt = crypto.decrypt(iobuffer, len, json);
        if (successfullDecrypt) {
            application.consumeCommand(json);
            nextTimeSendStatus = now;
        }
    }

    if (nextTimeSendStatus <= now) {
        nextTimeSendStatus = now + IDLE_PING_INTERVAL;
        application.reportStatus(json);
        size_t len = crypto.encrypt(json, iobuffer, sizeof(iobuffer));
        network.Send(iobuffer, len);
    }   
}