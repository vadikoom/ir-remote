#include "Arduino.h"
#include <ESP8266WiFi.h>
#include <WiFiUdp.h>


#ifndef WIFI_SSID
    #error "WIFI_SSID macro is not defined!"
#endif

#ifndef WIFI_PASS
    #error "WIFI_PASS macro is not defined!"
#endif

#define UDP_PORT 4944

// UDP
WiFiUDP UDP;

char buffer[] = "hello world";
char buffer2[200];

void setup() {
    Serial.begin(115200);

    delay(5000);

    // Begin WiFi
    WiFi.begin(WIFI_SSID, WIFI_PASS);
    
    // Connecting to WiFi...
    Serial.print("Connecting to ");
    Serial.print(WIFI_SSID);
    // Loop continuously while WiFi is not connected
    
    while (WiFi.status() != WL_CONNECTED) {
        delay(100);
        Serial.print(".");
    }
    
    // Connected to WiFi
    Serial.println();
    Serial.print("Connected! IP address: ");
    Serial.println(WiFi.localIP());

    // Begin listening to UDP port
    UDP.begin(UDP_PORT);
    Serial.print("Listening on UDP port ");
    Serial.println(UDP_PORT);
    
}

void loop() {
    delay(3000);
    UDP.beginPacket("255.255.255.255", UDP_PORT);
    UDP.write(buffer, sizeof(buffer));
    UDP.endPacket();

    int packetSize = UDP.parsePacket(); 
    if (packetSize) {
        Serial.print("Received packet! Size: ");
        Serial.println(packetSize); 
        int len = UDP.read(buffer2, 255);
        if (len > 0) {
            buffer2[len] = '\0';
        }
        Serial.print("Packet received: ");
        Serial.println(buffer2);
    }

    Serial.println("Sent packet!");
}
