use std::collections::BTreeMap;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

use make87::encodings::{Encoder, ProtobufEncoder};
use make87::interfaces::zenoh::ZenohInterface;
use make87_messages::core::Header;
use make87_messages::google::protobuf::Timestamp;
use make87_messages::text::PlainText;
use make87::models::{AccessPoint, ApplicationConfig, ApplicationInfo, BoundSubscriber, InterfaceConfig, MountedPeripherals, PublisherTopicConfig, SubscriberTopicConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let interface_name = "zenoh_test";
    let topic_name = "HELLO_WORLD_MESSAGE";
    let topic_key = "my_topic_key";

    let publisher_config_partial = PublisherTopicConfig {
        topic_name: topic_name.into(),
        topic_key: topic_key.into(),
        message_type: "make87_messages.text.text_plain.PlainText".into(),
        interface_name: "zenoh".into(),
        config: BTreeMap::from([
            ("congestion_control".to_string(), serde_json::json!("Drop")),
            ("priority".to_string(), serde_json::json!("Data")),
            ("express".to_string(), serde_json::json!(true)),
            ("reliability".to_string(), serde_json::json!("Reliable")),
        ]),
        protocol: "zenoh".into(),
        encoding: Some("proto".into()),
    };

    let config = ApplicationConfig {
        interfaces: BTreeMap::from([(
            interface_name.into(),
            InterfaceConfig {
                name: interface_name.into(),
                publishers: Default::default(),
                subscribers: Default::default(),
                requesters: Default::default(),
                providers: Default::default(),
                clients: Default::default(),
                servers: Default::default(),
            },
        )]),
        peripherals: MountedPeripherals {
            peripherals: vec![],
        },
        config: serde_json::Value::Null,
        storage: Default::default(),
        application_info: ApplicationInfo {
            deployed_application_id: "4408ba07-6963-4243-9572-fe7fa679784c".into(),
            system_id: "b0e65164-f54d-4350-8e39-ea257b46cde3".into(),
            git_url: None,
            git_branch: None,
            application_id: "20f6f4d4-229b-4b22-987b-e22f61713dc4".into(),
            application_name: "pub_app".into(),
            deployed_application_name: "pub_app_1".into(),
            is_release_version: true,
        },
    };

    let mut publisher_config = config.clone();
    publisher_config
        .interfaces
        .get_mut(interface_name)
        .unwrap()
        .publishers
        .insert(topic_name.into(), publisher_config_partial);


    let message_encoder = ProtobufEncoder::<PlainText>::new();
    let zenoh_interface = ZenohInterface::new(publisher_config, "zenoh");
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
