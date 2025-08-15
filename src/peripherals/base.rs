use crate::config::load_config_from_default_env;
use crate::models::{ApplicationConfig, Peripheral as PeripheralModel};
use crate::peripherals::{
    CameraPeripheral, CodecPeripheral, GenericDevicePeripheral, GpioPeripheral, GpuPeripheral, I2cPeripheral,
    IspPeripheral, OtherPeripheral, RealSenseCameraPeripheral, RenderingPeripheral
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Peripheral {
    Camera(CameraPeripheral),
    Gpu(GpuPeripheral),
    Gpio(GpioPeripheral),
    I2c(I2cPeripheral),
    Isp(IspPeripheral),
    Codec(CodecPeripheral),
    Rendering(RenderingPeripheral),
    RealSense(RealSenseCameraPeripheral),
    GenericDevice(GenericDevicePeripheral),
    Other(OtherPeripheral),
}


pub struct PeripheralManager {
    peripherals: HashMap<String, Peripheral>,
    config: ApplicationConfig,
}

impl PeripheralManager {
    pub fn new(config: ApplicationConfig) -> Self {
        let mut peripherals = HashMap::new();
        for mp in &config.peripherals.peripherals {
            let name = mp.name.clone();
            let peripheral = create_peripheral_from_model(&mp.peripheral);
            peripherals.insert(name, peripheral);
        }
        Self { peripherals, config }
    }

    pub fn from_default_env() -> Result<Self, crate::config::ConfigError> {
        let config = load_config_from_default_env()?;
        Ok(Self::new(config))
    }

    pub fn get_peripheral_by_name(&self, name: &str) -> Option<&Peripheral> {
        self.peripherals.get(name)
    }

    pub fn list_peripherals(&self) -> Vec<&Peripheral> {
        self.peripherals.values().collect()
    }

    pub fn iter(&self) -> impl Iterator<Item=(&String, &Peripheral)> {
        self.peripherals.iter()
    }

    pub fn len(&self) -> usize {
        self.peripherals.len()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.peripherals.contains_key(name)
    }
}

fn create_peripheral_from_model(mp: &PeripheralModel) -> Peripheral {
    match mp {
        PeripheralModel::Camera(c) => Peripheral::Camera(CameraPeripheral::from_model(c)),
        PeripheralModel::GPU(g) => Peripheral::Gpu(GpuPeripheral::from_model(g)),
        PeripheralModel::GPIO(gpio) => Peripheral::Gpio(GpioPeripheral::from_model(gpio)),
        PeripheralModel::I2C(i2c) => Peripheral::I2c(I2cPeripheral::from_model(i2c)),
        PeripheralModel::ISP(isp) => Peripheral::Isp(IspPeripheral::from_model(isp)),
        PeripheralModel::Codec(codec) => Peripheral::Codec(CodecPeripheral::from_model(codec)),
        PeripheralModel::Rendering(rendering) => Peripheral::Rendering(RenderingPeripheral::from_model(rendering)),
        PeripheralModel::RealSense(rs) => Peripheral::RealSense(RealSenseCameraPeripheral::from_model(rs)),
        PeripheralModel::GenericDevice(gd) => Peripheral::GenericDevice(GenericDevicePeripheral::from_model(gd)),
        PeripheralModel::Speaker(other)
        | PeripheralModel::Keyboard(other)
        | PeripheralModel::Mouse(other)
        | PeripheralModel::Other(other) => Peripheral::Other(OtherPeripheral::from_model(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ApplicationEnvConfig, ApplicationInfo, MountedPeripherals, MountedPeripheral, PeripheralType, CameraPeripheral};
    use crate::models::Peripheral as PeripheralModel;
    use std::collections::BTreeMap;
    use serde_json::Value;

    fn make_test_config() -> ApplicationEnvConfig {
        ApplicationEnvConfig {
            interfaces: BTreeMap::new(),
            peripherals: MountedPeripherals {
                peripherals: vec![
                    MountedPeripheral {
                        name: "cam1".to_string(),
                        peripheral: PeripheralModel::Camera(CameraPeripheral {
                            reference: "ref1".to_string(),
                            peripheral_type: PeripheralType::Camera,
                            name: "cam1".to_string(),
                            device_nodes: vec!["/dev/video0".to_string()],
                            volumes: vec![("/host/path".to_string(), "/container/path".to_string())],
                            camera_type: Some("usb".to_string()),
                            protocol: Some("v4l2".to_string()),
                        })
                    }
                ]
            },
            config: Value::default(),
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
    fn test_new_and_get_peripheral_by_name() {
        let config = make_test_config();
        let manager = PeripheralManager::new(config.clone());
        assert_eq!(manager.len(), 1);
        assert!(manager.contains("cam1"));
        let p = manager.get_peripheral_by_name("cam1");
        assert!(matches!(p, Some(Peripheral::Camera(_))));
    }

    #[test]
    fn test_list_peripherals_and_iter() {
        let config = make_test_config();
        let manager = PeripheralManager::new(config);
        let peripherals = manager.list_peripherals();
        assert_eq!(peripherals.len(), 1);
        let mut iter_count = 0;
        for (name, p) in manager.iter() {
            assert_eq!(name, "cam1");
            assert!(matches!(p, Peripheral::Camera(_)));
            iter_count += 1;
        }
        assert_eq!(iter_count, 1);
    }

    #[test]
    fn test_from_default_env_error() {
        // Unset the env var to ensure error
        unsafe { std::env::remove_var(crate::config::DEFAULT_ENV_VAR); }
        let result = PeripheralManager::from_default_env();
        assert!(result.is_err());
    }
}

