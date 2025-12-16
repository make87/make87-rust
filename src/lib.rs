//! # Deprecated
//!
//! **This SDK is deprecated and unmaintained.**
//!
//! This SDK supported the legacy make87 "application SDK integration" workflow.
//! make87 has pivoted to a CLI/agent-based device management workflow.
//!
//! **This package will receive no further updates or fixes.**
//! If you are still using it, pin your dependency and migrate off this package.
//!
//! Status: Deprecated | Maintenance: none

#[deprecated(
    since = "0.2.5",
    note = "This SDK is deprecated and unmaintained. Migrate off or pin to a lower version."
)]
pub mod encodings;

#[deprecated(
    since = "0.2.5",
    note = "This SDK is deprecated and unmaintained. Migrate off or pin to a lower version."
)]
pub mod interfaces;

#[deprecated(
    since = "0.2.5",
    note = "This SDK is deprecated and unmaintained. Migrate off or pin to a lower version."
)]
pub mod config;

mod internal;

#[deprecated(
    since = "0.2.5",
    note = "This SDK is deprecated and unmaintained. Migrate off or pin to a lower version."
)]
pub mod models;

#[deprecated(
    since = "0.2.5",
    note = "This SDK is deprecated and unmaintained. Migrate off or pin to a lower version."
)]
pub mod peripherals;

#[cfg(feature = "storage")]
#[deprecated(
    since = "0.2.5",
    note = "This SDK is deprecated and unmaintained. Migrate off or pin to a lower version."
)]
pub mod storage;

#[deprecated(
    since = "0.2.5",
    note = "This SDK is deprecated and unmaintained. Migrate off or pin to a lower version."
)]
pub fn run_forever() {
    loop {
        std::thread::park();
    }
}
