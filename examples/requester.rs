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
    let interface_name = "zenoh";
    let topic_name = "HELLO_WORLD_MESSAGE";
    let topic_key = "my_topic_key";

    let requester_config_partial = BoundRequester {
        access_point: AccessPoint {
            vpn_ip: "127.0.0.1".to_string(),
            vpn_port: 7447,
            public_ip: None,
            public_port: None,
            same_node: true,
        },
        config: RequesterEndpointConfig {
            endpoint_name: topic_name.into(),
            endpoint_key: topic_key.into(),
            requester_message_type: "make87_messages.text.text_plain.PlainText".into(),
            provider_message_type: "make87_messages.text.text_plain.PlainText".into(),
            interface_name: interface_name.into(),
            protocol: "zenoh".into(),
            encoding: Some("proto".into()),
            config: BTreeMap::from([
                ("congestion_control".to_string(), serde_json::json!("DROP")),
                ("priority".to_string(), serde_json::json!("DATA")),
                ("express".to_string(), serde_json::json!(true)),
            ]),
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

    let mut requester_config = config.clone();
    requester_config
        .interfaces
        .get_mut(interface_name)
        .unwrap()
        .requesters
        .insert(topic_name.into(), requester_config_partial);


    let message_encoder = ProtobufEncoder::<PlainText>::new();
    let zenoh_interface = ZenohInterface::new(requester_config, "zenoh");
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
