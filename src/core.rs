#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinState {
    Low = 0,
    High = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinMode {
    Input,
    Output,
    InputPullup,
}

#[derive(Debug, Clone)]
pub enum SimulationEvent {
    PinStateChanged { pin: u8, state: PinState },
    PinModeChanged { pin: u8, mode: PinMode },
    BrownoutTriggered { total_ma: u32 },
}

pub struct VirtualMcu {
    pub pin_modes: HashMap<u8, PinMode>,
    pub pin_states: HashMap<u8, PinState>,
    pub pin_current_draw: HashMap<u8, u32>,
    pub is_browned_out: bool,
    current_time_ms: u64,
    event_subscribers: Vec<Box<dyn Fn(&SimulationEvent) + Send>>,
}

impl Default for VirtualMcu {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualMcu {
    pub fn new() -> Self {
        let mut pin_states = HashMap::new();
        let mut pin_modes = HashMap::new();
        let mut pin_current_draw = HashMap::new();

        for pin in 0..=13 {
            pin_states.insert(pin, PinState::Low);
            pin_modes.insert(pin, PinMode::Input);
            pin_current_draw.insert(pin, 0);
        }

        Self {
            pin_modes,
            pin_states,
            pin_current_draw,
            is_browned_out: false,
            current_time_ms: 0,
            event_subscribers: Vec::new(),
        }
    }

    pub fn subscribe<F>(&mut self, callback: F)
    where
        F: Fn(&SimulationEvent) + Send + 'static,
    {
        self.event_subscribers.push(Box::new(callback));
    }

    pub fn emit(&self, event: SimulationEvent) {
        for sub in &self.event_subscribers {
            sub(&event);
        }
    }

    pub fn pin_mode(&mut self, pin: u8, mode: PinMode) {
        self.pin_modes.insert(pin, mode);
        if mode == PinMode::InputPullup {
            self.pin_states.insert(pin, PinState::High);
        }
        self.emit(SimulationEvent::PinModeChanged { pin, mode });
    }

    pub fn digital_write(&mut self, pin: u8, state: PinState) {
        if self.is_browned_out {
            return;
        }
        self.pin_states.insert(pin, state);
        self.emit(SimulationEvent::PinStateChanged { pin, state });
    }

    // --- ⚡ HEURISTIC: FLOATING PIN EMULATION ---
    pub fn digital_read(&self, pin: u8) -> PinState {
        if self.is_browned_out {
            return PinState::Low;
        }

        match self.pin_modes.get(&pin) {
            Some(PinMode::Input) => {
                // Pin is ungrounded / floating! Random electromagnetic noise
                let pseudo_noise = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos() % 2) == 0;

                if pseudo_noise { PinState::High } else { PinState::Low }
            }
            Some(PinMode::InputPullup) | Some(PinMode::Output) => {
                *self.pin_states.get(&pin).unwrap_or(&PinState::Low)
            }
            None => PinState::Low,
        }
    }

    // --- ⚡ HEURISTIC: THERMAL CURRENT BUDGET ---
    pub fn update_pin_current(&mut self, pin: u8, ma: u32) {
        self.pin_current_draw.insert(pin, ma);
        let total_ma: u32 = self.pin_current_draw.values().sum();

        // ATmega328P limit is 200mA total VCC/GND current
        if total_ma > 200 {
            self.is_browned_out = true;
            self.emit(SimulationEvent::BrownoutTriggered { total_ma });
        }
    }

    pub fn total_current(&self) -> u32 {
        self.pin_current_draw.values().sum()
    }

    pub fn set_external_pin_input(&mut self, pin: u8, state: PinState) {
        self.pin_states.insert(pin, state);
        self.emit(SimulationEvent::PinStateChanged { pin, state });
    }

    pub fn delay(&mut self, ms: u64) {
        self.current_time_ms += ms;
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }

    pub fn millis(&self) -> u64 {
        self.current_time_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcu_write_and_read() {
        let mut mcu = VirtualMcu::new();
        mcu.pin_mode(13, PinMode::Output);
        mcu.digital_write(13, PinState::High);
        assert_eq!(mcu.digital_read(13), PinState::High);
    }

    #[test]
    fn test_input_pullup_defaults_to_high() {
        let mut mcu = VirtualMcu::new();
        mcu.pin_mode(2, PinMode::InputPullup);
        assert_eq!(mcu.digital_read(2), PinState::High);
    }

    #[test]
    fn test_thermal_brownout() {
        let mut mcu = VirtualMcu::new();
        assert!(!mcu.is_browned_out);
        mcu.update_pin_current(13, 100);
        assert!(!mcu.is_browned_out);
        mcu.update_pin_current(12, 101);
        assert!(mcu.is_browned_out);
        assert_eq!(mcu.digital_read(13), PinState::Low);
    }
}
