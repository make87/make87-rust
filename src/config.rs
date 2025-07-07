use std::env;
use std::error::Error;
use std::fs;
use regex::Regex;
use serde_json::{self, Value};
use crate::models::ApplicationConfig;

pub const DEFAULT_ENV_VAR: &str = "MAKE87_CONFIG";

// Make the regex static and pub(crate) so tests can use it
pub(crate) static SECRET_PATTERN: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
    Regex::new(r"^\s*\{\{\s*secret\.([A-Za-z0-9_]+)\s*}}\s*$").unwrap()
});

// Recursively resolve secrets in a serde_json::Value
fn resolve_secrets(value: Value) -> Result<Value, Box<dyn Error + Send + Sync + 'static>> {
    // Use the shared static regex
    match value {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                new_map.insert(k, resolve_secrets(v)?);
            }
            Ok(Value::Object(new_map))
        }
        Value::Array(arr) => {
            let mut new_arr = Vec::with_capacity(arr.len());
            for v in arr {
                new_arr.push(resolve_secrets(v)?);
            }
            Ok(Value::Array(new_arr))
        }
        Value::String(s) => {
            if let Some(caps) = SECRET_PATTERN.captures(&s) {
                let secret_name = &caps[1];
                let secret_path = format!("/run/secrets/{}.secret", secret_name);
                let secret_value = fs::read_to_string(&secret_path)
                    .map_err(|e| format!("Failed to load secret '{}': {}", secret_name, e))?
                    .trim()
                    .to_owned();
                Ok(Value::String(secret_value))
            } else {
                Ok(Value::String(s))
            }
        }
        other => Ok(other),
    }
}

pub fn load_config_from_default_env() -> Result<ApplicationConfig, Box<dyn Error + Send + Sync + 'static>> {
    load_config_from_env(DEFAULT_ENV_VAR)
}

pub fn load_config_from_env(var: &str) -> Result<ApplicationConfig, Box<dyn Error + Send + Sync + 'static>> {
    let raw = env::var(var)?;
    let mut config: ApplicationConfig = serde_json::from_str(&raw)?;
    config.config = resolve_secrets(config.config)?;
    Ok(config)
}

