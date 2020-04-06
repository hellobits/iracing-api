use iracing_api::sdk::Client;
use iracing_api::sdk::IRacingClient;
use std::io::ErrorKind;

fn main() {
    match Client::new() {
        Ok(_) => (),
        Err(error) => match error.kind() {
            ErrorKind::Other => (), // Unsupported operating system
            error => panic!(error),
        },
    }
}
