use std::process::{Command, Child};
use std::thread::sleep;
use std::time::Duration;
use std::env;
use std::path::PathBuf;
use uuid::Uuid;

// You must import or define your ApplicationConfig etc here!
use make87::models::{ApplicationConfig, URLMapping, MountedPeripherals, TopicConfig, /*...*/};

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
    let topic_name = "HELLO_WORLD_MESSAGE";
    let topic_key = "my_topic_key";

    let mut topics = vec![];
    topics.push(TopicConfig::Pub {
        topic_name: topic_name.into(),
        topic_key: topic_key.into(),
        message_type: "make87_messages.text.text_plain.PlainText".into(),
        interface_name: "zenoh".into(),
        config: std::collections::BTreeMap::new(),
        protocol: "zenoh".into(),
        encoding: Some("proto".into()),
    });

    let config = ApplicationConfig {
        topics: topics.clone(),
        endpoints: vec![],
        services: vec![],
        url_mapping: URLMapping { name_to_url: Default::default() },
        peripherals: MountedPeripherals { peripherals: vec![] },
        config: serde_json::Value::Null,
        entrypoint_name: None,
        deployed_application_id: "4408ba07-6963-4243-9572-fe7fa679784c".into(),
        system_id: "b0e65164-f54d-4350-8e39-ea257b46cde3".into(),
        deployed_application_name: "pub_app_1".into(),
        is_release_version: true,
        public_ip: None,
        vpn_ip: "10.10.0.1".into(),
        port_config: vec![],
        git_url: None,
        git_branch: None,
        application_id: "20f6f4d4-229b-4b22-987b-e22f61713dc4".into(),
        application_name: "pub_app".into(),
        storage_url: None,
        storage_endpoint_url: None,
        storage_access_key: None,
        storage_secret_key: None,
    };
    let config_json = serde_json::to_string(&config).unwrap();

    // 2. Find your publisher and subscriber binaries
    // When run via `cargo test`, your binaries will be at target/debug/<bin-name>
    let publisher_bin = assert_cmd::cargo::cargo_bin("publisher"); // from assert_cmd crate
    let subscriber_bin = assert_cmd::cargo::cargo_bin("subscriber"); // from assert_cmd crate

    // 3. Start subscriber first
    let mut sub_proc = spawn_with_env(subscriber_bin.to_str().unwrap(), &config_json);
    sleep(Duration::from_secs(1)); // Give subscriber time to start

    // 4. Start publisher
    let mut pub_proc = spawn_with_env(publisher_bin.to_str().unwrap(), &config_json);
    sleep(Duration::from_secs(1)); // Allow pub/sub to communicate

    // 5. Terminate both (clean shutdown, or let them exit naturally)
    pub_proc.kill().ok();
    let _ = pub_proc.wait();

    sub_proc.kill().ok();
    let output = sub_proc.wait_with_output().expect("Failed to wait on subscriber");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // 6. Check output
    assert!(stdout.to_lowercase().contains("hello"));
    assert!(stdout.to_lowercase().contains("world"));
}

