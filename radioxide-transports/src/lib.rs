use radioxide_proto::{RadioxideMessage, RadioxideResponse};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

/// Read a length-prefixed frame: 4-byte big-endian length, then that many bytes of JSON.
async fn read_frame(stream: &mut TcpStream) -> tokio::io::Result<Option<Vec<u8>>> {
    let len = match stream.read_u32().await {
        Ok(n) => n as usize,
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    };
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    Ok(Some(buf))
}

/// Write a length-prefixed frame.
async fn write_frame(stream: &mut TcpStream, data: &[u8]) -> tokio::io::Result<()> {
    stream.write_u32(data.len() as u32).await?;
    stream.write_all(data).await?;
    stream.flush().await?;
    Ok(())
}

pub mod tcp {
    use super::*;

    /// Start a TCP server that deserializes incoming `RadioxideMessage`s and
    /// invokes `handler` for each one, sending the returned response back.
    /// The handler returns a future, allowing async operations (e.g., serial I/O).
    pub async fn start_server<F, Fut>(addr: &str, handler: F) -> tokio::io::Result<()>
    where
        F: Fn(RadioxideMessage) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = RadioxideResponse> + Send,
    {
        let handler = std::sync::Arc::new(handler);
        let listener = TcpListener::bind(addr).await?;
        info!("Listening on {addr}");
        loop {
            let (mut socket, peer) = listener.accept().await?;
            let handler = handler.clone();
            tokio::spawn(async move {
                info!("Client connected: {peer}");
                while let Ok(Some(frame)) = read_frame(&mut socket).await {
                    let msg: RadioxideMessage = match serde_json::from_slice(&frame) {
                        Ok(m) => m,
                        Err(e) => {
                            warn!("Bad message from {peer}: {e}");
                            let resp = RadioxideResponse {
                                success: false,
                                message: format!("Invalid message: {e}"),
                                status: None,
                            };
                            let data = match serde_json::to_vec(&resp) {
                                Ok(d) => d,
                                Err(e) => {
                                    error!("Failed to serialize error response for {peer}: {e}");
                                    break;
                                }
                            };
                            let _ = write_frame(&mut socket, &data).await;
                            continue;
                        }
                    };
                    let resp = handler(msg).await;
                    let data = match serde_json::to_vec(&resp) {
                        Ok(d) => d,
                        Err(e) => {
                            error!("Failed to serialize response for {peer}: {e}");
                            break;
                        }
                    };
                    if write_frame(&mut socket, &data).await.is_err() {
                        break;
                    }
                }
                info!("Client disconnected: {peer}");
            });
        }
    }

