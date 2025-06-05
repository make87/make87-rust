use std::collections::BTreeMap;
use std::error::Error;

use make87::encodings::{Encoder, ProtobufEncoder};
use make87::interfaces::zenoh::{ConfiguredSubscriber, ZenohInterface, ZenohSubscriberConfig};
use make87_messages::text::PlainText;
use make87::models::{AccessPoint, ApplicationConfig, ApplicationInfo, BoundSubscriber, InterfaceConfig, MountedPeripherals, PublisherTopicConfig, SubscriberTopicConfig};

macro_rules! recv_and_print {
    ($sub:expr) => {{
        let subscriber = $sub;
        let message_encoder = ProtobufEncoder::<PlainText>::new();
        while let Ok(sample) = subscriber.recv_async().await {
            let message_decoded = message_encoder.decode(&sample.payload().to_bytes());
            match message_decoded {
                Ok(msg) => println!("Received: {:?}", msg),
                Err(e) => eprintln!("Decode error: {e}"),
            }
        }
    }};
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let zenoh_interface = ZenohInterface::from_default_env("zenoh")?;
    let session = zenoh_interface.get_session().await?;

    let configured_subscriber = zenoh_interface.get_subscriber(&session,"HELLO_WORLD_MESSAGE").await?;

    // Use a macro to avoid duplicated code. User  to do it inline instead.
    match configured_subscriber {
        ConfiguredSubscriber::Fifo(sub) => recv_and_print!(&sub),
        ConfiguredSubscriber::Ring(sub) => recv_and_print!(&sub),
    }

    Ok(())
}
