use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum SdkError {
    #[error("schema mismatch: {0}")]
    Schema(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

pub trait GameAdapter {
    fn version(&self) -> Version;
    // future: hooks for feeding snapshots, receiving intents via IPC (gRPC/WebSocket)
}
