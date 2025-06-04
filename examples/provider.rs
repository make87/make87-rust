use make87::encodings::{Encoder, ProtobufEncoder};
use make87::interfaces::zenoh::{ConfiguredProvider, ZenohInterface};
use make87::models::{
    ApplicationConfig, ApplicationInfo, InterfaceConfig,
    MountedPeripherals, ProviderEndpointConfig,
};
use make87_messages::core::Header;
use make87_messages::google::protobuf::Timestamp;
use make87_messages::text::PlainText;
use std::collections::BTreeMap;
use std::error::Error;

macro_rules! recv_and_reply {
    ($prv:expr) => {{
        let provider = $prv;
        let message_encoder = ProtobufEncoder::<PlainText>::new();
        while let Ok(query) = provider.recv_async().await {
            let payload = query.payload().ok_or("No payload to decode")?;
            let message_decoded = message_encoder.decode(&payload.to_bytes());
            match message_decoded {
                Ok(msg) => {
                    println!("Received: {:?}", msg);
                    let reply = PlainText {
                        header: Header {
                            timestamp: Timestamp::get_current_time().into(),
                            entity_path: msg.header.as_ref().unwrap().entity_path.clone(),
                            reference_id: msg.header.as_ref().unwrap().reference_id,
                        }
                        .into(),
                        body: msg.body.chars().rev().collect(),
                        ..Default::default()
                    };
                    let reply_encoded = message_encoder.encode(&reply)?;
                    query
                        .reply(&query.key_expr().clone(), &reply_encoded)
                        .await?;
                }
                Err(e) => eprintln!("Decode error: {e}"),
            }
        }
        Ok::<(), Box<dyn Error + Send + Sync>>(())
    }};
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // 1. Build config objects (minimal example)
    let interface_name = "zenoh";
    let endpoint_name = "HELLO_WORLD_MESSAGE";
    let endpoint_key = "my_topic_key";

    let provider_config_partial = ProviderEndpointConfig {
        endpoint_name: endpoint_name.into(),
        endpoint_key: endpoint_key.into(),
        provider_message_type: "make87_messages.text.text_plain.PlainText".into(),
        requester_message_type: "make87_messages.text.text_plain.PlainText".into(),
        interface_name: interface_name.into(),
        protocol: "zenoh".into(),
        encoding: Some("proto".into()),
        config: BTreeMap::from([(
            "handler".to_string(),
            serde_json::json!({
                "handler_type": "FIFO",
                "capacity": 100
            }),
        )]),
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

    let mut provider_config = config.clone();
    provider_config
        .interfaces
        .get_mut(interface_name)
        .unwrap()
        .providers
        .insert(endpoint_name.into(), provider_config_partial);

    let zenoh_interface = ZenohInterface::new(provider_config, "zenoh");
    let session = zenoh_interface.get_session().await?;

    let configured_provider = zenoh_interface
        .get_provider(&session, "HELLO_WORLD_MESSAGE")
        .await?;

    match configured_provider {
        ConfiguredProvider::Fifo(prv) => recv_and_reply!(prv)?,
        ConfiguredProvider::Ring(prv) => recv_and_reply!(prv)?,
    }

    Ok(())
}
