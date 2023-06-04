#define RAW_BUFFER_LENGTH 750
#define RECORD_GAP_MICROS 16000
#define DEBUG

//------------------------------------------------------------------------------
// Include the IRremote library header
//
#include <IRremote.h>

//------------------------------------------------------------------------------
// Tell IRremote which Arduino pin is connected to the IR Receiver (TSOP4838)
//
int recvPin = 11;
IRrecv irrecv(recvPin);

//+=============================================================================
// Configure the Arduino
//
void  setup ( )
{
  Serial.begin(9600);   // Status message will be sent to PC at 9600 baud
  irrecv.enableIRIn();  // Start the receiver
}

//+=============================================================================
// Display IR code
//
void  ircode (decode_results *results)
{
  // Panasonic has an Address
  if (results->decode_type == PANASONIC) {
    Serial.print(results->address, HEX);
    Serial.print(":");
  }

  // Print Code
  Serial.print(results->value, HEX);
}

//+=============================================================================
// Display encoding type
//
void  encoding(decode_type_t x)
{
  switch (x) {
    default:
    case UNKNOWN:      Serial.print("UNKNOWN");       break ;
    case NEC:          Serial.print("NEC");           break ;
    case SONY:         Serial.print("SONY");          break ;
    case RC5:          Serial.print("RC5");           break ;
    case RC6:          Serial.print("RC6");           break ;
//    case DISH:         Serial.print("DISH");          break ;
    case SHARP:        Serial.print("SHARP");         break ;
    case JVC:          Serial.print("JVC");           break ;
 //   case SANYO:        Serial.print("SANYO");         break ;
//    case MITSUBISHI:   Serial.print("MITSUBISHI");    break ;
    case SAMSUNG:      Serial.print("SAMSUNG");       break ;
    case LG:           Serial.print("LG");            break ;
    case WHYNTER:      Serial.print("WHYNTER");       break ;
 //   case AIWA_RC_T501: Serial.print("AIWA_RC_T501");  break ;
    case PANASONIC:    Serial.print("PANASONIC");     break ;
    case DENON:        Serial.print("Denon");         break ;
  }
}

//+=============================================================================
// Dump out the decode_results structure.
//
void  dumpInfo (decode_results *results)
{
  // Check if the buffer overflowed
  if (results->overflow) {
    Serial.println("IR code too long. Edit IRremoteInt.h and increase RAWBUF");
    return;
  }

  // Show Encoding standard
  Serial.print("Encoding  : ");
  encoding(results->decode_type);
  Serial.println("");

  // Show Code & length
  Serial.print("Code      : ");
  ircode(results);
  Serial.print(" (");
  Serial.print(results->bits, DEC);
  Serial.println(" bits)");
}

//+=============================================================================
// Dump out the decode_results structure.
//
void  dumpRaw (irparams_struct *results)
{
  // Print Raw data
  Serial.print("Timing[");
  Serial.print(results->rawlen-1, DEC);
  Serial.println("]: ");

  for (int i = 1;  i < results->rawlen;  i++) {
    unsigned long  x = results->rawbuf[i] * USECPERTICK;
    if (!(i & 1)) {  // even
      Serial.print("-");
      if (x < 1000)  Serial.print(" ") ;
      if (x < 100)   Serial.print(" ") ;
      Serial.print(x, DEC);
    } else {  // odd
      Serial.print("     ");
      Serial.print("+");
      if (x < 1000)  Serial.print(" ") ;
      if (x < 100)   Serial.print(" ") ;
      Serial.print(x, DEC);
      if (i < results->rawlen-1) Serial.print(", "); //',' not needed for last one
    }
    if (!(i % 8))  Serial.println("");
  }
  Serial.println("");                    // Newline
}

//+=============================================================================
// Dump out the decode_results structure.
//
void  dumpCode (irparams_struct *results)
{
  // Start declaration
  Serial.print("unsigned int  ");          // variable type
  Serial.print("rawData[");                // array name
  Serial.print(results->rawlen - 1, DEC);  // array size
  Serial.print("] = {");                   // Start declaration

  // Dump data
  for (int i = 1;  i < results->rawlen;  i++) {
    Serial.print(results->rawbuf[i] * USECPERTICK, DEC);
    if ( i < results->rawlen-1 ) Serial.print(","); // ',' not needed on last one
    if (!(i & 1))  Serial.print(" ");
  }

  // End declaration
  Serial.print("};");  // 

  // Newline
  Serial.println("");
}

void  dumpInfo2(IRData *results)
{
  // Show Encoding standard
  Serial.print("Encoding  : ");
  encoding(results->protocol);
  Serial.println("");

  // Show Code & length
  Serial.print("Code      : ");
  Serial.print(results->decodedRawData, HEX);
  Serial.print(" (");
  Serial.print(results->numberOfBits, DEC);
  Serial.println(" bits)");

  Serial.print("Flags     : ");
  Serial.println(results->flags, HEX);

  Serial.print("Address  : ");
  Serial.println(results->address, HEX);


  Serial.print("Command  : ");
  Serial.println(results->command, HEX);

  Serial.print("Bits     : ");
  Serial.println(results->numberOfBits, DEC);

  Serial.print("HeaderMarkMicros   : ");
  Serial.println(results->DistanceWidthTimingInfo.HeaderMarkMicros, DEC);

  Serial.print("HeaderSpaceMicros   : ");
  Serial.println(results->DistanceWidthTimingInfo.HeaderSpaceMicros, DEC);

  Serial.print("ZeroMarkMicros   : ");
  Serial.println(results->DistanceWidthTimingInfo.ZeroMarkMicros, DEC);

  Serial.print("ZeroSpaceMicros   : ");
  Serial.println(results->DistanceWidthTimingInfo.ZeroSpaceMicros, DEC);

  Serial.print("OneMarkMicros   : ");
  Serial.println(results->DistanceWidthTimingInfo.OneMarkMicros, DEC);

  Serial.print("OneSpaceMicros   : ");
  Serial.println(results->DistanceWidthTimingInfo.OneSpaceMicros, DEC);

  Serial.print("rawlen   : ");
  Serial.println(results->rawDataPtr->rawlen, DEC);

  Serial.print("OverflowFlag   : ");
  Serial.println(results->rawDataPtr->OverflowFlag, DEC);

  Serial.print("ssiiizeee   : ");
  Serial.println(sizeof(results->rawDataPtr->rawbuf)/sizeof(results->rawDataPtr->rawbuf[0]), DEC);

  dumpRaw(results->rawDataPtr);
  dumpCode(results->rawDataPtr);
}

//+=============================================================================
// The repeating section of the code
//
void  loop ( )
{

  if (irrecv.decode()) {
    dumpInfo2(&IrReceiver.decodedIRData);
    Serial.println("");
    irrecv.resume();
  }

  // decode_results  results;        // Somewhere to store the results


  // if (irrecv.decode(&results)) {  // Grab an IR code
  //   dumpInfo(&results);           // Output the results
  //   dumpRaw(&results);            // Output the results in RAW format
  //   dumpCode(&results);           // Output the results as source code
  //   Serial.println("");           // Blank line between entries
  //   irrecv.resume();              // Prepare for the next value
  // }
}