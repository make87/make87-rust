use std::collections::BTreeMap;
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::Duration;

// You must import or define your ApplicationConfig etc here!
use make87::models::{
    AccessPoint, ApplicationConfig, ApplicationInfo, BoundSubscriber, InterfaceConfig,
    MountedPeripherals, PublisherTopicConfig, SubscriberTopicConfig,
};

/// Helper to start a subprocess with given env and binary path.
fn spawn_with_env(bin_path: &str, config_json: &str) -> Child {
    Command::new(bin_path)
        .env("MAKE87_CONFIG", config_json)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn process")
}

/// Test publisher/subscriber integration.
#[test]
fn test_pub_sub_defaults_only() {
    // 1. Build config objects (minimal example)
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
                }),
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

    let mut publisher_config = config.clone();
    publisher_config
        .interfaces
        .get_mut(interface_name)
        .unwrap()
        .publishers
        .insert(topic_name.into(), publisher_config_partial);

    let mut subscriber_config = config.clone();
    subscriber_config
        .interfaces
        .get_mut(interface_name)
        .unwrap()
        .subscribers
        .insert(topic_name.into(), subscriber_config_partial);

    let subscriber_config_json = serde_json::to_string(&subscriber_config).unwrap();
    let publisher_config_json = serde_json::to_string(&publisher_config).unwrap();

    // 2. Find your publisher and subscriber binaries
    // When run via `cargo test`, your binaries will be at target/debug/examples/<bin-name>
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let debug_examples = format!("{}/debug/examples", target_dir);
    let publisher_bin = std::path::Path::new(&debug_examples).join("publisher");
    let subscriber_bin = std::path::Path::new(&debug_examples).join("subscriber");

    // 3. Start subscriber first
    let mut sub_proc = spawn_with_env(subscriber_bin.to_str().unwrap(), &subscriber_config_json);
    sleep(Duration::from_secs(1)); // Give subscriber time to start

    // 4. Start publisher
    let mut pub_proc = spawn_with_env(publisher_bin.to_str().unwrap(), &publisher_config_json);
    sleep(Duration::from_secs(1)); // Allow pub/sub to communicate

    // 5. Terminate both (clean shutdown, or let them exit naturally)
    pub_proc.kill().ok();
    let _ = pub_proc.wait();

    sub_proc.kill().ok();
    let output = sub_proc
        .wait_with_output()
        .expect("Failed to wait on subscriber");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // 6. Check output
    assert!(stdout.to_lowercase().contains("olleh"));
    assert!(stdout.to_lowercase().contains("dlrow"));
}
