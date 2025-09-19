// use std::collections::BTreeMap;
// use std::process::{Child, Command};
// use std::thread::sleep;
// use std::time::Duration;
//
// // You must import or define your ApplicationConfig etc here!
// use make87::models::{
//     AccessPoint, ApplicationConfig, ApplicationInfo, BoundClient, ClientServiceConfig,
//     InterfaceConfig, MountedPeripherals, ServerServiceConfig,
// };
//
// /// Helper to start a subprocess with given env and binary path.
// fn spawn_with_env(bin_path: &str, config_json: &str) -> Child {
//     Command::new(bin_path)
//         .env("MAKE87_CONFIG", config_json)
//         .stdout(std::process::Stdio::piped())
//         .stderr(std::process::Stdio::piped())
//         .spawn()
//         .expect("Failed to spawn process")
// }
//
// /// Test client/server integration for rerun interface.
// #[test]
// fn test_client_server_defaults_only() {
//     // 1. Build config objects (minimal example)
//     let interface_name = "rerun";
//     let service_name = "RERUN_SERVICE";
//     let service_key = "my_service_key";
//
//     let server_config_partial = ServerServiceConfig {
//         name: service_name.into(),
//         key: service_key.into(),
//         spec: "rerun_service_spec".into(),
//         interface_name: interface_name.into(),
//         protocol: "grpc".into(),
//         config: BTreeMap::from([
//             ("max_bytes".to_string(), serde_json::json!(1073741824u64)), // 1GB
//         ]),
//     };
//
//     let client_config_partial = BoundClient {
//         access_point: AccessPoint {
//             vpn_ip: "127.0.0.1".to_string(),
//             vpn_port: 9876,
//             public_ip: None,
//             public_port: None,
//             same_node: true,
//         },
//         config: ClientServiceConfig {
//             name: service_name.into(),
//             key: service_key.into(),
//             spec: "rerun_service_spec".into(),
//             interface_name: interface_name.into(),
//             protocol: "grpc".into(),
//             config: BTreeMap::from([
//                 ("batcher_config".to_string(), serde_json::json!({
//                     "flush_tick": 0.2,
//                     "flush_num_bytes": 1048576u64,
//                     "flush_num_rows": 18446744073709551615u64
//                 })),
//             ]),
//         },
//     };
//
//     let config = ApplicationConfig {
//         interfaces: BTreeMap::from([(
//             interface_name.into(),
//             InterfaceConfig {
//                 name: interface_name.into(),
//                 publishers: Default::default(),
//                 subscribers: Default::default(),
//                 requesters: Default::default(),
//                 providers: Default::default(),
//                 clients: Default::default(),
//                 servers: Default::default(),
//             },
//         )]),
//         peripherals: MountedPeripherals {
//             peripherals: vec![],
//         },
//         config: serde_json::Value::Null,
//         storage: Default::default(),
//         application_info: ApplicationInfo {
//             deployed_application_id: "4408ba07-6963-4243-9572-fe7fa679784c".into(),
//             system_id: "b0e65164-f54d-4350-8e39-ea257b46cde3".into(),
//             git_url: None,
//             git_branch: None,
//             application_id: "20f6f4d4-229b-4b22-987b-e22f61713dc4".into(),
//             application_name: "rerun_app".into(),
//             deployed_application_name: "rerun_app_1".into(),
//             is_release_version: true,
//         },
//     };
//
//     let mut server_config = config.clone();
//     server_config
//         .interfaces
//         .get_mut(interface_name)
//         .unwrap()
//         .servers
//         .insert(service_name.into(), server_config_partial);
//
//     let mut client_config = config.clone();
//     client_config
//         .interfaces
//         .get_mut(interface_name)
//         .unwrap()
//         .clients
//         .insert(service_name.into(), client_config_partial);
//
//     let server_config_json = serde_json::to_string(&server_config).unwrap();
//     let client_config_json = serde_json::to_string(&client_config).unwrap();
//
//     // 2. Find your client and server binaries
//     // When run via `cargo test`, your binaries will be at target/debug/examples/<bin-name>
//     let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
//     let debug_examples = format!("{}/debug/examples", target_dir);
//
//     // For rerun, we might need to create example binaries or use existing ones
//     // For now, let's assume we have rerun_server and rerun_client examples
//     let server_bin = std::path::Path::new(&debug_examples).join("rerun_server");
//     let client_bin = std::path::Path::new(&debug_examples).join("rerun_client");
//
//     // Check if the binaries exist - if not, skip this test
//     if !server_bin.exists() || !client_bin.exists() {
//         println!("Skipping rerun integration test - example binaries not found");
//         println!("Expected binaries:");
//         println!("  Server: {:?}", server_bin);
//         println!("  Client: {:?}", client_bin);
//         return;
//     }
//
//     // 3. Start server first
//     let mut server_proc = spawn_with_env(server_bin.to_str().unwrap(), &server_config_json);
//     sleep(Duration::from_secs(2)); // Give server time to start and bind to port
//
//     // 4. Start client
//     let mut client_proc = spawn_with_env(client_bin.to_str().unwrap(), &client_config_json);
//     sleep(Duration::from_secs(2)); // Allow client/server to communicate
//
//     // 5. Terminate both (clean shutdown, or let them exit naturally)
//     client_proc.kill().ok();
//     let client_output = client_proc.wait_with_output().expect("Failed to wait on client");
//     let client_stdout = String::from_utf8_lossy(&client_output.stdout);
//     let client_stderr = String::from_utf8_lossy(&client_output.stderr);
//
//     server_proc.kill().ok();
//     let server_output = server_proc
//         .wait_with_output()
//         .expect("Failed to wait on server");
//     let server_stdout = String::from_utf8_lossy(&server_output.stdout);
//     let server_stderr = String::from_utf8_lossy(&server_output.stderr);
//
//     println!("Client stdout:\n{}", client_stdout);
//     println!("Client stderr:\n{}", client_stderr);
//     println!("Server stdout:\n{}", server_stdout);
//     println!("Server stderr:\n{}", server_stderr);
//
//     // 6. Check output - for rerun, we expect successful connection and data logging
//     // These assertions may need to be adjusted based on actual rerun behavior
//     assert!(
//         server_stdout.to_lowercase().contains("listening")
//         || server_stdout.to_lowercase().contains("serving")
//         || server_stdout.to_lowercase().contains("started")
//         || server_stderr.to_lowercase().contains("listening")
//         || server_stderr.to_lowercase().contains("serving")
//         || server_stderr.to_lowercase().contains("started"),
//         "Server should indicate it's listening/serving/started"
//     );
//
//     assert!(
//         client_stdout.to_lowercase().contains("connected")
//         || client_stdout.to_lowercase().contains("recording")
//         || client_stderr.to_lowercase().contains("connected")
//         || client_stderr.to_lowercase().contains("recording"),
//         "Client should indicate successful connection or recording"
//     );
// }
//
// /// Test minimal rerun client/server configuration.
// #[test]
// fn test_client_server_minimal_config() {
//     // 1. Build minimal config objects
//     let interface_name = "rerun";
//     let service_name = "MINIMAL_RERUN_SERVICE";
//     let service_key = "minimal_service_key";
//
//     let server_config_partial = ServerServiceConfig {
//         name: service_name.into(),
//         key: service_key.into(),
//         spec: "minimal_rerun_service_spec".into(),
//         interface_name: interface_name.into(),
//         protocol: "grpc".into(),
//         config: BTreeMap::new(), // No max_bytes specified - should use default
//     };
//
//     let client_config_partial = BoundClient {
//         access_point: AccessPoint {
//             vpn_ip: "127.0.0.1".to_string(),
//             vpn_port: 9876,
//             public_ip: None,
//             public_port: None,
//             same_node: true,
//         },
//         config: ClientServiceConfig {
//             name: service_name.into(),
//             key: service_key.into(),
//             spec: "minimal_rerun_service_spec".into(),
//             interface_name: interface_name.into(),
//             protocol: "grpc".into(),
//             config: BTreeMap::new(), // No batcher config specified - should use defaults
//         },
//     };
//
//     let config = ApplicationConfig {
//         interfaces: BTreeMap::from([(
//             interface_name.into(),
//             InterfaceConfig {
//                 name: interface_name.into(),
//                 publishers: Default::default(),
//                 subscribers: Default::default(),
//                 requesters: Default::default(),
//                 providers: Default::default(),
//                 clients: Default::default(),
//                 servers: Default::default(),
//             },
//         )]),
//         peripherals: MountedPeripherals {
//             peripherals: vec![],
//         },
//         config: serde_json::Value::Null,
//         storage: Default::default(),
//         application_info: ApplicationInfo {
//             deployed_application_id: "4408ba07-6963-4243-9572-fe7fa679784c".into(),
//             system_id: "b0e65164-f54d-4350-8e39-ea257b46cde3".into(),
//             git_url: None,
//             git_branch: None,
//             application_id: "20f6f4d4-229b-4b22-987b-e22f61713dc4".into(),
//             application_name: "minimal_rerun_app".into(),
//             deployed_application_name: "minimal_rerun_app_1".into(),
//             is_release_version: true,
//         },
//     };
//
//     let mut server_config = config.clone();
//     server_config
//         .interfaces
//         .get_mut(interface_name)
//         .unwrap()
//         .servers
//         .insert(service_name.into(), server_config_partial);
//
//     let mut client_config = config.clone();
//     client_config
//         .interfaces
//         .get_mut(interface_name)
//         .unwrap()
//         .clients
//         .insert(service_name.into(), client_config_partial);
//
//     // Test that the configs serialize successfully
//     let server_config_json = serde_json::to_string(&server_config).unwrap();
//     let client_config_json = serde_json::to_string(&client_config).unwrap();
//
//     // Verify the JSON contains expected structure
//     assert!(server_config_json.contains("rerun"));
//     assert!(server_config_json.contains("MINIMAL_RERUN_SERVICE"));
//     assert!(client_config_json.contains("rerun"));
//     assert!(client_config_json.contains("MINIMAL_RERUN_SERVICE"));
//
//     println!("Minimal config test passed - configurations serialize correctly");
// }