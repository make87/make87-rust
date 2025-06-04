#[derive(Debug, Clone)]
pub struct CameraPeripheral {
    pub reference: String,
    pub name: String,
    pub device_nodes: Vec<String>,
    pub volumes: Vec<(String, String)>,
    pub camera_type: Option<String>,
    pub protocol: Option<String>,
}

impl CameraPeripheral {
    pub fn from_model(m: &crate::models::CameraPeripheral) -> Self {
        Self {
            reference: m.reference.clone(),
            name: m.name.clone(),
            device_nodes: m.device_nodes.clone(),
            volumes: m.volumes.clone(),
            camera_type: m.camera_type.clone(),
            protocol: m.protocol.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CameraPeripheral as CameraPeripheralModel;

    #[test]
    fn test_from_model() {
        let model = CameraPeripheralModel {
            reference: "ref1".to_string(),
            peripheral_type: crate::models::PeripheralType::Camera,
            name: "cam1".to_string(),
            device_nodes: vec!["/dev/video0".to_string()],
            volumes: vec![("/host/path".to_string(), "/container/path".to_string())],
            camera_type: Some("usb".to_string()),
            protocol: Some("v4l2".to_string()),
        };
        let cam = CameraPeripheral::from_model(&model);
        assert_eq!(cam.reference, model.reference);
        assert_eq!(cam.name, model.name);
        assert_eq!(cam.device_nodes, model.device_nodes);
        assert_eq!(cam.volumes, model.volumes);
        assert_eq!(cam.camera_type, model.camera_type);
        assert_eq!(cam.protocol, model.protocol);
    }
}

