pub mod encodings;
pub mod interfaces;

pub mod config;
mod internal;
pub mod models;

pub fn run_forever() {
    loop {
        std::thread::park();
    }
}
