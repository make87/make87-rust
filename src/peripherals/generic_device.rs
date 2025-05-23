#[derive(Debug, Clone)]
pub struct GenericDevicePeripheral {
    pub name: String,
    pub device_node: String,
}


impl GenericDevicePeripheral {
    pub fn from_model(m: &crate::models::GenericDevicePeripheral) -> Self {
        Self {
            name: m.name.clone(),
            device_node: m.device_node.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::GenericDevicePeripheral as GenericDevicePeripheralModel;

    #[test]
    fn test_from_model() {
        let model = GenericDevicePeripheralModel {
            peripheral_type: crate::models::PeripheralType::GenericDevice,
            name: "dev1".to_string(),
            device_node: "/dev/some_device".to_string(),
        };
        let dev = GenericDevicePeripheral::from_model(&model);
        assert_eq!(dev.name, model.name);
        assert_eq!(dev.device_node, model.device_node);
    }
}
