#![allow(dead_code)]

use crate::core::{PinMode, PinState, VirtualMcu};
use std::sync::{Arc, Mutex};

// A global handle so C++ functions can talk to the active Rust MCU
lazy_static::lazy_static! {
    pub static ref GLOBAL_MCU: Arc<Mutex<Option<VirtualMcu>>> = Arc::new(Mutex::new(None));
}

#[no_mangle]
pub extern "C" fn crabboard_pin_mode(pin: u8, mode: u8) {
    let pin_mode = match mode {
        0 => PinMode::Input,
        1 => PinMode::Output,
        2 => PinMode::InputPullup,
        _ => PinMode::Input,
    };

    if let Ok(mut guard) = GLOBAL_MCU.lock() {
        if let Some(mcu) = guard.as_mut() {
            mcu.pin_mode(pin, pin_mode);
        }
    }
}

#[no_mangle]
pub extern "C" fn crabboard_digital_write(pin: u8, val: u8) {
    let state = if val == 1 {
        PinState::High
    } else {
        PinState::Low
    };
    if let Ok(mut guard) = GLOBAL_MCU.lock() {
        if let Some(mcu) = guard.as_mut() {
            mcu.digital_write(pin, state);
        }
    }
}

#[no_mangle]
pub extern "C" fn crabboard_digital_read(pin: u8) -> u8 {
    if let Ok(guard) = GLOBAL_MCU.lock() {
        if let Some(mcu) = guard.as_ref() {
            return mcu.digital_read(pin) as u8;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn crabboard_delay(ms: u32) {
    if let Ok(mut guard) = GLOBAL_MCU.lock() {
        if let Some(mcu) = guard.as_mut() {
            mcu.delay(ms as u64);
        }
    }
}

// Declarations of functions implemented in the user's C++ sketch
extern "C" {
    pub fn setup();
    pub fn loop_sketch();
}
