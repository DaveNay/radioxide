mod radio;

use std::sync::Arc;

use clap::Parser;
use radioxide_proto::{RadioCommand, RadioxideMessage, RadioxideResponse, DEFAULT_ADDR};
use radioxide_transports::tcp;

use radio::dummy::DummyRadio;
use radio::yaesu::ft450d::Ft450d;
use radio::yaesu::serial::SerialConfig;
use radio::Radio;

#[derive(Parser)]
#[command(name = "radioxide-daemon", about = "Radioxide radio control daemon")]
struct Args {
    /// Serial port for the radio (e.g., /dev/ttyUSB0). If omitted, uses a dummy backend.
    #[arg(long)]
    serial: Option<String>,

    /// Serial port baud rate
    #[arg(long, default_value = "9600")]
    baud: u32,

    /// Listen address
    #[arg(long, default_value = DEFAULT_ADDR)]
    addr: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let radio: Arc<dyn Radio> = if let Some(port) = args.serial {
        println!("Opening serial port {port} at {} baud...", args.baud);
        let config = SerialConfig::new(port, args.baud);
        match Ft450d::new(config) {
            Ok(r) => Arc::new(r),
            Err(e) => {
                eprintln!("Failed to open radio: {e}");
                std::process::exit(1);
            }
        }
    } else {
        println!("No --serial specified, using dummy backend");
        Arc::new(DummyRadio::new())
    };

    println!("Radioxide daemon starting...");

    let handler = move |msg: RadioxideMessage| {
        let radio = radio.clone();
        async move { dispatch(radio, msg).await }
    };

    if let Err(e) = tcp::start_server(&args.addr, handler).await {
        eprintln!("Daemon error: {e}");
    }
}

async fn dispatch(radio: Arc<dyn Radio>, msg: RadioxideMessage) -> RadioxideResponse {
    println!("Received: {:?}", msg.command);

    // Get commands return a formatted value; Set commands return a confirmation message.
    // Both include current status in the response.
    let result: Result<String, radio::BackendError> = match msg.command {
        RadioCommand::SetFrequency(hz) => radio.set_frequency(hz).await.map(|_| "Frequency set".into()),
        RadioCommand::GetFrequency => radio.get_frequency().await.map(|hz| format!("Frequency: {hz} Hz")),
        RadioCommand::SetBand(band) => radio.set_band(band).await.map(|_| format!("Band set to {band}")),
        RadioCommand::GetBand => radio.get_band().await.map(|b| format!("Band: {b}")),
        RadioCommand::SetMode(mode) => radio.set_mode(mode).await.map(|_| format!("Mode set to {mode}")),
        RadioCommand::GetMode => radio.get_mode().await.map(|m| format!("Mode: {m}")),
        RadioCommand::Tune => radio.tune().await.map(|_| "Tuning started".into()),
        RadioCommand::PttOn => radio.set_ptt(true).await.map(|_| "PTT on".into()),
        RadioCommand::PttOff => radio.set_ptt(false).await.map(|_| "PTT off".into()),
        RadioCommand::SetPower(pct) => radio.set_power(pct).await.map(|_| format!("Power set to {pct}%")),
        RadioCommand::GetPower => radio.get_power().await.map(|p| format!("Power: {p}%")),
        RadioCommand::SetVolume(pct) => radio.set_volume(pct).await.map(|_| format!("Volume set to {pct}%")),
        RadioCommand::GetVolume => radio.get_volume().await.map(|v| format!("Volume: {v}%")),
        RadioCommand::SetAgc(agc) => radio.set_agc(agc).await.map(|_| format!("AGC set to {agc}")),
        RadioCommand::GetAgc => radio.get_agc().await.map(|a| format!("AGC: {a}")),
        RadioCommand::GetStatus => radio.get_status().await.map(|_| "Radio status".into()),
    };

    match result {
        Ok(message) => {
            let status = radio.get_status().await.ok();
            RadioxideResponse {
                success: true,
                message,
                status,
            }
        }
        Err(e) => RadioxideResponse {
            success: false,
            message: format!("Error: {e}"),
            status: None,
        },
    }
}

