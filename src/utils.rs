use serde::{Deserialize};
use zenoh::qos;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Priority {
    RealTime,
    InteractiveHigh,
    InteractiveLow,
    DataHigh,
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

#[derive(Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Reliability {
    BestEffort,
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

#[derive(Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum CongestionControl {
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

#[derive(Deserialize, Clone)]
#[serde(tag = "handler_type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum HandlerChannel {
    Fifo { capacity: Option<usize> },
    Ring { capacity: Option<usize> },
}