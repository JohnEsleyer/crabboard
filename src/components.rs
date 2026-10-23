#![allow(dead_code)]

use crate::core::PinState;

pub trait Device: Send {
    fn name(&self) -> &str;
    fn pin(&self) -> u8;
    fn on_pin_changed(&mut self, state: PinState) -> PhysicsResult;
}

#[derive(Debug, PartialEq, Eq)]
pub enum PhysicsResult {
    Nominal { current_draw_ma: u32 },
    BurnedOut { reason: String },
}

pub struct VirtualLed {
    pub name: String,
    pub pin: u8,
    pub has_resistor: bool,
    pub is_lit: bool,
    pub is_burned_out: bool,
}

impl VirtualLed {
    pub fn new(name: String, pin: u8, has_resistor: bool) -> Self {
        Self {
            name,
            pin,
            has_resistor,
            is_lit: false,
            is_burned_out: false,
        }
    }

    pub fn on_pin_changed(&mut self, state: PinState) -> PhysicsResult {
        if self.is_burned_out {
            self.is_lit = false;
            return PhysicsResult::Nominal { current_draw_ma: 0 };
        }

        match state {
            PinState::High => {
                if !self.has_resistor {
                    // 💥 Magic Smoke Rule: LED directly exposed to 5V!
                    self.is_burned_out = true;
                    self.is_lit = false;
                    PhysicsResult::BurnedOut {
                        reason: format!(
                            "💥 POP! {} on Pin {} had no current-limiting resistor! Excess current destroyed diode.",
                            self.name, self.pin
                        ),
                    }
                } else {
                    // Nominal: LED with a ~220 ohm resistor draws ~18-20mA
                    self.is_lit = true;
                    PhysicsResult::Nominal { current_draw_ma: 20 }
                }
            }
            PinState::Low => {
                self.is_lit = false;
                PhysicsResult::Nominal { current_draw_ma: 0 }
            }
        }
    }
}

impl Device for VirtualLed {
    fn name(&self) -> &str {
        &self.name
    }

    fn pin(&self) -> u8 {
        self.pin
    }

    fn on_pin_changed(&mut self, state: PinState) -> PhysicsResult {
        self.on_pin_changed(state)
    }
}

pub struct VirtualButton {
    pub name: String,
    pub pin: u8,
    pub is_pressed: bool,
}

impl VirtualButton {
    pub fn new(name: String, pin: u8) -> Self {
        Self {
            name,
            pin,
            is_pressed: false,
        }
    }

    pub fn set_pressed(&mut self, pressed: bool) -> PinState {
        self.is_pressed = pressed;
        if pressed {
            PinState::Low
        } else {
            PinState::High
        }
    }
}

impl Device for VirtualButton {
    fn name(&self) -> &str {
        &self.name
    }

    fn pin(&self) -> u8 {
        self.pin
    }

    fn on_pin_changed(&mut self, _state: PinState) -> PhysicsResult {
        PhysicsResult::Nominal { current_draw_ma: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_led_burnout_without_resistor() {
        let mut led = VirtualLed::new("RedLed".into(), 13, false);
        let res = led.on_pin_changed(PinState::High);
        match res {
            PhysicsResult::BurnedOut { reason } => {
                assert!(reason.contains("no current-limiting resistor"));
            }
            _ => panic!("Expected LED to burn out without resistor!"),
        }
        assert!(led.is_burned_out);
        assert!(!led.is_lit);

        // Even if driven LOW and HIGH again, stays burned out
        let res2 = led.on_pin_changed(PinState::Low);
        assert_eq!(res2, PhysicsResult::Nominal { current_draw_ma: 0 });
        let res3 = led.on_pin_changed(PinState::High);
        assert_eq!(res3, PhysicsResult::Nominal { current_draw_ma: 0 });
        assert!(!led.is_lit);
    }

    #[test]
    fn test_led_with_resistor_nominal() {
        let mut led = VirtualLed::new("RedLed".into(), 13, true);
        let res = led.on_pin_changed(PinState::High);
        assert_eq!(res, PhysicsResult::Nominal { current_draw_ma: 20 });
        assert!(led.is_lit);
        assert!(!led.is_burned_out);

        let res2 = led.on_pin_changed(PinState::Low);
        assert_eq!(res2, PhysicsResult::Nominal { current_draw_ma: 0 });
        assert!(!led.is_lit);
    }
}
