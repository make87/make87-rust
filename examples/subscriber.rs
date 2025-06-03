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
    // 1. Build config objects (minimal example)
    let interface_name = "zenoh";
    let topic_name = "HELLO_WORLD_MESSAGE";
    let topic_key = "my_topic_key";

    let subscriber_config_partial = BoundSubscriber {
        access_point: AccessPoint {
            vpn_ip: "127.0.0.1".to_string(),
            vpn_port: 7447,
            public_ip: None,
            public_port: None,
            same_node: true,
        },
        config: SubscriberTopicConfig {
            topic_name: topic_name.into(),
            topic_key: topic_key.into(),
            message_type: "make87_messages.text.text_plain.PlainText".into(),
            interface_name: "zenoh".into(),
            protocol: "zenoh".into(),
            encoding: Some("proto".into()),
            config: BTreeMap::from([(
                "handler".to_string(),
                serde_json::json!({
                    "handler_type": "FIFO",
                    "capacity": 100
                })
            )]),
        },

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

    let mut subscriber_config = config.clone();
    subscriber_config.interfaces.get_mut(interface_name).unwrap().subscribers.insert(
        topic_name.into(),
        subscriber_config_partial,
    );



    let zenoh_interface = ZenohInterface::new(subscriber_config,"zenoh");
    let session = zenoh_interface.get_session().await?;

    let configured_subscriber = zenoh_interface.get_subscriber(&session,"HELLO_WORLD_MESSAGE").await?;

    // Use a macro to avoid duplicated code. User  to do it inline instead.
    match configured_subscriber {
        ConfiguredSubscriber::Fifo(sub) => recv_and_print!(&sub),
        ConfiguredSubscriber::Ring(sub) => recv_and_print!(&sub),
    }

    Ok(())
}
