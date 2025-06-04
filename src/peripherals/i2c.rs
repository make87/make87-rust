#[derive(Debug, Clone)]
pub struct I2cPeripheral {
    pub bus_number: u32,
    pub name: String,
    pub device_nodes: Vec<String>,
    pub detected_devices: Vec<I2cDetectedDevice>,
}

impl I2cPeripheral {
    pub fn from_model(m: &crate::models::I2cPeripheral) -> Self {
        Self {
            bus_number: m.bus_number,
            name: m.name.clone(),
            device_nodes: m.device_nodes.clone(),
            detected_devices: m.detected_devices.iter().map(|d| I2cDetectedDevice::from_model(d)).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct I2cDetectedDevice {
    pub address: String,
    pub description: Option<String>,
}

impl I2cDetectedDevice {
    pub fn from_model(m: &crate::models::I2cDetectedDevice) -> Self {
        Self {
            address: m.address.clone(),
            description: m.description.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{I2cPeripheral as I2cPeripheralModel, I2cDetectedDevice as I2cDetectedDeviceModel};

    #[test]
    fn test_from_model() {
        let detected_model = I2cDetectedDeviceModel {
            address: "0x40".to_string(),
            description: Some("sensor".to_string()),
        };
        let model = I2cPeripheralModel {
            peripheral_type: crate::models::PeripheralType::I2C,
            bus_number: 1,
            name: "i2c1".to_string(),
            device_nodes: vec!["/dev/i2c-1".to_string()],
            detected_devices: vec![detected_model.clone()],
        };
        let i2c = I2cPeripheral::from_model(&model);
        assert_eq!(i2c.bus_number, model.bus_number);
        assert_eq!(i2c.name, model.name);
        assert_eq!(i2c.device_nodes, model.device_nodes);
        assert_eq!(i2c.detected_devices.len(), 1);
        assert_eq!(i2c.detected_devices[0].address, detected_model.address);
        assert_eq!(i2c.detected_devices[0].description, detected_model.description);
    }
}
