pub mod encodings;
pub mod interfaces;

pub mod config;
mod internal;
pub mod models;
pub mod peripherals;
#[cfg(feature = "storage")]
pub mod storage;

pub fn run_forever() {
    loop {
        std::thread::park();
    }
}
