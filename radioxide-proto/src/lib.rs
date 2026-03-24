use serde::{Serialize, Deserialize};

pub const DEFAULT_PORT: u16 = 7600;
pub const DEFAULT_ADDR: &str = "127.0.0.1:7600";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RadioxideCommand {
    Play,
    Pause,
    Stop,
    SetVolume(u8),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RadioxideMessage {
    pub command: RadioxideCommand,
    pub payload: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RadioxideResponse {
    pub success: bool,
    pub message: String,
}
