#ifdef UNIT_TEST
   #define DEBUG_CONSOLE
#endif

#ifdef DEBUG_SERIAL
   #define debug(...)    Serial.print(__VA_ARGS__)
#endif

#ifdef DEBUG_CONSOLE
   #include "stdio.h"
   #define debug(...)    do { printf(__VA_ARGS__); fflush(stdout); } while (0)
#endif

#ifndef debug
   #define debug(...)    do {} while (0) 
#endif
