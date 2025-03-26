use crate::topics::TypedPublisher;
use crate::{get_publisher, resolve_topic_name};
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use make87_messages::core::Header;
use make87_messages::text::{log_message, LogMessage};
use make87_messages::well_known_types::Timestamp;
use make87_messages::CurrentTime;
use once_cell::sync::Lazy;
use std::process;
use std::sync::Once;

static LOGGER: Lazy<Option<&'static Logger>> = Lazy::new(|| {
    if let Some(log_topic_name) = resolve_topic_name("LOGS") {
        if let Some(log_topic) = get_publisher::<LogMessage>(log_topic_name) {
            let logger = Box::new(Logger::new(log_topic));
            Some(Box::leak(logger))
        } else {
            None
        }
    } else {
        None
    }
});

static INIT: Once = Once::new();

struct Logger {
    log_topic: TypedPublisher<LogMessage>,
    entity_name: String,
}

impl Logger {
    fn new(log_topic: TypedPublisher<LogMessage>) -> Self {
        Logger {
            log_topic,
            entity_name: format!(
                "{}/logs",
                std::env::var("DEPLOYED_APPLICATION_NAME").unwrap_or("".to_string())
            ),
        }
    }
}
impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = LogMessage {
                header: Some(Header {
                    timestamp: Timestamp::get_current_time(),
                    reference_id: 0,
                    entity_path: self.entity_name.clone(),
                }),
                level: match record.metadata().level() {
                    Level::Error => log_message::LogLevel::Error as i32,
                    Level::Warn => log_message::LogLevel::Warning as i32,
                    Level::Info => log_message::LogLevel::Info as i32,
                    Level::Debug => log_message::LogLevel::Debug as i32,
                    _ => log_message::LogLevel::Debug as i32,
                },
                message: record.args().to_string(),
                source: record.module_path().unwrap_or("n/a").to_string(),
                file_name: record.file().unwrap_or("n/a").to_string(),
                line_number: match record.line() {
                    Some(i) => i as i32,
                    None => -1,
                },
                process_id: process::id() as i64,
                thread_id: 0,
            };

            self.log_topic.publish(&message).unwrap();
        }
    }

    fn flush(&self) {}
}

pub(crate) fn setup() -> Result<(), SetLoggerError> {
    if let Some(logger) = *LOGGER {
        INIT.call_once(|| {
            log::set_logger(logger).unwrap();
            log::set_max_level(LevelFilter::Info);
        });
    }
    Ok(())
}
