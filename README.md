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

```

10110010 01101011 11100000 

10110010 01101011 11100000 

10110010 01101011 11100000 

10110010 01101011 11100000 

10110010 10011111 00010000 


codes:

0000 0000 0000 0000
m000 0000 0000 0000
0000 0000 tttt mm00


tttt = 17 -> 0000
tttt = 18 -> 0001
tttt = 19 -> 0011
tttt = 20 -> 0010
tttt = 21 -> 0110
tttt = 22 -> 0111
tttt = 23 -> 0101
tttt = 24 -> 0100
tttt = 25 -> 1100
tttt = 26 -> 1101
tttt = 27 -> 1001
tttt = 28 -> 1000
tttt = 29 -> 1010
tttt = 30 -> 1011

mmm = water -> 001
mmm = sun   -> 111
mmm = fan   -> 101
mmm = auto  -> 010
mmm = cold  -> 100
    
```