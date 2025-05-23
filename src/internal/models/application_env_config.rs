use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use serde_json::Value;

fn default_interface_name() -> String {
    "zenoh".to_string()
}

fn default_protocol() -> String {
    "zenoh".to_string()
}


fn default_encoding() -> Option<String> {
    Some("proto".to_string())
}


fn default_publish_mode() -> PublishMode {
    PublishMode::Ingress
}

fn default_port_protocol() -> ProtocolEnum {
    ProtocolEnum::TCP
}

fn default_is_system_interface() -> bool {
    false
}

pub fn deserialize_u16_from_i32<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let value: i32 = Deserialize::deserialize(deserializer)?;
    u16::try_from(value).map_err(serde::de::Error::custom)
}

pub fn serialize_u16_as_i32<S>(value: &u16, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i32(*value as i32)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MappedURL {
    pub vpn_ip: String,
    pub vpn_port: u16,
    pub public_ip: Option<String>,
    pub public_port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct URLMapping {
    pub name_to_url: HashMap<String, MappedURL>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApplicationEnvConfig {
    pub topics: Vec<TopicConfig>,
    pub endpoints: Vec<EndpointConfig>,
    pub services: Vec<ServiceConfig>,
    pub url_mapping: URLMapping,
    pub peripherals: MountedPeripherals,
    pub config: Value,
    pub entrypoint_name: Option<String>,
    pub deployed_application_id: String,
    pub system_id: String,
    pub deployed_application_name: String,
    pub is_release_version: bool,
    pub public_ip: Option<String>,
    pub vpn_ip: String,
    pub port_config: Vec<PortConfig>,
    pub git_url: Option<String>,
    pub git_branch: Option<String>,
    pub application_id: String,
    pub application_name: String,
    pub storage_url: Option<String>,
    pub storage_endpoint_url: Option<String>,
    pub storage_access_key: Option<String>,
    pub storage_secret_key: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
#[serde(tag = "topic_type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TopicConfig {
    Pub {
        topic_name: String,
        topic_key: String,
        message_type: String,
        #[serde(default = "default_interface_name")]
        interface_name: String,
        #[serde(flatten)]
        config: BTreeMap<String, Value>,
        #[serde(default = "default_protocol")]
        protocol: String,
        #[serde(default = "default_encoding")]
        encoding: Option<String>,
    },
    Sub {
        topic_name: String,
        topic_key: String,
        message_type: String,
        #[serde(default = "default_interface_name")]
        interface_name: String,

        #[serde(flatten)]
        config: BTreeMap<String, Value>,
        #[serde(default = "default_protocol")]
        protocol: String,
        #[serde(default = "default_encoding")]
        encoding: Option<String>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
#[serde(tag = "endpoint_type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EndpointConfig {
    Req {
        endpoint_name: String,
        endpoint_key: String,
        requester_message_type: String,
        provider_message_type: String,
        #[serde(default = "default_interface_name")]
        interface_name: String,

        #[serde(flatten)]
        config: BTreeMap<String, Value>,

        #[serde(default = "default_protocol")]
        protocol: String,
        #[serde(default = "default_encoding")]
        encoding: Option<String>,
    },
    Prv {
        endpoint_name: String,
        endpoint_key: String,
        requester_message_type: String,
        provider_message_type: String,
        #[serde(default = "default_interface_name")]
        interface_name: String,

        #[serde(flatten)]
        config: BTreeMap<String, Value>,

        #[serde(default = "default_protocol")]
        protocol: String,
        #[serde(default = "default_encoding")]
        encoding: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "service_type")]
pub enum ServiceConfig {
    /// The service is a client
    Client {
        name: String,
        spec: String,
        key: String,
        #[serde(default = "default_interface_name")]
        interface_name: String,

        #[serde(flatten)]
        config: BTreeMap<String, Value>,
        #[serde(default = "default_protocol")]
        protocol: String,
    },
    /// The service is a server
    Server {
        name: String,
        key: String,
        spec: String,
        #[serde(default = "default_interface_name")]
        interface_name: String,

        #[serde(flatten)]
        config: BTreeMap<String, Value>,
        #[serde(default = "default_protocol")]
        protocol: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountedPeripherals {
    pub peripherals: Vec<MountedPeripheral>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct MountedPeripheral {
    /// The name of the peripheral. That is used by the application version to identify the peripheral
    pub name: String,
    pub peripheral: Peripheral,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GpuPeripheral {
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub model: String,
    pub index: Option<u32>,
    pub device_nodes: Vec<String>,
    pub vram: Option<u32>, // VRAM in MB
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct I2cPeripheral {
    pub peripheral_type: PeripheralType,
    pub bus_number: u32,
    pub name: String,
    pub device_nodes: Vec<String>,
    pub detected_devices: Vec<I2cDetectedDevice>,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GpioPeripheral {
    pub peripheral_type: PeripheralType,
    pub chip_name: String,
    pub label: String,
    pub num_lines: u32,
    pub device_nodes: Vec<String>,
    pub lines: Vec<GpioLineInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CameraPeripheral {
    pub reference: String,
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub device_nodes: Vec<String>,
    pub volumes: Vec<(String, String)>, // Additional volumes to mount
    #[serde(default)]
    pub camera_type: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RealSenseCameraPeripheral {
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub device_nodes: Vec<String>, // All device nodes to mount
    // pub primary_device_nodes: Vec<String>, // Primary device nodes (could be multiple)
    pub serial_number: String, // Serial number of the camera
    pub model: String,         // Model name
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct IspPeripheral {
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub supported_features: Vec<String>, // Features like "denoise", "scale", etc.
    pub device_nodes: Vec<String>,       // Device nodes like /dev/video13
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CodecPeripheral {
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub supported_codecs: Vec<String>, // Codec types like "H.264", "H.265", etc.
    pub device_nodes: Vec<String>,     // Device nodes like /dev/video10
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RenderingPeripheral {
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub supported_apis: Vec<String>,  // APIs like "OpenGL", "Vulkan"
    pub max_performance: Option<u32>, // Optional performance metric (e.g., FLOPS)
    pub device_nodes: Vec<String>,    // Relevant device nodes
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OtherPeripheral {
    pub reference: String,
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub device_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GenericDevicePeripheral {
    pub peripheral_type: PeripheralType,
    pub name: String,
    pub device_node: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct I2cDetectedDevice {
    pub address: String,
    pub description: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GpioLineInfo {
    pub line_offset: u32,
    pub name: Option<String>,
    pub consumer: Option<String>,
    pub direction: String,
    pub active_state: String,
    pub used: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct PortConfig {
    pub name: String,
    #[serde(default = "default_port_protocol")]
    pub protocol: ProtocolEnum,

    /// The port inside the container.
    #[serde(
        deserialize_with = "deserialize_u16_from_i32",
        serialize_with = "serialize_u16_as_i32"
    )]
    pub target_port: u16,

    /// The port on the swarm hosts.
    #[serde(
        deserialize_with = "deserialize_u16_from_i32",
        serialize_with = "serialize_u16_as_i32"
    )]
    pub published_port: u16,
    #[serde(default = "default_publish_mode")]
    pub publish_mode: PublishMode,
    #[serde(default = "default_is_system_interface")]
    pub is_system_interface: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum ProtocolEnum {
    TCP,
    UDP,
    SCTP,
    HTTP,
    WS,
    SSH,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum PublishMode {
    Ingress,
    Host,
}

impl Default for PublishMode {
    fn default() -> Self {
        PublishMode::Ingress
    }
}