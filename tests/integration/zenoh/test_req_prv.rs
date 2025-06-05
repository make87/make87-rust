use std::collections::BTreeMap;
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::Duration;

// You must import or define your ApplicationConfig etc here!
use make87::models::{
    AccessPoint, ApplicationConfig, ApplicationInfo, BoundRequester, InterfaceConfig,
    MountedPeripherals, ProviderEndpointConfig, RequesterEndpointConfig,
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

/// Test requester/provider integration.
#[test]
fn test_req_prv_defaults_only() {
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
                "handler_type": "RING",
                "capacity": 10
            }),
        )]),
    };

    let requester_config_partial = BoundRequester {
        access_point: AccessPoint {
            vpn_ip: "127.0.0.1".to_string(),
            vpn_port: 7447,
            public_ip: None,
            public_port: None,
            same_node: true,
        },
        config: RequesterEndpointConfig {
            endpoint_name: endpoint_name.into(),
            endpoint_key: endpoint_key.into(),
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

    let mut provider_config = config.clone();
    provider_config
        .interfaces
        .get_mut(interface_name)
        .unwrap()
        .providers
        .insert(endpoint_name.into(), provider_config_partial);

    let mut requester_config = config.clone();
    requester_config
        .interfaces
        .get_mut(interface_name)
        .unwrap()
        .requesters
        .insert(endpoint_name.into(), requester_config_partial);

    let provider_config_json = serde_json::to_string(&provider_config).unwrap();
    let requester_config_json = serde_json::to_string(&requester_config).unwrap();

    // 2. Find your requester and provider binaries
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let debug_examples = format!("{}/debug/examples", target_dir);
    let provider_bin = std::path::Path::new(&debug_examples).join("provider");
    let requester_bin = std::path::Path::new(&debug_examples).join("requester");

    // 3. Start provider first
    let mut prv_proc = spawn_with_env(provider_bin.to_str().unwrap(), &provider_config_json);
    sleep(Duration::from_secs(1)); // Give provider time to start

    // 4. Start requester
    let mut req_proc = spawn_with_env(requester_bin.to_str().unwrap(), &requester_config_json);
    sleep(Duration::from_secs(1)); // Allow req/prv to communicate

    // 5. Terminate both (clean shutdown, or let them exit naturally)
    req_proc.kill().ok();
    let req_output = req_proc
        .wait_with_output()
        .expect("Failed to wait on requester");
    let req_stdout = String::from_utf8_lossy(&req_output.stdout);
    let req_stderr = String::from_utf8_lossy(&req_output.stderr);

    prv_proc.kill().ok();
    let prv_output = prv_proc.wait_with_output().expect("Failed to wait on provider");
    let prv_stdout = String::from_utf8_lossy(&prv_output.stdout);
    let prv_stderr = String::from_utf8_lossy(&prv_output.stderr);

    println!("Requester stdout:\n{}", req_stdout);
    println!("Requester stderr:\n{}", req_stderr);
    println!("Provider stdout:\n{}", prv_stdout);
    println!("Provider stderr:\n{}", prv_stderr);

    // 6. Check output
    assert!(req_stdout.to_lowercase().contains("olleh"));
    assert!(req_stdout.to_lowercase().contains("dlrow"));
}