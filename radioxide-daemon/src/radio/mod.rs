pub mod dummy;
pub mod yaesu;

use async_trait::async_trait;
use radioxide_proto::{Agc, Band, Mode, RadioStatus, Vfo};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("serial port error: {0}")]
    Serial(#[from] tokio_serial::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CAT protocol error: {0}")]
    Protocol(String),

    #[error("CAT command timeout")]
    Timeout,

    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("radio not connected")]
    NotConnected,
}

pub type Result<T> = std::result::Result<T, BackendError>;

#[async_trait]
pub trait Radio: Send + Sync {
    async fn set_frequency(&self, hz: u64) -> Result<()>;
    async fn get_frequency(&self) -> Result<u64>;
    async fn set_band(&self, band: Band) -> Result<()>;
    async fn get_band(&self) -> Result<Band>;
    async fn set_mode(&self, mode: Mode) -> Result<()>;
    async fn get_mode(&self) -> Result<Mode>;
    async fn tune(&self) -> Result<()>;
    async fn set_ptt(&self, on: bool) -> Result<()>;
    async fn get_ptt(&self) -> Result<bool>;
    async fn set_power(&self, percent: u8) -> Result<()>;
    async fn get_power(&self) -> Result<u8>;
    async fn set_volume(&self, percent: u8) -> Result<()>;
    async fn get_volume(&self) -> Result<u8>;
    async fn set_agc(&self, agc: Agc) -> Result<()>;
    async fn get_agc(&self) -> Result<Agc>;
    async fn set_vfo(&self, vfo: Vfo) -> Result<()>;
    async fn get_vfo(&self) -> Result<Vfo>;
    async fn get_status(&self) -> Result<RadioStatus>;
}
