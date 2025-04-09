use crate::topics::{get_publisher, resolve_topic_name};
use gag::BufferRedirect;
use make87_messages::text::{log_message, LogMessage};
use std::io::{BufRead, BufReader};
use std::{process, thread, time};
use make87_messages::core::Header;
use make87_messages::google::protobuf::Timestamp;

pub(crate) fn setup() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(stderr_topic_name) = resolve_topic_name("STDERR") {
        if let Some(publisher) = get_publisher::<LogMessage>(stderr_topic_name) {
            let redirect = BufferRedirect::stderr().unwrap();
            let sleep_duration = time::Duration::from_millis(20);
            let entity_name = format!(
                "{}/logs",
                std::env::var("DEPLOYED_APPLICATION_NAME").unwrap_or("".to_string())
            );
            // Read from the redirect in the main thread and send data through the channel
            thread::spawn(move || {
                let mut reader = BufReader::new(redirect);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) => {
                            thread::sleep(sleep_duration);
                        }
                        Ok(_) => {
                            let message = LogMessage {
                                header: Some(Header {
                                    timestamp: Timestamp::get_current_time().into(),
                                    reference_id: 0,
                                    entity_path: entity_name.clone(),
                                }),
                                level: log_message::LogLevel::Error as i32,
                                message: line.to_string(),
                                source: "stderr".to_string(),
                                file_name: "n/a".to_string(),
                                line_number: 0,
                                process_id: process::id() as i64,
                                thread_id: 0,
                            };

                            if let Err(e) = publisher.publish(&message) {
                                eprintln!("Error publishing message: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading from stderr buffer: {}", e);
                            break;
                        }
                    }
                }
            });
            Ok(())
        } else {
            Err("Failed to get publisher topic".into())
        }
    } else {
        Err("Failed to get topic name for STDERR".into())
    }
}