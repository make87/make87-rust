pub mod endpoints;
pub mod errors;
pub mod peripherals;
pub mod session;
pub mod topics;
pub mod application_config;
mod utils;

pub use application_config::get_config_value;
pub use endpoints::{get_provider, get_requester, resolve_endpoint_name};
pub use peripherals::resolve_peripheral_name;
pub use topics::{get_publisher, get_subscriber, resolve_topic_name};

pub fn initialize() {
    session::initialize().unwrap();
    topics::initialize().unwrap();
    endpoints::initialize().unwrap();
    peripherals::initialize().unwrap();
    application_config::initialize().unwrap();
}

pub fn keep_running() {
    loop {
        std::thread::park();
    }
}
