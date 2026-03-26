use clap::{Parser, Subcommand};
use radioxide_proto::{DEFAULT_ADDR, RadioCommand, RadioxideMessage, Vfo};
use radioxide_transports::tcp;

#[derive(Parser)]
#[command(
    name = "radioxide-cli",
    about = "Command-line client for the Radioxide radio daemon"
)]
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
        /// Frequency in Hz (30000–60000000)
        #[arg(value_parser = clap::value_parser!(u64).range(30_000..=60_000_000))]
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
        /// Power percentage (0–100)
        #[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
        pct: u8,
    },
    /// Get current RF power level
    GetPower,
    /// Set AF volume (0-100%)
    Volume {
        /// Volume percentage (0–100)
        #[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
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
    /// Select active VFO (A or B)
    Vfo {
        /// VFO to select (A or B)
        vfo: String,
    },
    /// Get active VFO
    GetVfo,
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
        Command::Vfo { vfo } => Ok(RadioCommand::SetVfo(vfo.parse::<Vfo>()?)),
        Command::GetVfo => Ok(RadioCommand::GetVfo),
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
        println!("  VFO:       {}", st.vfo);
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

    let msg = RadioxideMessage { command: radio_cmd };

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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        let mut full = vec!["radioxide-cli"];
        full.extend_from_slice(args);
        Cli::try_parse_from(full)
    }

    #[test]
    fn parse_freq() {
        let cli = parse(&["freq", "14074000"]).unwrap();
        assert!(matches!(cli.command, Command::Freq { hz: 14_074_000 }));
    }

    #[test]
    fn parse_freq_out_of_range() {
        assert!(parse(&["freq", "999999999"]).is_err());
        assert!(parse(&["freq", "0"]).is_err());
    }

    #[test]
    fn parse_band() {
        let cli = parse(&["band", "20m"]).unwrap();
        assert!(matches!(cli.command, Command::Band { ref band } if band == "20m"));
    }

    #[test]
    fn parse_mode() {
        let cli = parse(&["mode", "USB"]).unwrap();
        assert!(matches!(cli.command, Command::Mode { ref mode } if mode == "USB"));
    }

    #[test]
    fn parse_power() {
        let cli = parse(&["power", "50"]).unwrap();
        assert!(matches!(cli.command, Command::Power { pct: 50 }));
    }

    #[test]
    fn parse_power_out_of_range() {
        assert!(parse(&["power", "200"]).is_err());
    }

    #[test]
    fn parse_volume() {
        let cli = parse(&["volume", "30"]).unwrap();
        assert!(matches!(cli.command, Command::Volume { pct: 30 }));
    }

    #[test]
    fn parse_agc() {
        let cli = parse(&["agc", "fast"]).unwrap();
        assert!(matches!(cli.command, Command::Agc { ref setting } if setting == "fast"));
    }

    #[test]
    fn parse_status() {
        let cli = parse(&["status"]).unwrap();
        assert!(matches!(cli.command, Command::Status));
    }

    #[test]
    fn parse_tune() {
        let cli = parse(&["tune"]).unwrap();
        assert!(matches!(cli.command, Command::Tune));
    }

    #[test]
    fn parse_ptt_on_off() {
        let cli = parse(&["ptt-on"]).unwrap();
        assert!(matches!(cli.command, Command::PttOn));
        let cli = parse(&["ptt-off"]).unwrap();
        assert!(matches!(cli.command, Command::PttOff));
    }

    #[test]
    fn parse_custom_addr() {
        let cli = parse(&["--addr", "10.0.0.1:8080", "status"]).unwrap();
        assert_eq!(cli.addr, "10.0.0.1:8080");
    }

    #[test]
    fn build_command_all_variants() {
        assert_eq!(
            build_command(Command::Freq { hz: 7_074_000 }).unwrap(),
            RadioCommand::SetFrequency(7_074_000)
        );
        assert_eq!(
            build_command(Command::GetFreq).unwrap(),
            RadioCommand::GetFrequency
        );
        assert_eq!(
            build_command(Command::Band { band: "20m".into() }).unwrap(),
            RadioCommand::SetBand(radioxide_proto::Band::Band20m)
        );
        assert_eq!(
            build_command(Command::GetBand).unwrap(),
            RadioCommand::GetBand
        );
        assert_eq!(
            build_command(Command::Mode { mode: "USB".into() }).unwrap(),
            RadioCommand::SetMode(radioxide_proto::Mode::USB)
        );
        assert_eq!(
            build_command(Command::GetMode).unwrap(),
            RadioCommand::GetMode
        );
        assert_eq!(build_command(Command::Tune).unwrap(), RadioCommand::Tune);
        assert_eq!(build_command(Command::PttOn).unwrap(), RadioCommand::PttOn);
        assert_eq!(
            build_command(Command::PttOff).unwrap(),
            RadioCommand::PttOff
        );
        assert_eq!(
            build_command(Command::Power { pct: 75 }).unwrap(),
            RadioCommand::SetPower(75)
        );
        assert_eq!(
            build_command(Command::GetPower).unwrap(),
            RadioCommand::GetPower
        );
        assert_eq!(
            build_command(Command::Volume { pct: 30 }).unwrap(),
            RadioCommand::SetVolume(30)
        );
        assert_eq!(
            build_command(Command::GetVolume).unwrap(),
            RadioCommand::GetVolume
        );
        assert_eq!(
            build_command(Command::Agc {
                setting: "fast".into()
            })
            .unwrap(),
            RadioCommand::SetAgc(radioxide_proto::Agc::Fast)
        );
        assert_eq!(
            build_command(Command::GetAgc).unwrap(),
            RadioCommand::GetAgc
        );
        assert_eq!(
            build_command(Command::Vfo { vfo: "A".into() }).unwrap(),
            RadioCommand::SetVfo(Vfo::A)
        );
        assert_eq!(
            build_command(Command::Vfo { vfo: "B".into() }).unwrap(),
            RadioCommand::SetVfo(Vfo::B)
        );
        assert_eq!(
            build_command(Command::GetVfo).unwrap(),
            RadioCommand::GetVfo
        );
        assert_eq!(
            build_command(Command::Status).unwrap(),
            RadioCommand::GetStatus
        );
    }

    #[test]
    fn parse_vfo() {
        let cli = parse(&["vfo", "A"]).unwrap();
        assert!(matches!(cli.command, Command::Vfo { ref vfo } if vfo == "A"));
        let cli = parse(&["vfo", "b"]).unwrap();
        assert!(matches!(cli.command, Command::Vfo { ref vfo } if vfo == "b"));
    }

    #[test]
    fn parse_get_vfo() {
        let cli = parse(&["get-vfo"]).unwrap();
        assert!(matches!(cli.command, Command::GetVfo));
    }

    #[test]
    fn build_command_vfo_invalid() {
        assert!(build_command(Command::Vfo { vfo: "C".into() }).is_err());
    }
}