    /// Send a single `RadioxideMessage` to the server and return its response.
    pub async fn send_message(
        addr: &str,
        msg: &RadioxideMessage,
    ) -> tokio::io::Result<RadioxideResponse> {
        let mut stream = TcpStream::connect(addr).await?;
        let serialized =
            serde_json::to_vec(msg).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        write_frame(&mut stream, &serialized).await?;

        let frame = read_frame(&mut stream)
            .await?
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "no response"))?;
        let resp: RadioxideResponse = serde_json::from_slice(&frame)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::tcp;
    use radioxide_proto::*;
    use std::sync::atomic::{AtomicU16, Ordering};

    static PORT_COUNTER: AtomicU16 = AtomicU16::new(17600);

    fn next_addr() -> String {
        let port = PORT_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("127.0.0.1:{port}")
    }

    fn canned_handler() -> impl Fn(RadioxideMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = RadioxideResponse> + Send>> + Send + Sync + 'static {
        move |msg: RadioxideMessage| {
            Box::pin(async move {
                let message = format!("handled: {:?}", msg.command);
                RadioxideResponse {
                    success: true,
                    message,
                    status: Some(RadioStatus::default()),
                }
            })
        }
    }

    #[tokio::test]
    async fn tcp_roundtrip() {
        let addr = next_addr();
        let server_addr = addr.clone();
        tokio::spawn(async move {
            let _ = tcp::start_server(&server_addr, canned_handler()).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let msg = RadioxideMessage { command: RadioCommand::GetStatus };
        let resp = tcp::send_message(&addr, &msg).await.unwrap();
        assert!(resp.success);
        assert!(resp.status.is_some());
    }

    #[tokio::test]
    async fn tcp_set_frequency() {
        let addr = next_addr();
        let server_addr = addr.clone();
        tokio::spawn(async move {
            let _ = tcp::start_server(&server_addr, canned_handler()).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let msg = RadioxideMessage { command: RadioCommand::SetFrequency(7_074_000) };
        let resp = tcp::send_message(&addr, &msg).await.unwrap();
        assert!(resp.success);
        assert!(resp.message.contains("SetFrequency"));
    }

    #[tokio::test]
    async fn tcp_multiple_commands() {
        let addr = next_addr();
        let server_addr = addr.clone();
        tokio::spawn(async move {
            let _ = tcp::start_server(&server_addr, canned_handler()).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        for cmd in [RadioCommand::GetStatus, RadioCommand::GetFrequency, RadioCommand::GetBand] {
            let msg = RadioxideMessage { command: cmd };
            let resp = tcp::send_message(&addr, &msg).await.unwrap();
            assert!(resp.success);
        }
    }

    #[tokio::test]
    async fn tcp_concurrent_clients() {
        let addr = next_addr();
        let server_addr = addr.clone();
        tokio::spawn(async move {
            let _ = tcp::start_server(&server_addr, canned_handler()).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let addr1 = addr.clone();
        let addr2 = addr.clone();
        let (r1, r2) = tokio::join!(
            async move {
                let msg = RadioxideMessage { command: RadioCommand::GetStatus };
                tcp::send_message(&addr1, &msg).await.unwrap()
            },
            async move {
                let msg = RadioxideMessage { command: RadioCommand::GetFrequency };
                tcp::send_message(&addr2, &msg).await.unwrap()
            }
        );
        assert!(r1.success);
        assert!(r2.success);
    }

    #[tokio::test]
    async fn tcp_malformed_json() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpStream;

        let addr = next_addr();
        let server_addr = addr.clone();
        tokio::spawn(async move {
            let _ = tcp::start_server(&server_addr, canned_handler()).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let mut stream = TcpStream::connect(&addr).await.unwrap();
        let garbage = b"not valid json at all";
        stream.write_u32(garbage.len() as u32).await.unwrap();
        stream.write_all(garbage).await.unwrap();
        stream.flush().await.unwrap();

        // Server should respond with an error, not crash
        let len = stream.read_u32().await.unwrap() as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await.unwrap();
        let resp: RadioxideResponse = serde_json::from_slice(&buf).unwrap();
        assert!(!resp.success);
        assert!(resp.message.contains("Invalid message"));
    }
}

#[cfg(target_os = "linux")]
pub mod dbus {
    use super::*;
    use radioxide_proto::{Agc, Band, Mode, RadioCommand, RadioStatus};
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use zbus::dbus_interface;
    use zbus::fdo;
    use zbus::ConnectionBuilder;

    type Handler = Arc<
        dyn Fn(RadioxideMessage) -> Pin<Box<dyn Future<Output = RadioxideResponse> + Send>>
            + Send
            + Sync,
    >;

    pub struct RadioxideDBus {
        handler: Handler,
    }

    impl RadioxideDBus {
        fn msg(command: RadioCommand) -> RadioxideMessage {
            RadioxideMessage { command }
        }

        /// Extract status from a successful response, or return a D-Bus error.
        fn require_status(resp: &RadioxideResponse) -> fdo::Result<&RadioStatus> {
            resp.status
                .as_ref()
                .ok_or_else(|| fdo::Error::Failed(resp.message.clone()))
        }
    }

    #[dbus_interface(name = "com.radioxide.Daemon")]
    impl RadioxideDBus {
        async fn set_frequency(&self, hz: u64) -> fdo::Result<bool> {
            let resp = (self.handler)(Self::msg(RadioCommand::SetFrequency(hz))).await;
            Ok(resp.success)
        }

        async fn get_frequency(&self) -> fdo::Result<u64> {
            let resp = (self.handler)(Self::msg(RadioCommand::GetFrequency)).await;
            Self::require_status(&resp).map(|s| s.frequency_hz)
        }

        async fn set_band(&self, band: String) -> fdo::Result<bool> {
            let band: Band = band
                .parse()
                .map_err(|e: String| fdo::Error::InvalidArgs(e))?;
            let resp = (self.handler)(Self::msg(RadioCommand::SetBand(band))).await;
            Ok(resp.success)
        }

        async fn get_band(&self) -> fdo::Result<String> {
            let resp = (self.handler)(Self::msg(RadioCommand::GetBand)).await;
            Self::require_status(&resp).map(|s| s.band.to_string())
        }

        async fn set_mode(&self, mode: String) -> fdo::Result<bool> {
            let mode: Mode = mode
                .parse()
                .map_err(|e: String| fdo::Error::InvalidArgs(e))?;
            let resp = (self.handler)(Self::msg(RadioCommand::SetMode(mode))).await;
            Ok(resp.success)
        }

        async fn get_mode(&self) -> fdo::Result<String> {
            let resp = (self.handler)(Self::msg(RadioCommand::GetMode)).await;
            Self::require_status(&resp).map(|s| s.mode.to_string())
        }

        async fn tune(&self) -> fdo::Result<bool> {
            let resp = (self.handler)(Self::msg(RadioCommand::Tune)).await;
            Ok(resp.success)
        }

        async fn ptt_on(&self) -> fdo::Result<bool> {
            let resp = (self.handler)(Self::msg(RadioCommand::PttOn)).await;
            Ok(resp.success)
        }

        async fn ptt_off(&self) -> fdo::Result<bool> {
            let resp = (self.handler)(Self::msg(RadioCommand::PttOff)).await;
            Ok(resp.success)
        }

        async fn set_power(&self, percent: u8) -> fdo::Result<bool> {
            let resp = (self.handler)(Self::msg(RadioCommand::SetPower(percent))).await;
            Ok(resp.success)
        }

        async fn get_power(&self) -> fdo::Result<u8> {
            let resp = (self.handler)(Self::msg(RadioCommand::GetPower)).await;
            Self::require_status(&resp).map(|s| s.power)
        }

        async fn set_volume(&self, percent: u8) -> fdo::Result<bool> {
            let resp = (self.handler)(Self::msg(RadioCommand::SetVolume(percent))).await;
            Ok(resp.success)
        }

        async fn get_volume(&self) -> fdo::Result<u8> {
            let resp = (self.handler)(Self::msg(RadioCommand::GetVolume)).await;
            Self::require_status(&resp).map(|s| s.volume)
        }

        async fn set_agc(&self, agc: String) -> fdo::Result<bool> {
            let agc: Agc = agc
                .parse()
                .map_err(|e: String| fdo::Error::InvalidArgs(e))?;
            let resp = (self.handler)(Self::msg(RadioCommand::SetAgc(agc))).await;
            Ok(resp.success)
        }

        async fn get_agc(&self) -> fdo::Result<String> {
            let resp = (self.handler)(Self::msg(RadioCommand::GetAgc)).await;
            Self::require_status(&resp).map(|s| s.agc.to_string())
        }

        async fn get_status(&self) -> fdo::Result<String> {
            let resp = (self.handler)(Self::msg(RadioCommand::GetStatus)).await;
            let status = Self::require_status(&resp)?;
            serde_json::to_string(status)
                .map_err(|e| fdo::Error::Failed(format!("serialization error: {e}")))
        }
    }

    /// Start a D-Bus service that dispatches radio commands via the given handler.
    /// Mirrors the TCP `start_server` pattern with the same handler signature.
    pub async fn start_dbus_service<F, Fut>(handler: F) -> zbus::Result<()>
    where
        F: Fn(RadioxideMessage) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = RadioxideResponse> + Send + 'static,
    {
        let handler: Handler = Arc::new(move |msg| Box::pin(handler(msg)));
        let daemon = RadioxideDBus { handler };

        let _connection = ConnectionBuilder::session()?
            .name("com.radioxide.Daemon")?
            .serve_at("/com/radioxide/Daemon", daemon)?
            .build()
            .await?;

        info!("D-Bus service running on session bus (com.radioxide.Daemon)");

        // Keep the service alive indefinitely.
        std::future::pending::<()>().await;
        Ok(())
    }
}
