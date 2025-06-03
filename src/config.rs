use std::env;
use std::error::Error;
use serde_json;
use crate::models::ApplicationConfig;

pub const DEFAULT_ENV_VAR: &str = "MAKE87_CONFIG";


pub fn load_config_from_default_env() -> Result<ApplicationConfig, Box<dyn Error + Send + Sync + 'static>> {
    load_config_from_env(DEFAULT_ENV_VAR)
}


pub fn load_config_from_env(var: &str) -> Result<ApplicationConfig, Box<dyn Error + Send + Sync + 'static>> {
    let raw = env::var(var)?;
    let config: ApplicationConfig = serde_json::from_str(&raw)?;
    Ok(config)
}

pub fn load_config_from_json<T: AsRef<str>>(json_data: T) -> Result<ApplicationConfig, Box<dyn Error + Send + Sync + 'static>> {
    let config: ApplicationConfig = serde_json::from_str(json_data.as_ref())?;
    Ok(config)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ApplicationEnvConfig, ApplicationInfo, MountedPeripherals};
    use serde_json::Value;
    use std::collections::BTreeMap;
    use std::env;

    fn default_app_config() -> ApplicationEnvConfig {
        ApplicationEnvConfig {
            interfaces: BTreeMap::new(),
            peripherals: MountedPeripherals { peripherals: vec![] },
            config: Value::Null,
            storage: None,
            application_info: ApplicationInfo {
                deployed_application_id: "id1".into(),
                deployed_application_name: "app".into(),
                system_id: "sysid".into(),
                application_id: "appid".into(),
                application_name: "myapp".into(),
                git_url: None,
                git_branch: None,
                is_release_version: true,
            },
        }
    }

    #[test]
    fn test_load_config_from_json_ok() {
        let config = default_app_config();
        let json = serde_json::to_string(&config).unwrap();
        let result = load_config_from_json(&json);
        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.application_info.deployed_application_id, config.application_info.deployed_application_id);
    }

    #[test]
    fn test_load_config_from_json_error() {
        let bad_json = "{ not valid json ";
        let result = load_config_from_json(bad_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_from_env_ok() {
        let config = default_app_config();
        let json = serde_json::to_string(&config).unwrap();
        let var = "MY_TEST_CONFIG_ENV";
        unsafe { env::set_var(var, &json); }
        let result = load_config_from_env(var);
        assert!(result.is_ok());
        let loaded = result.unwrap();
        assert_eq!(loaded.application_info.system_id, config.application_info.system_id);
        unsafe { env::remove_var(var); }
    }

    #[test]
    fn test_load_config_from_env_missing_var() {
        let var = "MY_TEST_CONFIG_ENV_MISSING";
        unsafe { env::remove_var(var); } // Make sure it doesn't exist
        let result = load_config_from_env(var);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_from_default_env_ok() {
        let config = default_app_config();
        let json = serde_json::to_string(&config).unwrap();
        unsafe { env::set_var(DEFAULT_ENV_VAR, &json); }
        let result = load_config_from_default_env();
        assert!(result.is_ok());
        unsafe { env::remove_var(DEFAULT_ENV_VAR); }
    }

    #[test]
    fn test_load_config_from_default_env_missing() {
        unsafe { env::remove_var(DEFAULT_ENV_VAR); }
        let result = load_config_from_default_env();
        assert!(result.is_err());
    }
}

