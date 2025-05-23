#[derive(Debug, Clone)]
pub struct GpioPeripheral {
    pub chip_name: String,
    pub label: String,
    pub num_lines: u32,
    pub device_nodes: Vec<String>,
    pub lines: Vec<GpioLineInfo>,
}

impl GpioPeripheral {
    pub fn from_model(m: &crate::models::GpioPeripheral) -> Self {
        Self {
            chip_name: m.chip_name.clone(),
            label: m.label.clone(),
            num_lines: m.num_lines,
            device_nodes: m.device_nodes.clone(),
            lines: m.lines.iter().map(|l| GpioLineInfo::from_model(l)).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GpioLineInfo {
    pub line_offset: u32,
    pub name: Option<String>,
    pub consumer: Option<String>,
    pub direction: String,
    pub active_state: String,
    pub used: bool,
}

impl GpioLineInfo {
    pub fn from_model(m: &crate::models::GpioLineInfo) -> Self {
        Self {
            line_offset: m.line_offset,
            name: m.name.clone(),
            consumer: m.consumer.clone(),
            direction: m.direction.clone(),
            active_state: m.active_state.clone(),
            used: m.used,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{GpioPeripheral as GpioPeripheralModel, GpioLineInfo as GpioLineInfoModel};

    #[test]
    fn test_from_model() {
        let line_model = GpioLineInfoModel {
            line_offset: 1,
            name: Some("line1".to_string()),
            consumer: Some("consumer1".to_string()),
            direction: "in".to_string(),
            active_state: "high".to_string(),
            used: true,
        };
        let model = GpioPeripheralModel {
            peripheral_type: crate::models::PeripheralType::GPIO,
            chip_name: "chip0".to_string(),
            label: "label0".to_string(),
            num_lines: 1,
            device_nodes: vec!["/dev/gpiochip0".to_string()],
            lines: vec![line_model.clone()],
        };
        let gpio = GpioPeripheral::from_model(&model);
        assert_eq!(gpio.chip_name, model.chip_name);
        assert_eq!(gpio.label, model.label);
        assert_eq!(gpio.num_lines, model.num_lines);
        assert_eq!(gpio.device_nodes, model.device_nodes);
        assert_eq!(gpio.lines.len(), 1);
        assert_eq!(gpio.lines[0].line_offset, line_model.line_offset);
        assert_eq!(gpio.lines[0].name, line_model.name);
        assert_eq!(gpio.lines[0].consumer, line_model.consumer);
        assert_eq!(gpio.lines[0].direction, line_model.direction);
        assert_eq!(gpio.lines[0].active_state, line_model.active_state);
        assert_eq!(gpio.lines[0].used, line_model.used);
    }
}
