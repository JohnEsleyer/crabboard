#![allow(dead_code)]

use pest::Parser;
use pest_derive::Parser;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[grammar = "circuit.pest"]
pub struct CircuitParser;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ComponentKind {
    Led,
    Button,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub kind: ComponentKind,
    pub pin: u8,
    pub has_resistor: bool,
}

#[derive(Debug)]
pub struct CircuitSpec {
    pub mcu_name: String,
    pub components: Vec<Component>,
}

impl CircuitSpec {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read circuit spec file: {}", e))?;
        Self::parse_spec(&content)
    }

    pub fn parse_spec(input: &str) -> Result<Self, String> {
        let pairs = CircuitParser::parse(Rule::circuit_file, input)
            .map_err(|e| format!("Syntax Error in circuit.spec:\n{}", e))?;

        let mut mcu_name = String::from("GenericMCU");
        let mut components = Vec::new();

        for pair in pairs {
            for inner in pair.into_inner() {
                if let Rule::statement = inner.as_rule() {
                    let stmt = inner.into_inner().next().unwrap();
                    match stmt.as_rule() {
                        Rule::mcu_decl => {
                            let id = stmt.into_inner().next().unwrap();
                            mcu_name = id.as_str().to_string();
                        }
                        Rule::component_decl => {
                            let mut parts = stmt.into_inner();
                            let kind_rule = parts.next().unwrap();
                            let id_rule = parts.next().unwrap();
                            let pin_target_rule = parts.next().unwrap();
                            let resistor_rule = parts.next(); // Optional

                            let kind = match kind_rule.as_str() {
                                "LED" => ComponentKind::Led,
                                "BUTTON" => ComponentKind::Button,
                                other => return Err(format!("Unknown component: {}", other)),
                            };

                            let name = id_rule.as_str().to_string();
                            let pin_str = pin_target_rule
                                .into_inner()
                                .next()
                                .unwrap()
                                .as_str();

                            let pin: u8 = pin_str.parse().map_err(|_| {
                                format!("Invalid pin number '{}' for {}", pin_str, name)
                            })?;

                            let has_resistor = resistor_rule.is_some();

                            components.push(Component {
                                name,
                                kind,
                                pin,
                                has_resistor,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        // Circuit validation rule: Check for pin collisions
        let mut used_pins = std::collections::HashSet::new();
        for comp in &components {
            if !used_pins.insert(comp.pin) {
                return Err(format!(
                    "💥 Pin Conflict: Multiple components attached to pin {}",
                    comp.pin
                ));
            }
        }

        Ok(CircuitSpec {
            mcu_name,
            components,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_spec_parsing_with_resistor() {
        let spec_str = r#"
            MCU ArduinoUno
            LED StatusLed -> pin(13) [resistor: 220R]
            BUTTON ActionBtn -> pin(2)
        "#;
        let parsed = CircuitSpec::parse_spec(spec_str).expect("Should parse valid spec");
        assert_eq!(parsed.mcu_name, "ArduinoUno");
        assert_eq!(parsed.components.len(), 2);
        assert_eq!(parsed.components[0].name, "StatusLed");
        assert_eq!(parsed.components[0].kind, ComponentKind::Led);
        assert_eq!(parsed.components[0].pin, 13);
        assert!(parsed.components[0].has_resistor);
        assert_eq!(parsed.components[1].name, "ActionBtn");
        assert_eq!(parsed.components[1].kind, ComponentKind::Button);
        assert_eq!(parsed.components[1].pin, 2);
        assert!(!parsed.components[1].has_resistor);
    }

    #[test]
    fn test_pin_conflict_detection() {
        let spec_str = r#"
            MCU ArduinoUno
            LED Led1 -> pin(13)
            LED Led2 -> pin(13)
        "#;
        let err = CircuitSpec::parse_spec(spec_str).unwrap_err();
        assert!(err.contains("Pin Conflict"));
    }

    #[test]
    fn test_invalid_syntax() {
        let spec_str = r#"
            LED Led1 -- pin(13)
        "#;
        assert!(CircuitSpec::parse_spec(spec_str).is_err());
    }
}
