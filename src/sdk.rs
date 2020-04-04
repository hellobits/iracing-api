//! Rust implementation of iRacing's SDK
//!
//! iRacing provides a C-based SDK for its telemetry and session API. The SDK can be used as a
//! reference implementation, and it defines many important parameters on how to read the shared
//! memory mapped file. This module implements the relevant parts of the SDK in Rust, so that the
//! crate can read the telemetry and session data from memory.

use std::io::Error;

#[cfg(not(windows))]
mod unix;
#[cfg(not(windows))]
pub use unix::UnixClient as Client;

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::WinClient as Client;

pub trait IRacingClient {
    fn new() -> Result<Self, Error>
    where
        Self: Sized;
}