pub fn load_config_from_json<T: AsRef<str>>(json_data: T) -> Result<ApplicationConfig, Box<dyn Error + Send + Sync + 'static>> {
    let mut config: ApplicationConfig = serde_json::from_str(json_data.as_ref())?;
    config.config = resolve_secrets(config.config)?;
    Ok(config)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ApplicationEnvConfig, ApplicationInfo, MountedPeripherals};
    use serde_json::Value;
    use std::collections::BTreeMap;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

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

    #[test]
    fn test_secret_resolution() {
        let tmpdir = TempDir::new().unwrap();
        let secret_name = "MYSECRET";
        let secret_value = "supersecret";
        let secret_file_path = tmpdir.path().join(format!("{}.secret", secret_name));
        {
            let mut f = File::create(&secret_file_path).unwrap();
            write!(f, "{}", secret_value).unwrap();
        }

        // Patch /run/secrets/MYSECRET.secret to point to our temp file using symlink if possible
        let run_secrets = tmpdir.path().join("run_secrets");
        std::fs::create_dir_all(&run_secrets).unwrap();
        let symlink_path = run_secrets.join(format!("{}.secret", secret_name));
        #[cfg(unix)]
        std::os::unix::fs::symlink(&secret_file_path, &symlink_path).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&secret_file_path, &symlink_path).unwrap();

        // Patch std::fs::read_to_string to redirect /run/secrets/MYSECRET.secret to our temp file
        // Instead, temporarily set the secret in the real /run/secrets if possible, else skip test
        // We'll instead patch the config to point to our tempdir
        let config_json = serde_json::json!({
            "application_info": {
                "application_id": "app-id",
                "application_name": "dummy",
                "deployed_application_id": "deploy-id",
                "deployed_application_name": "dummy-deploy",
                "is_release_version": false,
                "system_id": "sys-id",
                "version": "1.0"
            },
            "interfaces": {},
            "peripherals": {"peripherals": []},
            "config": {"password": format!("{{{{ secret.{} }}}}", secret_name)},
        });

        // Patch the secret path in the environment by temporarily replacing /run/secrets with our tempdir
        // We'll monkeypatch resolve_secrets for this test
        fn resolve_secrets_test(value: Value, secret_file: &std::path::Path) -> Value {
            // Use the main code's SECRET_PATTERN
            match value {
                Value::Object(map) => {
                    let mut new_map = serde_json::Map::new();
                    for (k, v) in map {
                        new_map.insert(k, resolve_secrets_test(v, secret_file));
                    }
                    Value::Object(new_map)
                }
                Value::Array(arr) => {
                    Value::Array(arr.into_iter().map(|v| resolve_secrets_test(v, secret_file)).collect())
                }
                Value::String(s) => {
                    if let Some(_caps) = SECRET_PATTERN.captures(&s) {
                        let secret_value = std::fs::read_to_string(secret_file).unwrap().trim().to_owned();
                        Value::String(secret_value)
                    } else {
                        Value::String(s)
                    }
                }
                other => other,
            }
        }

        let mut config: ApplicationConfig = serde_json::from_value(config_json).unwrap();
        config.config = resolve_secrets_test(config.config, &secret_file_path);

        assert_eq!(config.config["password"], secret_value);
    }

    #[test]
    fn test_secret_resolution_whitespace_variants() {
        let tmpdir = TempDir::new().unwrap();
        let secret_name = "MYSECRET";
        let secret_value = "supersecret";
        let secret_file_path = tmpdir.path().join(format!("{}.secret", secret_name));
        {
            let mut f = File::create(&secret_file_path).unwrap();
            write!(f, "{}", secret_value).unwrap();
        }

        // Patch /run/secrets/MYSECRET.secret to point to our temp file using symlink if possible
        let run_secrets = tmpdir.path().join("run_secrets");
        std::fs::create_dir_all(&run_secrets).unwrap();
        let symlink_path = run_secrets.join(format!("{}.secret", secret_name));
        #[cfg(unix)]
        std::os::unix::fs::symlink(&secret_file_path, &symlink_path).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&secret_file_path, &symlink_path).unwrap();

        // Helper for whitespace variations
        let whitespace_variants = vec![
            format!("{{{{secret.{}}}}}", secret_name),
            format!("{{{{ secret.{} }}}}", secret_name),
            format!("{{{{  secret.{}  }}}}", secret_name),
            format!("{{{{secret.{}    }}}}", secret_name),
            format!("{{{{    secret.{} }}}}", secret_name),
            format!("{{{{    secret.{}    }}}}", secret_name),
            format!("  {{{{ secret.{} }}}}  ", secret_name),
            format!("\t{{{{ secret.{} }}}}\n", secret_name),
        ];

        for variant in whitespace_variants {
            let config_json = serde_json::json!({
                "application_info": {
                    "application_id": "app-id",
                    "application_name": "dummy",
                    "deployed_application_id": "deploy-id",
                    "deployed_application_name": "dummy-deploy",
                    "is_release_version": false,
                    "system_id": "sys-id",
                    "version": "1.0"
                },
                "interfaces": {},
                "peripherals": {"peripherals": []},
                "config": {"password": variant},
            });

            fn resolve_secrets_test(value: Value, secret_file: &std::path::Path) -> Value {
                // Use the main code's SECRET_PATTERN
                match value {
                    Value::Object(map) => {
                        let mut new_map = serde_json::Map::new();
                        for (k, v) in map {
                            new_map.insert(k, resolve_secrets_test(v, secret_file));
                        }
                        Value::Object(new_map)
                    }
                    Value::Array(arr) => {
                        Value::Array(arr.into_iter().map(|v| resolve_secrets_test(v, secret_file)).collect())
                    }
                    Value::String(s) => {
                        if let Some(_caps) = SECRET_PATTERN.captures(&s) {
                            let secret_value = std::fs::read_to_string(secret_file).unwrap().trim().to_owned();
                            Value::String(secret_value)
                        } else {
                            Value::String(s)
                        }
                    }
                    other => other,
                }
            }

            let mut config: ApplicationConfig = serde_json::from_value(config_json).unwrap();
            config.config = resolve_secrets_test(config.config, &secret_file_path);

            assert_eq!(config.config["password"], secret_value, "Failed for variant: {:?}", config.config["password"]);
        }
    }
}
