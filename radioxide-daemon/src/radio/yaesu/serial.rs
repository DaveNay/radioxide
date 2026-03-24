use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio_serial::SerialPortBuilderExt;
use tracing::{info, warn};

use super::cat::{CatCommand, CatResponse};
use crate::radio::BackendError;

/// Configuration for a serial CAT connection.
#[derive(Clone)]
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

/// Maximum number of reconnection attempts before giving up.
const MAX_RECONNECT_ATTEMPTS: u32 = 3;

/// Delay between reconnection attempts.
const RECONNECT_DELAY: Duration = Duration::from_secs(1);

/// Open a serial port with FT-450D settings (8N2).
fn open_serial(config: &SerialConfig) -> crate::radio::Result<tokio_serial::SerialStream> {
    let port = tokio_serial::new(&config.port_path, config.baud_rate)
        .data_bits(tokio_serial::DataBits::Eight)
        .parity(tokio_serial::Parity::None)
        .stop_bits(tokio_serial::StopBits::Two)
        .flow_control(tokio_serial::FlowControl::None)
        .timeout(config.read_timeout)
        .open_native_async()?;
    Ok(port)
}

/// Async CAT serial port wrapper with automatic reconnection.
/// Thread-safe via internal Mutex.
pub struct CatPort {
    inner: Mutex<Option<tokio_serial::SerialStream>>,
    config: SerialConfig,
}

impl CatPort {
    /// Open a serial port with FT-450D settings (8N2).
    pub fn open(config: &SerialConfig) -> crate::radio::Result<Self> {
        let port = open_serial(config)?;
        Ok(Self {
            inner: Mutex::new(Some(port)),
            config: config.clone(),
        })
    }

    /// Attempt to reconnect the serial port. Returns the new stream or an error.
    async fn reconnect(&self, guard: &mut Option<tokio_serial::SerialStream>) -> crate::radio::Result<()> {
        for attempt in 1..=MAX_RECONNECT_ATTEMPTS {
            warn!(
                "Serial connection lost, reconnect attempt {}/{}...",
                attempt, MAX_RECONNECT_ATTEMPTS
            );
            tokio::time::sleep(RECONNECT_DELAY).await;
            match open_serial(&self.config) {
                Ok(port) => {
                    info!("Serial port reconnected");
                    *guard = Some(port);
                    return Ok(());
                }
                Err(e) => {
                    warn!("Reconnect attempt {} failed: {e}", attempt);
                }
            }
        }
        *guard = None;
        Err(BackendError::NotConnected)
    }

    /// Ensure the port is connected, reconnecting if necessary.
    /// Must be called with the lock held.
    async fn ensure_connected(
        &self,
        guard: &mut Option<tokio_serial::SerialStream>,
    ) -> crate::radio::Result<()> {
        if guard.is_none() {
            self.reconnect(guard).await?;
        }
        Ok(())
    }

    /// Send a command and read the response. Returns parsed CatResponse.
    /// Retries once on I/O or timeout errors after reconnecting.
    pub async fn execute(&self, cmd: &CatCommand) -> crate::radio::Result<CatResponse> {
        let mut guard = self.inner.lock().await;
        self.ensure_connected(&mut guard).await?;

        let result = Self::do_execute(guard.as_mut().unwrap(), cmd).await;
        match result {
            Ok(resp) => Ok(resp),
            Err(BackendError::Io(_) | BackendError::Serial(_) | BackendError::Timeout) => {
                self.reconnect(&mut guard).await?;
                Self::do_execute(guard.as_mut().unwrap(), cmd).await
            }
            Err(e) => Err(e),
        }
    }

    async fn do_execute(
        port: &mut tokio_serial::SerialStream,
        cmd: &CatCommand,
    ) -> crate::radio::Result<CatResponse> {
        port.write_all(cmd.as_bytes()).await?;
        port.flush().await?;

        let raw = read_until_semicolon(port).await?;
        let response_str =
            String::from_utf8(raw).map_err(|e| BackendError::Protocol(format!("invalid UTF-8: {e}")))?;

        CatResponse::parse(&response_str, cmd.verb())
    }

    /// Send a set command. For commands that echo back a response, reads and discards it.
    /// For commands with no response, returns after a short read attempt.
    /// Retries once on I/O or timeout errors after reconnecting.
    pub async fn send(&self, cmd: &CatCommand) -> crate::radio::Result<()> {
        let mut guard = self.inner.lock().await;
        self.ensure_connected(&mut guard).await?;

        let result = Self::do_send(guard.as_mut().unwrap(), cmd).await;
        match result {
            Ok(()) => Ok(()),
            Err(BackendError::Io(_) | BackendError::Serial(_) | BackendError::Timeout) => {
                self.reconnect(&mut guard).await?;
                Self::do_send(guard.as_mut().unwrap(), cmd).await
            }
            Err(e) => Err(e),
        }
    }

    async fn do_send(
        port: &mut tokio_serial::SerialStream,
        cmd: &CatCommand,
    ) -> crate::radio::Result<()> {
        port.write_all(cmd.as_bytes()).await?;
        port.flush().await?;

        // Many set commands on the FT-450D don't return a response.
        // Do a short non-blocking read to drain any echo, but don't fail if nothing comes back.
        let mut buf = [0u8; 64];
        match tokio::time::timeout(Duration::from_millis(100), port.read(&mut buf)).await {
            Ok(Ok(_)) => {}
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
