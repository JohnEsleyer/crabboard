#ifndef ARDUINO_H
#define ARDUINO_H

#include <stdint.h>

#define LOW  0
#define HIGH 1

#define INPUT        0
#define OUTPUT       1
#define INPUT_PULLUP 2

#ifdef __cplusplus
extern "C" {
#endif

// These link directly to our Rust engine
void crabboard_pin_mode(uint8_t pin, uint8_t mode);
void crabboard_digital_write(uint8_t pin, uint8_t val);
uint8_t crabboard_digital_read(uint8_t pin);
void crabboard_delay(uint32_t ms);

// Arduino standard API aliases
inline void pinMode(uint8_t pin, uint8_t mode) { crabboard_pin_mode(pin, mode); }
inline void digitalWrite(uint8_t pin, uint8_t val) { crabboard_digital_write(pin, val); }
inline uint8_t digitalRead(uint8_t pin) { return crabboard_digital_read(pin); }
inline void delay(uint32_t ms) { crabboard_delay(ms); }

// Sketch entry points exposed to Rust
void setup(void);
void loop_sketch(void);

#ifdef __cplusplus
}
#endif

#endif // ARDUINO_H
