use radioxide_proto::{RadioxideMessage, RadioxideResponse, DEFAULT_ADDR};
use radioxide_transports::tcp;

fn handle_command(msg: RadioxideMessage) -> RadioxideResponse {
    println!("Received: {:?}", msg);
    match msg.command {
        radioxide_proto::RadioxideCommand::Play => RadioxideResponse {
            success: true,
            message: "Playback started".into(),
        },
        radioxide_proto::RadioxideCommand::Pause => RadioxideResponse {
            success: true,
            message: "Playback paused".into(),
        },
        radioxide_proto::RadioxideCommand::Stop => RadioxideResponse {
            success: true,
            message: "Playback stopped".into(),
        },
        radioxide_proto::RadioxideCommand::SetVolume(vol) => RadioxideResponse {
            success: true,
            message: format!("Volume set to {vol}"),
        },
    }
}

#[tokio::main]
async fn main() {
    println!("Radioxide daemon starting...");
    if let Err(e) = tcp::start_server(DEFAULT_ADDR, handle_command).await {
        eprintln!("Daemon error: {e}");
    }
}
