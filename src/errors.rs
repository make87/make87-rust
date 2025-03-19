use thiserror::Error;

#[derive(Error, Debug)]
pub enum TopicManagerError {
    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] std::env::VarError),

    #[error("Zenoh error: {0}")]
    ZenohError(#[from] zenoh::Error),

    #[error("(De)Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Unknown topic type: {0}")]
    UnknownTopicType(String),
}

#[derive(Error, Debug)]
pub enum EndpointManagerError {
    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] std::env::VarError),

    #[error("Zenoh error: {0}")]
    ZenohError(#[from] zenoh::Error),

    #[error("(De)Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Unknown topic type: {0}")]
    UnknownEndpointType(String),

    #[error("Endpoint not available: {0}")]
    EndpointNotAvailable(String),
}
