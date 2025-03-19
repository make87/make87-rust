use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::error::Error;

static APPLICATION_CONFIG: OnceCell<HashMap<String, String>> = OnceCell::new();

pub(crate) fn initialize() -> Result<(), Box<dyn Error + Send + Sync>> {
    let env = std::env::var("APPLICATION_CONFIG").unwrap_or("{}".to_string());
    let env_config: HashMap<String, String> = serde_json::from_str(&env).unwrap_or(HashMap::new());
    APPLICATION_CONFIG
        .set(env_config)
        .map_err(|_| "Application config is already initialized")?;

    Ok(())
}

pub fn get_config_value(key: &str) -> Option<String> {
    APPLICATION_CONFIG.get().and_then(|config| config.get(key).cloned())
}
