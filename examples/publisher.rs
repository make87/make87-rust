use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

use make87::encodings::{Encoder, ProtobufEncoder};
use make87::interfaces::zenoh::ZenohInterface;
use make87_messages::core::Header;
use make87_messages::google::protobuf::Timestamp;
use make87_messages::text::PlainText;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let message_encoder = ProtobufEncoder::<PlainText>::new();
    let zenoh_interface = ZenohInterface::from_default_env("zenoh")?;
    let session = zenoh_interface.get_session().await?;

    let publisher = zenoh_interface.get_publisher(&session,"HELLO_WORLD_MESSAGE").await?;
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
        publisher.put(&message_encoded).await?;

        println!("Published: {:?}", message);
        sleep(Duration::from_millis(100)).await;
    }
}
