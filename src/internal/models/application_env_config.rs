use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const CURRENT_CONFIG_VERSION: &str = "1.0.0";

#[derive(Serialize, Deserialize, Clone)]
pub struct AccessPoint {
    pub vpn_ip: String,
    pub vpn_port: u16,
    pub public_ip: Option<String>,
    pub public_port: Option<u16>,
    pub same_node: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BoundSubscriber {
    #[serde(flatten)]
    pub access_point: AccessPoint,
    #[serde(flatten)]
    pub config: SubscriberTopicConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BoundRequester {
    #[serde(flatten)]
    pub access_point: AccessPoint,
    #[serde(flatten)]
    pub config: RequesterEndpointConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BoundClient {
    #[serde(flatten)]
    pub access_point: AccessPoint,
    #[serde(flatten)]
    pub config: ClientServiceConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InterfaceConfig {
    pub name: String,
    pub publishers: BTreeMap<String, PublisherTopicConfig>,
    pub subscribers: BTreeMap<String, BoundSubscriber>,
    pub requesters: BTreeMap<String, BoundRequester>,
    pub providers: BTreeMap<String, ProviderEndpointConfig>,
    pub clients: BTreeMap<String, BoundClient>,
    pub servers: BTreeMap<String, ServerServiceConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    pub url: String,
    pub endpoint_url: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ApplicationInfo {
    pub deployed_application_id: String,
    pub deployed_application_name: String,
    pub system_id: String,
    pub application_id: String,
    pub application_name: String,
    pub git_url: Option<String>,
    pub git_branch: Option<String>,
    pub is_release_version: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ApplicationEnvConfig {
    pub interfaces: BTreeMap<String, InterfaceConfig>,
    pub peripherals: MountedPeripherals,
    pub config: Value,
    pub storage: Option<StorageConfig>,
    pub application_info: ApplicationInfo,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum PeripheralType {
    Camera,
    RealSense,
    Speaker,
    Keyboard,
    Mouse,
    Microphone,
    GPU,
    I2C,
    GPIO,
    ISP,
    Codec,
    Rendering,
    GenericDevice,
    Other(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GpuPeripheral {
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub model: String,
    pub index: Option<u32>,
    pub device_nodes: Vec<String>,
    pub vram: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GenericDevicePeripheral {
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub device_node: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GenericDeviceConstraints {
    #[serde(default)]
    pub path_prefix: Option<String>,
    #[serde(default)]
    pub path_suffix: Option<String>,
    #[serde(default)]
    pub contains: Option<Vec<String>>,
    #[serde(default)]
    pub contains_not: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct I2cDetectedDevice {
    pub address: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct I2cPeripheral {
    pub peripheral_type: PeripheralType,
    pub bus_number: u32,
    pub name: String,
    pub device_nodes: Vec<String>,
    pub detected_devices: Vec<I2cDetectedDevice>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GpioLineInfo {
    pub line_offset: u32,
    pub name: Option<String>,
    pub consumer: Option<String>,
    pub direction: String,
    pub active_state: String,
    pub used: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GpioPeripheral {
    pub peripheral_type: PeripheralType,
    pub chip_name: String,
    pub label: String,
    pub num_lines: u32,
    pub device_nodes: Vec<String>,
    pub lines: Vec<GpioLineInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CameraPeripheral {
    pub reference: String,
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub device_nodes: Vec<String>,
    pub volumes: Vec<(String, String)>,
    #[serde(default)]
    pub camera_type: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CameraConstraints {
    #[serde(default)]
    pub camera_types: Option<Vec<String>>,
    #[serde(default)]
    pub protocols: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RealSenseCameraPeripheral {
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub device_nodes: Vec<String>,
    pub serial_number: String,
    pub model: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IspPeripheral {
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub supported_features: Vec<String>,
    pub device_nodes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CodecPeripheral {
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub supported_codecs: Vec<String>,
    pub device_nodes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RenderingPeripheral {
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub supported_apis: Vec<String>,
    pub max_performance: Option<u32>,
    pub device_nodes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OtherPeripheral {
    pub reference: String,
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub device_nodes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Peripheral {
    GPU(GpuPeripheral),
    I2C(I2cPeripheral),
    GPIO(GpioPeripheral),
    Camera(CameraPeripheral),
    RealSense(RealSenseCameraPeripheral),
    ISP(IspPeripheral),
    Codec(CodecPeripheral),
    Rendering(RenderingPeripheral),
    Speaker(OtherPeripheral),
    Keyboard(OtherPeripheral),
    Mouse(OtherPeripheral),
    GenericDevice(GenericDevicePeripheral),
    Other(OtherPeripheral),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GpuConstraints {
    #[serde(default)]
    pub min_vram: Option<u32>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub min_driver_version: Option<String>,
    #[serde(default)]
    pub min_cuda_version: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum PeripheralConstraints {
    GPU(GpuConstraints),
    Camera(CameraConstraints),
    GenericDevice(GenericDeviceConstraints),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PeripheralRequirement {
    pub peripheral_type: PeripheralType,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraints: Option<PeripheralConstraints>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MountedPeripheral {
    pub name: String,
    pub peripheral: Peripheral,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MountedPeripherals {
    pub peripherals: Vec<MountedPeripheral>,
}

fn default_interface_name() -> String {
    "zenoh".to_string()
}

fn default_protocol() -> String {
    "zenoh".to_string()
}

fn default_encoding() -> Option<String> {
    Some("proto".to_string())
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PublisherTopicConfig {
    pub topic_name: String,
    pub topic_key: String,
    pub message_type: String,
    #[serde(default = "default_interface_name")]
    pub interface_name: String,
    #[serde(flatten)]
    pub config: BTreeMap<String, serde_json::Value>,
    #[serde(default = "default_protocol")]
    pub protocol: String,
    #[serde(default = "default_encoding")]
    pub encoding: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SubscriberTopicConfig {
    pub topic_name: String,
    pub topic_key: String,
    pub message_type: String,
    #[serde(default = "default_interface_name")]
    pub interface_name: String,
    #[serde(flatten)]
    pub config: BTreeMap<String, serde_json::Value>,
    #[serde(default = "default_protocol")]
    pub protocol: String,
    #[serde(default = "default_encoding")]
    pub encoding: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RequesterEndpointConfig {
    pub endpoint_name: String,
    pub endpoint_key: String,
    pub requester_message_type: String,
    pub provider_message_type: String,
    #[serde(default = "default_interface_name")]
    pub interface_name: String,
    #[serde(flatten)]
    pub config: BTreeMap<String, serde_json::Value>,
    #[serde(default = "default_protocol")]
    pub protocol: String,
    #[serde(default = "default_encoding")]
    pub encoding: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProviderEndpointConfig {
    pub endpoint_name: String,
    pub endpoint_key: String,
    pub requester_message_type: String,
    pub provider_message_type: String,
    #[serde(default = "default_interface_name")]
    pub interface_name: String,
    #[serde(flatten)]
    pub config: BTreeMap<String, serde_json::Value>,
    #[serde(default = "default_protocol")]
    pub protocol: String,
    #[serde(default = "default_encoding")]
    pub encoding: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum RestartPolicy {
    Always,
    OnFailure,
    Never,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientServiceConfig {
    pub name: String,
    pub spec: String,
    pub key: String,
    #[serde(default = "default_interface_name")]
    pub interface_name: String,
    #[serde(flatten)]
    pub config: BTreeMap<String, serde_json::Value>,
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerServiceConfig {
    pub name: String,
    pub key: String,
    pub spec: String,
    #[serde(default = "default_interface_name")]
    pub interface_name: String,
    #[serde(flatten)]
    pub config: BTreeMap<String, serde_json::Value>,
    #[serde(default = "default_protocol")]
    pub protocol: String,
}
