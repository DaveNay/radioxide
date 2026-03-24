use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio_serial::SerialPortBuilderExt;

use super::cat::{CatCommand, CatResponse};
use crate::radio::BackendError;

/// Configuration for a serial CAT connection.
pub struct SerialConfig {
    pub port_path: String,
    pub baud_rate: u32,
    pub read_timeout: Duration,
}

impl SerialConfig {
    pub fn new(port_path: String, baud_rate: u32) -> Self {
        Self {
            port_path,
            baud_rate,
            read_timeout: Duration::from_secs(2),
        }
    }
}

/// Async CAT serial port wrapper. Thread-safe via internal Mutex.
pub struct CatPort {
    inner: Mutex<tokio_serial::SerialStream>,
}

impl CatPort {
    /// Open a serial port with FT-450D settings (8N2).
    pub fn open(config: &SerialConfig) -> crate::radio::Result<Self> {
        let port = tokio_serial::new(&config.port_path, config.baud_rate)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::Two)
            .flow_control(tokio_serial::FlowControl::None)
            .timeout(config.read_timeout)
            .open_native_async()?;

        Ok(Self {
            inner: Mutex::new(port),
        })
    }

    /// Send a command and read the response. Returns parsed CatResponse.
    pub async fn execute(&self, cmd: &CatCommand) -> crate::radio::Result<CatResponse> {
        let mut port = self.inner.lock().await;

        // Write command
        port.write_all(cmd.as_bytes()).await?;
        port.flush().await?;

        // Read until ';' terminator
        let raw = read_until_semicolon(&mut port).await?;
        let response_str =
            String::from_utf8(raw).map_err(|e| BackendError::Protocol(format!("invalid UTF-8: {e}")))?;

        CatResponse::parse(&response_str, cmd.verb())
    }

    /// Send a set command. For commands that echo back a response, reads and discards it.
    /// For commands with no response, returns after a short read attempt.
    pub async fn send(&self, cmd: &CatCommand) -> crate::radio::Result<()> {
        let mut port = self.inner.lock().await;

        port.write_all(cmd.as_bytes()).await?;
        port.flush().await?;

        // Many set commands on the FT-450D don't return a response.
        // Do a short non-blocking read to drain any echo, but don't fail if nothing comes back.
        let mut buf = [0u8; 64];
        match tokio::time::timeout(Duration::from_millis(100), port.read(&mut buf)).await {
            Ok(Ok(_)) => {} // drained any echo
            Ok(Err(_)) => {}
            Err(_) => {} // timeout is fine — no response expected
        }

        Ok(())
    }
}

/// Read bytes from the port until a `;` terminator is found.
async fn read_until_semicolon(
    port: &mut tokio_serial::SerialStream,
) -> crate::radio::Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(64);
    let mut byte = [0u8; 1];

    let result = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            port.read_exact(&mut byte).await?;
            buf.push(byte[0]);
            if byte[0] == b';' {
                return Ok::<_, std::io::Error>(());
            }
        }
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(buf),
        Ok(Err(e)) => Err(BackendError::Io(e)),
        Err(_) => Err(BackendError::Timeout),
    }
}
