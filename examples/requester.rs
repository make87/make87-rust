use std::collections::BTreeMap;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

use make87::encodings::{Encoder, ProtobufEncoder};
use make87::interfaces::zenoh::ZenohInterface;
use make87_messages::core::Header;
use make87_messages::google::protobuf::Timestamp;
use make87_messages::text::PlainText;
use make87::models::{AccessPoint, ApplicationConfig, ApplicationInfo, BoundRequester, BoundSubscriber, InterfaceConfig, MountedPeripherals, PublisherTopicConfig, RequesterEndpointConfig, SubscriberTopicConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let message_encoder = ProtobufEncoder::<PlainText>::new();
    let zenoh_interface = ZenohInterface::from_default_env("zenoh")?;
    let session = zenoh_interface.get_session().await?;

    let requester = zenoh_interface.get_requester(&session, "HELLO_WORLD_MESSAGE").await?;
    let mut header = Header {
        entity_path: "/pytest/pub_sub".to_string(),
        reference_id: 0,
        ..Default::default()
    };

    loop {
        header.timestamp = Timestamp::get_current_time().into();

        let message = PlainText {
            header: Some(header.clone()),
            body: "Hello, World! 🦀".to_string(),
            ..Default::default()
        };
        let message_encoded = message_encoder.encode(&message)?;
        let replies = requester.get().payload(&message_encoded).await?;

        while let Ok(reply) = replies.recv_async().await {
            match reply.result() {
                Ok(sample) => {
                    let message_decoded = message_encoder.decode(&sample.payload().to_bytes());
                    match message_decoded {
                        Ok(msg) => println!("Received response: {:?}", msg),
                        Err(e) => eprintln!("Decode error: {e}"),
                    }
                }
                Err(err) => {
                    let payload = err
                        .payload()
                        .try_to_string()
                        .unwrap_or_else(|e| e.to_string().into());
                    println!("Received error: {}", payload);
                }
            }
        }
        sleep(Duration::from_millis(100)).await;
    }
}
