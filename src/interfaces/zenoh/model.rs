use serde::{Deserialize, Serialize};
use zenoh::qos;

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Priority {
    RealTime,
    InteractiveHigh,
    InteractiveLow,
    DataHigh,
    #[default]
    Data,
    DataLow,
    Background,
}

impl Priority {
    pub fn to_zenoh(&self) -> qos::Priority {
        match self {
            Priority::RealTime => qos::Priority::RealTime,
            Priority::InteractiveHigh => qos::Priority::InteractiveHigh,
            Priority::InteractiveLow => qos::Priority::InteractiveLow,
            Priority::DataHigh => qos::Priority::DataHigh,
            Priority::Data => qos::Priority::Data,
            Priority::DataLow => qos::Priority::DataLow,
            Priority::Background => qos::Priority::Background,
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Reliability {
    BestEffort,
    #[default]
    Reliable,
}

impl Reliability {
    pub fn to_zenoh(&self) -> qos::Reliability {
        match self {
            Reliability::BestEffort => qos::Reliability::BestEffort,
            Reliability::Reliable => qos::Reliability::Reliable,
        }
    }
}


#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CongestionControl {
    #[default]
    Drop,
    Block,
}

impl CongestionControl {
    pub fn to_zenoh(&self) -> qos::CongestionControl {
        match self {
            CongestionControl::Drop => qos::CongestionControl::Drop,
            CongestionControl::Block => qos::CongestionControl::Block,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
#[serde(tag = "handler_type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HandlerChannel {
    Fifo { capacity: u32 },
    Ring { capacity: u32 },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ZenohSubscriberConfig {
    pub handler: HandlerChannel,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ZenohPublisherConfig {
    pub congestion_control: CongestionControl,
    pub priority: Priority,
    pub express: bool,
    pub reliability: Reliability,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ZenohQuerierConfig {
    pub congestion_control: CongestionControl,
    pub priority: Priority,
    pub express: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ZenohQueryableConfig {
    pub handler: HandlerChannel,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_priority_to_zenoh() {
        assert_eq!(Priority::RealTime.to_zenoh(), zenoh::qos::Priority::RealTime);
        assert_eq!(Priority::Data.to_zenoh(), zenoh::qos::Priority::Data);
    }

    #[test]
    fn test_reliability_to_zenoh() {
        assert_eq!(Reliability::BestEffort.to_zenoh(), zenoh::qos::Reliability::BestEffort);
        assert_eq!(Reliability::Reliable.to_zenoh(), zenoh::qos::Reliability::Reliable);
    }

    #[test]
    fn test_congestion_control_to_zenoh() {
        assert_eq!(CongestionControl::Drop.to_zenoh(), zenoh::qos::CongestionControl::Drop);
        assert_eq!(CongestionControl::Block.to_zenoh(), zenoh::qos::CongestionControl::Block);
    }

    #[test]
    fn test_handler_channel_serialization() {
        let fifo = HandlerChannel::Fifo { capacity: 10 };
        let ring = HandlerChannel::Ring { capacity: 5 };
        let fifo_json = serde_json::to_string(&fifo).unwrap();
        let ring_json = serde_json::to_string(&ring).unwrap();
        assert!(fifo_json.contains("FIFO"));
        assert!(ring_json.contains("RING"));
        let de_fifo: HandlerChannel = serde_json::from_str(&fifo_json).unwrap();
        let de_ring: HandlerChannel = serde_json::from_str(&ring_json).unwrap();
        assert_eq!(fifo, de_fifo);
        assert_eq!(ring, de_ring);
    }

    #[test]
    fn test_zenoh_subscriber_config_serialization() {
        let config = ZenohSubscriberConfig {
            handler: HandlerChannel::Fifo { capacity: 3 },
        };
        let json = serde_json::to_string(&config).unwrap();
        let de: ZenohSubscriberConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, de);
    }

    #[test]
    fn test_zenoh_publisher_config_serialization() {
        let config = ZenohPublisherConfig {
            congestion_control: CongestionControl::Block,
            priority: Priority::InteractiveHigh,
            express: true,
            reliability: Reliability::BestEffort,
        };
        let json = serde_json::to_string(&config).unwrap();
        let de: ZenohPublisherConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, de);
    }

    #[test]
    fn test_zenoh_querier_config_serialization() {
        let config = ZenohQuerierConfig {
            congestion_control: CongestionControl::Drop,
            priority: Priority::Data,
            express: false,
        };
        let json = serde_json::to_string(&config).unwrap();
        let de: ZenohQuerierConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, de);
    }

    #[test]
    fn test_zenoh_queryable_config_serialization() {
        let config = ZenohQueryableConfig {
            handler: HandlerChannel::Ring { capacity: 7 },
        };
        let json = serde_json::to_string(&config).unwrap();
        let de: ZenohQueryableConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, de);
    }
}
