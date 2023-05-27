# IR Remote Control 

It is a small project to control an air conditioner via internet using mobile phone.

## Architecture

#### High level schema

```mermaid
graph LR
    A(WebClient) --> B(Backend Server)
    B --> C(ESP8266)
    C --> D(IR LED)
    D --> E(Air Conditioner)
```
