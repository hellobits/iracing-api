//! Rust implementation of iRacing's SDK
//!
//! iRacing provides a C-based SDK for its telemetry and session API. The SDK can be used as a
//! reference implementation, and it defines many important parameters on how to read the shared
//! memory mapped file. This module implements the relevant parts of the SDK in Rust, so that the
//! crate can read the telemetry and session data from memory.

pub const DATA_READY_EVENT: &str = "Local\\IRSDKDataValidEvent";
pub const MEMORY_MAPPED_FILE: &str = "Local\\IRSDKMemMapFileName";
