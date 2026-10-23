mod circuit;
mod components;
mod core;
mod ffi;

use circuit::CircuitSpec;
use components::{PhysicsResult, VirtualButton, VirtualLed};
use core::VirtualMcu;
use ffi::{loop_sketch, setup, GLOBAL_MCU};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spec = CircuitSpec::load_from_file("circuit.spec").expect("Failed to load spec");

    let led_comp = spec
        .components
        .iter()
        .find(|c| c.kind == circuit::ComponentKind::Led)
        .cloned()
        .unwrap_or(circuit::Component {
            name: "StatusLed".into(),
            kind: circuit::ComponentKind::Led,
            pin: 13,
            has_resistor: true,
        });

    let btn_comp = spec
        .components
        .iter()
        .find(|c| c.kind == circuit::ComponentKind::Button)
        .cloned()
        .unwrap_or(circuit::Component {
            name: "ActionBtn".into(),
            kind: circuit::ComponentKind::Button,
            pin: 2,
            has_resistor: false,
        });

    let led_pin = led_comp.pin;
    let button_pin = btn_comp.pin;

    let led = Arc::new(Mutex::new(VirtualLed::new(
        led_comp.name,
        led_pin,
        led_comp.has_resistor,
    )));
    let button = Arc::new(Mutex::new(VirtualButton::new(
        btn_comp.name,
        button_pin,
    )));

    // 1. Initialize Virtual MCU
    let mut mcu = VirtualMcu::new();

    // Hook LED state change & physics simulation
    let led_clone = Arc::clone(&led);
    let failure_msg = Arc::new(Mutex::new(None::<String>));
    let failure_msg_clone = Arc::clone(&failure_msg);

    mcu.subscribe(move |event| {
        if let core::SimulationEvent::PinStateChanged { pin, state } = event {
            let mut l = led_clone.lock().unwrap();
            if l.pin == *pin {
                match l.on_pin_changed(*state) {
                    PhysicsResult::BurnedOut { reason } => {
                        let mut f = failure_msg_clone.lock().unwrap();
                        if f.is_none() {
                            *f = Some(reason);
                        }
                    }
                    PhysicsResult::Nominal { .. } => {}
                }
            }
        }
    });

    // Share MCU with C++ FFI
    *GLOBAL_MCU.lock().unwrap() = Some(mcu);

    enable_raw_mode()?;
    print!("{}[2J", 27 as char);

    let running = Arc::new(Mutex::new(true));

    // --- Execute C++ Arduino Setup ---
    unsafe {
        setup();
    }

    // --- THREAD 1: Runs C++ loop_sketch() continuously ---
    let running_firmware = Arc::clone(&running);
    let firmware_handle = std::thread::spawn(move || {
        while *running_firmware.lock().unwrap() {
            unsafe {
                loop_sketch();
            }
        }
    });

    // --- THREAD 2: User Keyboard & Terminal View ---
    println!("🦀 CrabBoard Running Native C++ Firmware with Heuristic Physics!");
    println!("Controls: [SPACE] = Hold/Release Button | [Q] = Exit\n\r");

    let mut btn_pressed = false;

    while *running.lock().unwrap() {
        if let Some(msg) = failure_msg.lock().unwrap().take() {
            println!("\r\n💥 CRITICAL HEURISTIC FAILURE:\r\n{}\r\n", msg);
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(' ') => {
                            btn_pressed = !btn_pressed;
                            let new_pin_state = button.lock().unwrap().set_pressed(btn_pressed);

                            // Update MCU state for C++ digitalRead()
                            if let Ok(mut guard) = GLOBAL_MCU.lock() {
                                if let Some(mcu) = guard.as_mut() {
                                    mcu.set_external_pin_input(button_pin, new_pin_state);
                                }
                            }
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            *running.lock().unwrap() = false;
                        }
                        _ => {}
                    }
                }
            }
        }

        let is_browned_out = GLOBAL_MCU
            .lock()
            .unwrap()
            .as_ref()
            .map(|m| m.is_browned_out)
            .unwrap_or(false);

        let led_guard = led.lock().unwrap();
        let led_symbol = if is_browned_out {
            "⚡ [BROWNOUT - MCU HALTED]                             ".to_string()
        } else if led_guard.is_burned_out {
            "💥 [BURNED OUT]                                        ".to_string()
        } else if led_guard.is_lit {
            "🔴 [ON] (Drawing 20mA / Total: 20mA / 200mA limit)     ".to_string()
        } else {
            "⚪ [OFF] (Drawing 0mA / Total: 0mA / 200mA limit)      ".to_string()
        };
        let btn_symbol = if btn_pressed { "⬇️ [PRESSED]" } else { "⬆️ [OPEN]   " };

        print!(
            "\r[C++ RUNNING] Button (Pin {}): {}  |  LED (Pin {}): {}  ",
            button_pin, btn_symbol, led_pin, led_symbol
        );
        std::io::stdout().flush().unwrap();
    }

    firmware_handle.join().unwrap();
    disable_raw_mode()?;
    println!("\n\r\n🦀 Clean exit.");
    Ok(())
}
