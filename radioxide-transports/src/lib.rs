use radioxide_proto::{RadioxideMessage, RadioxideResponse};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use zbus::dbus_interface;
use zbus::Connection;

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
        println!("Listening on {addr}");
        loop {
            let (mut socket, peer) = listener.accept().await?;
            let handler = handler.clone();
            tokio::spawn(async move {
                println!("Client connected: {peer}");
                while let Ok(Some(frame)) = read_frame(&mut socket).await {
                    let msg: RadioxideMessage = match serde_json::from_slice(&frame) {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("Bad message from {peer}: {e}");
                            let resp = RadioxideResponse {
                                success: false,
                                message: format!("Invalid message: {e}"),
                                status: None,
                            };
                            let data = serde_json::to_vec(&resp).unwrap();
                            let _ = write_frame(&mut socket, &data).await;
                            continue;
                        }
                    };
                    let resp = handler(msg).await;
                    let data = serde_json::to_vec(&resp).unwrap();
                    if write_frame(&mut socket, &data).await.is_err() {
                        break;
                    }
                }
                println!("Client disconnected: {peer}");
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

pub mod dbus {
    use super::*;
    pub struct RadioxideDBus;

    #[dbus_interface(name = "com.radioxide.Daemon")]
    impl RadioxideDBus {
        pub fn send_command(&self, cmd: String) -> zbus::fdo::Result<String> {
            println!("DBus command received: {}", cmd);
            Ok(format!("Executed {}", cmd))
        }
    }

    pub async fn start_dbus_service() -> zbus::Result<()> {
        let connection = Connection::session().await?;
        let daemon = RadioxideDBus;
        connection
            .object_server()
            .at("/com/radioxide/Daemon", daemon)
            .await?;
        println!("DBus service running...");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }
}
