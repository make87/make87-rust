#[derive(Debug, Clone)]
pub struct RealSenseCameraPeripheral {
    pub name: String,
    pub device_nodes: Vec<String>,
    pub serial_number: String,
    pub model: String,
}

impl RealSenseCameraPeripheral {
    pub fn from_model(m: &crate::models::RealSenseCameraPeripheral) -> Self {
        Self {
            name: m.name.clone(),
            device_nodes: m.device_nodes.clone(),
            serial_number: m.serial_number.clone(),
            model: m.model.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RealSenseCameraPeripheral as RealSenseCameraPeripheralModel;

    #[test]
    fn test_from_model() {
        let model = RealSenseCameraPeripheralModel {
            peripheral_type: crate::models::PeripheralType::RealSense,
            name: "rs1".to_string(),
            device_nodes: vec!["/dev/rs0".to_string()],
            serial_number: "123456789".to_string(),
            model: "D435".to_string(),
        };
        let rs = RealSenseCameraPeripheral::from_model(&model);
        assert_eq!(rs.name, model.name);
        assert_eq!(rs.device_nodes, model.device_nodes);
        assert_eq!(rs.serial_number, model.serial_number);
        assert_eq!(rs.model, model.model);
    }
}
