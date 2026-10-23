#include "include/Arduino.h"

const int LED_PIN = 13;
const int BUTTON_PIN = 2;

void setup() {
    pinMode(LED_PIN, OUTPUT);
    pinMode(BUTTON_PIN, INPUT_PULLUP);
}

// Named loop_sketch to avoid C++ 'loop' keyword conflicts
void loop_sketch() {
    int buttonState = digitalRead(BUTTON_PIN);

    if (buttonState == LOW) {
        digitalWrite(LED_PIN, HIGH);
    } else {
        digitalWrite(LED_PIN, LOW);
    }

    delay(50);
}
