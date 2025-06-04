#[derive(Debug, Clone)]
pub struct IspPeripheral {
    pub name: String,
    pub supported_features: Vec<String>,
    pub device_nodes: Vec<String>,
}

impl IspPeripheral {
    pub fn from_model(m: &crate::models::IspPeripheral) -> Self {
        Self {
            name: m.name.clone(),
            supported_features: m.supported_features.clone(),
            device_nodes: m.device_nodes.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::IspPeripheral as IspPeripheralModel;

    #[test]
    fn test_from_model() {
        let model = IspPeripheralModel {
            peripheral_type: crate::models::PeripheralType::ISP,
            name: "isp1".to_string(),
            supported_features: vec!["denoise".to_string(), "scale".to_string()],
            device_nodes: vec!["/dev/video13".to_string()],
        };
        let isp = IspPeripheral::from_model(&model);
        assert_eq!(isp.name, model.name);
        assert_eq!(isp.supported_features, model.supported_features);
        assert_eq!(isp.device_nodes, model.device_nodes);
    }
}
