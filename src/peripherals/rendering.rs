#[derive(Debug, Clone)]
pub struct RenderingPeripheral {
    pub name: String,
    pub supported_apis: Vec<String>,
    pub max_performance: Option<u32>,
    pub device_nodes: Vec<String>,
}


impl RenderingPeripheral {
    pub fn from_model(m: &crate::models::RenderingPeripheral) -> Self {
        Self {
            name: m.name.clone(),
            supported_apis: m.supported_apis.clone(),
            max_performance: m.max_performance,
            device_nodes: m.device_nodes.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RenderingPeripheral as RenderingPeripheralModel;

    #[test]
    fn test_from_model() {
        let model = RenderingPeripheralModel {
            peripheral_type: crate::models::PeripheralType::Rendering,
            name: "render1".to_string(),
            supported_apis: vec!["OpenGL".to_string(), "Vulkan".to_string()],
            max_performance: Some(1000),
            device_nodes: vec!["/dev/renderD128".to_string()],
        };
        let rendering = RenderingPeripheral::from_model(&model);
        assert_eq!(rendering.name, model.name);
        assert_eq!(rendering.supported_apis, model.supported_apis);
        assert_eq!(rendering.max_performance, model.max_performance);
        assert_eq!(rendering.device_nodes, model.device_nodes);
    }
}
