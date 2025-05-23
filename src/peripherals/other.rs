#[derive(Debug, Clone)]
pub struct OtherPeripheral {
    pub reference: String,
    pub name: String,
    pub device_nodes: Vec<String>,
}

impl OtherPeripheral {
    pub fn from_model(m: &crate::models::OtherPeripheral) -> Self {
        Self {
            reference: m.reference.clone(),
            name: m.name.clone(),
            device_nodes: m.device_nodes.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::OtherPeripheral as OtherPeripheralModel;

    #[test]
    fn test_from_model() {
        let model = OtherPeripheralModel {
            reference: "ref2".to_string(),
            peripheral_type: crate::models::PeripheralType::Other("test".to_string()),
            name: "other1".to_string(),
            device_nodes: vec!["/dev/other0".to_string()],
        };
        let other = OtherPeripheral::from_model(&model);
        assert_eq!(other.reference, model.reference);
        assert_eq!(other.name, model.name);
        assert_eq!(other.device_nodes, model.device_nodes);
    }
}
