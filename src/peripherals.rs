use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;

#[derive(Deserialize, Clone)]
struct Peripherals {
    peripherals: Vec<Peripheral>,
}

#[derive(Deserialize, Clone)]
struct Peripheral {
    name: String,
    mount: String,
}

static PERIPHERAL_NAMES: OnceCell<HashMap<String, String>> = OnceCell::new();

fn parse_peripherals() -> Result<Peripherals, Box<dyn Error>> {
    let env = match std::env::var("PERIPHERALS") {
        Ok(env) => env,
        Err(std::env::VarError::NotPresent) => {
            return Ok(Peripherals {
                peripherals: vec![],
            })
        }
        Err(e) => return Err(Box::new(e)),
    };
    let peripherals = serde_json::from_str(&env)?;
    Ok(peripherals)
}

pub fn resolve_peripheral_name(name: &str) -> Option<String> {
    match PERIPHERAL_NAMES.get() {
        Some(map) => map.get(name).cloned(),
        None => None,
    }
}

pub(crate) fn initialize() -> Result<(), Box<dyn Error>> {
    let peripherals = parse_peripherals()?;
    let peripheral_mounts = peripherals
        .peripherals
        .into_iter()
        .map(|peripheral| (peripheral.name, peripheral.mount))
        .collect();
    PERIPHERAL_NAMES
        .set(peripheral_mounts)
        .map_err(|_| "Already initialized")?;
    Ok(())
}
