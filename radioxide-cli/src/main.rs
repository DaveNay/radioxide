use clap::Parser;
use radioxide_proto::{RadioxideCommand, RadioxideMessage, DEFAULT_ADDR};
use radioxide_transports::tcp;

#[derive(Parser)]
#[command(name = "radioxide-cli")]
struct Cli {
    /// Command to send: play, pause, stop, or volume:<0-100>
    #[arg(short, long)]
    command: String,

    /// Daemon address
    #[arg(short, long, default_value = DEFAULT_ADDR)]
    addr: String,
}

fn parse_command(s: &str) -> Result<RadioxideCommand, String> {
    match s.to_lowercase().as_str() {
        "play" => Ok(RadioxideCommand::Play),
        "pause" => Ok(RadioxideCommand::Pause),
        "stop" => Ok(RadioxideCommand::Stop),
        v if v.starts_with("volume:") => {
            let vol: u8 = v["volume:".len()..]
                .parse()
                .map_err(|_| "volume must be 0-255".to_string())?;
            Ok(RadioxideCommand::SetVolume(vol))
        }
        other => Err(format!("unknown command: {other}")),
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let command = match parse_command(&cli.command) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let msg = RadioxideMessage {
        command,
        payload: None,
    };

    match tcp::send_message(&cli.addr, &msg).await {
        Ok(resp) => {
            if resp.success {
                println!("{}", resp.message);
            } else {
                eprintln!("Server error: {}", resp.message);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Connection error: {e}");
            std::process::exit(1);
        }
    }
}
