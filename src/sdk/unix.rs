use crate::sdk::IRacingClient;
use std::io::{Error, ErrorKind};

pub struct UnixClient {}

impl IRacingClient for UnixClient {
    fn new() -> Result<Self, Error> {
        Err(Error::new(
            ErrorKind::Other,
            "Unsupported platform. iRacing only supports Windows.",
        ))
    }
}
