use clap::{Parser, Subcommand};
use radioxide_proto::{RadioCommand, RadioxideMessage, DEFAULT_ADDR};
use radioxide_transports::tcp;

#[derive(Parser)]
#[command(name = "radioxide-cli", about = "Command-line client for the Radioxide radio daemon")]
struct Cli {
    /// Daemon address
    #[arg(short, long, default_value = DEFAULT_ADDR, global = true)]
    addr: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Set VFO frequency in Hz (e.g., 14074000)
    Freq {
        /// Frequency in Hz
        hz: u64,
    },
    /// Get current VFO frequency
    GetFreq,
    /// Set band (e.g., 20m, 40m, 80m)
    Band {
        /// Band name
        band: String,
    },
    /// Get current band
    GetBand,
    /// Set operating mode (LSB, USB, CW, AM, FM, DIG)
    Mode {
        /// Mode name
        mode: String,
    },
    /// Get current operating mode
    GetMode,
    /// Trigger antenna tuner
    Tune,
    /// Key the transmitter (PTT on)
    PttOn,
    /// Unkey the transmitter (PTT off)
    PttOff,
    /// Set RF power output (0-100%)
    Power {
        /// Power percentage
        pct: u8,
    },
    /// Get current RF power level
    GetPower,
    /// Set AF volume (0-100%)
    Volume {
        /// Volume percentage
        pct: u8,
    },
    /// Get current AF volume
    GetVolume,
    /// Set AGC mode (off, fast, med, slow)
    Agc {
        /// AGC setting
        setting: String,
    },
    /// Get current AGC mode
    GetAgc,
    /// Get full radio status
    Status,
}

fn build_command(cmd: Command) -> Result<RadioCommand, String> {
    match cmd {
        Command::Freq { hz } => Ok(RadioCommand::SetFrequency(hz)),
        Command::GetFreq => Ok(RadioCommand::GetFrequency),
        Command::Band { band } => Ok(RadioCommand::SetBand(band.parse()?)),
        Command::GetBand => Ok(RadioCommand::GetBand),
        Command::Mode { mode } => Ok(RadioCommand::SetMode(mode.parse()?)),
        Command::GetMode => Ok(RadioCommand::GetMode),
        Command::Tune => Ok(RadioCommand::Tune),
        Command::PttOn => Ok(RadioCommand::PttOn),
        Command::PttOff => Ok(RadioCommand::PttOff),
        Command::Power { pct } => Ok(RadioCommand::SetPower(pct)),
        Command::GetPower => Ok(RadioCommand::GetPower),
        Command::Volume { pct } => Ok(RadioCommand::SetVolume(pct)),
        Command::GetVolume => Ok(RadioCommand::GetVolume),
        Command::Agc { setting } => Ok(RadioCommand::SetAgc(setting.parse()?)),
        Command::GetAgc => Ok(RadioCommand::GetAgc),
        Command::Status => Ok(RadioCommand::GetStatus),
    }
}

fn print_status(resp: &radioxide_proto::RadioxideResponse) {
    if let Some(ref st) = resp.status {
        println!("  Frequency: {} Hz", st.frequency_hz);
        println!("  Band:      {}", st.band);
        println!("  Mode:      {}", st.mode);
        println!("  Power:     {}%", st.power);
        println!("  Volume:    {}%", st.volume);
        println!("  AGC:       {}", st.agc);
        println!("  PTT:       {}", if st.ptt { "ON" } else { "OFF" });
        println!("  Tuning:    {}", if st.tuning { "YES" } else { "NO" });
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let radio_cmd = match build_command(cli.command) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let msg = RadioxideMessage {
        command: radio_cmd,
    };

    match tcp::send_message(&cli.addr, &msg).await {
        Ok(resp) => {
            if resp.success {
                println!("{}", resp.message);
                print_status(&resp);
            } else {
                eprintln!("Error: {}", resp.message);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Connection error: {e}");
            std::process::exit(1);
        }
    }
}
