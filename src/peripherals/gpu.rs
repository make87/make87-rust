#[derive(Debug, Clone)]
pub struct GpuPeripheral {
    pub name: String,
    pub model: String,
    pub index: Option<u32>,
    pub device_nodes: Vec<String>,
    pub vram: Option<u32>,
}

impl GpuPeripheral {
    pub fn from_model(m: &crate::models::GpuPeripheral) -> Self {
        Self {
            name: m.name.clone(),
            model: m.model.clone(),
            index: m.index,
            device_nodes: m.device_nodes.clone(),
            vram: m.vram,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::GpuPeripheral as GpuPeripheralModel;

    #[test]
    fn test_from_model() {
        let model = GpuPeripheralModel {
            peripheral_type: crate::models::PeripheralType::GPU,
            name: "gpu1".to_string(),
            model: "RTX 3090".to_string(),
            index: Some(0),
            device_nodes: vec!["/dev/nvidia0".to_string()],
            vram: Some(24576),
        };
        let gpu = GpuPeripheral::from_model(&model);
        assert_eq!(gpu.name, model.name);
        assert_eq!(gpu.model, model.model);
        assert_eq!(gpu.index, model.index);
        assert_eq!(gpu.device_nodes, model.device_nodes);
        assert_eq!(gpu.vram, model.vram);
    }
}
