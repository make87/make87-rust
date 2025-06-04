#[derive(Debug, Clone)]
pub struct CodecPeripheral {
    pub name: String,
    pub supported_codecs: Vec<String>,
    pub device_nodes: Vec<String>,
}

impl CodecPeripheral {
    pub fn from_model(m: &crate::models::CodecPeripheral) -> Self {
        Self {
            name: m.name.clone(),
            supported_codecs: m.supported_codecs.clone(),
            device_nodes: m.device_nodes.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CodecPeripheral as CodecPeripheralModel;

    #[test]
    fn test_from_model() {
        let model = CodecPeripheralModel {
            peripheral_type: crate::models::PeripheralType::Codec,
            name: "codec1".to_string(),
            supported_codecs: vec!["H.264".to_string(), "H.265".to_string()],
            device_nodes: vec!["/dev/video10".to_string()],
        };
        let codec = CodecPeripheral::from_model(&model);
        assert_eq!(codec.name, model.name);
        assert_eq!(codec.supported_codecs, model.supported_codecs);
        assert_eq!(codec.device_nodes, model.device_nodes);
    }
}
