mod radio;

use std::path::PathBuf;
use std::sync::Arc;

use radioxide_proto::{DEFAULT_ADDR, RadioCommand, RadioxideMessage, RadioxideResponse};
use radioxide_transports::tcp;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use radio::Radio;
use radio::dummy::DummyRadio;
use radio::yaesu::ft450d::Ft450d;
use radio::yaesu::serial::SerialConfig;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    /// Listen address for the TCP server.
    #[serde(default = "default_addr")]
    addr: String,

    /// Radio backend configuration. If absent, uses dummy backend.
    #[serde(default)]
    radio: Option<RadioConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RadioConfig {
    /// Serial port path (e.g., "/dev/ttyUSB0" on Linux, "/dev/cu.usbserial-XXX" on macOS, "COM3" on Windows).
    serial: String,

    /// Baud rate for the serial port.
    #[serde(default = "default_baud")]
    baud: u32,
}

fn default_addr() -> String {
    DEFAULT_ADDR.to_string()
}

fn default_baud() -> u32 {
    9600
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: default_addr(),
            radio: None,
        }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("radioxide")
        .join("config.json")
}

fn load_config() -> Config {
    let path = config_path();

    if !path.exists() {
        info!("No config file found at {}", path.display());
        info!("Using defaults. Create {} to configure.", path.display());

        // Create the config directory and write a default config
        if let Some(parent) = path.parent()
            && std::fs::create_dir_all(parent).is_ok()
        {
            let default = Config::default();
            if let Ok(json) = serde_json::to_string_pretty(&default) {
                let _ = std::fs::write(&path, json);
                info!("Wrote default config to {}", path.display());
            }
        }

        return Config::default();
    }

    match std::fs::read_to_string(&path) {
        Ok(contents) => match serde_json::from_str(&contents) {
            Ok(config) => {
                info!("Loaded config from {}", path.display());
                config
            }
            Err(e) => {
                error!("Error parsing {}: {e}", path.display());
                std::process::exit(1);
            }
        },
        Err(e) => {
            error!("Error reading {}: {e}", path.display());
            std::process::exit(1);
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = load_config();

    let radio: Arc<dyn Radio> = if let Some(ref rc) = config.radio {
        info!("Opening serial port {} at {} baud...", rc.serial, rc.baud);
        let serial_config = SerialConfig::new(rc.serial.clone(), rc.baud);
        match Ft450d::new(serial_config) {
            Ok(r) => Arc::new(r),
            Err(e) => {
                error!("Failed to open radio: {e}");
                std::process::exit(1);
            }
        }
    } else {
        info!("No radio configured, using dummy backend");
        Arc::new(DummyRadio::new())
    };

    info!("Radioxide daemon starting...");

    let tcp_radio = radio.clone();
    let tcp_handler = move |msg: RadioxideMessage| {
        let radio = tcp_radio.clone();
        async move { dispatch(radio, msg).await }
    };

    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install signal handler");
        info!("Shutdown signal received, stopping...");
    };

    #[cfg(target_os = "linux")]
    {
        use radioxide_transports::dbus;

        let dbus_radio = radio.clone();
        let dbus_handler = move |msg: RadioxideMessage| {
            let radio = dbus_radio.clone();
            async move { dispatch(radio, msg).await }
        };

        tokio::select! {
            result = tcp::start_server(&config.addr, tcp_handler) => {
                if let Err(e) = result {
                    error!("TCP server error: {e}");
                }
            }
            result = dbus::start_dbus_service(dbus_handler) => {
                if let Err(e) = result {
                    error!("D-Bus service error: {e}");
                }
            }
            _ = shutdown => {}
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        tokio::select! {
            result = tcp::start_server(&config.addr, tcp_handler) => {
                if let Err(e) = result {
                    error!("TCP server error: {e}");
                }
            }
            _ = shutdown => {}
        }
    }

    info!("Radioxide daemon stopped.");
}

pub(crate) async fn dispatch(radio: Arc<dyn Radio>, msg: RadioxideMessage) -> RadioxideResponse {
    info!("Received: {:?}", msg.command);

    // GetStatus is handled separately to avoid a redundant second get_status() call.
    if matches!(msg.command, RadioCommand::GetStatus) {
        return match radio.get_status().await {
            Ok(status) => RadioxideResponse {
                success: true,
                message: "Radio status".into(),
                status: Some(status),
            },
            Err(e) => RadioxideResponse {
                success: false,
                message: format!("Error: {e}"),
                status: None,
            },
        };
    }

    let result: Result<String, radio::BackendError> = match msg.command {
        RadioCommand::SetFrequency(hz) => radio
            .set_frequency(hz)
            .await
            .map(|_| "Frequency set".into()),
        RadioCommand::GetFrequency => radio
            .get_frequency()
            .await
            .map(|hz| format!("Frequency: {hz} Hz")),
        RadioCommand::SetBand(band) => radio
            .set_band(band)
            .await
            .map(|_| format!("Band set to {band}")),
        RadioCommand::GetBand => radio.get_band().await.map(|b| format!("Band: {b}")),
        RadioCommand::SetMode(mode) => radio
            .set_mode(mode)
            .await
            .map(|_| format!("Mode set to {mode}")),
        RadioCommand::GetMode => radio.get_mode().await.map(|m| format!("Mode: {m}")),
        RadioCommand::Tune => radio.tune().await.map(|_| "Tuning started".into()),
        RadioCommand::PttOn => radio.set_ptt(true).await.map(|_| "PTT on".into()),
        RadioCommand::PttOff => radio.set_ptt(false).await.map(|_| "PTT off".into()),
        RadioCommand::SetPower(pct) => radio
            .set_power(pct)
            .await
            .map(|_| format!("Power set to {pct}%")),
        RadioCommand::GetPower => radio.get_power().await.map(|p| format!("Power: {p}%")),
        RadioCommand::SetVolume(pct) => radio
            .set_volume(pct)
            .await
            .map(|_| format!("Volume set to {pct}%")),
        RadioCommand::GetVolume => radio.get_volume().await.map(|v| format!("Volume: {v}%")),
        RadioCommand::SetAgc(agc) => radio
            .set_agc(agc)
            .await
            .map(|_| format!("AGC set to {agc}")),
        RadioCommand::GetAgc => radio.get_agc().await.map(|a| format!("AGC: {a}")),
        RadioCommand::SetVfo(vfo) => radio
            .set_vfo(vfo)
            .await
            .map(|_| format!("VFO {vfo} selected")),
        RadioCommand::GetVfo => radio.get_vfo().await.map(|v| format!("VFO: {v}")),
        RadioCommand::GetStatus => unreachable!(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use radioxide_proto::{Agc, Band, Mode, RadioCommand, Vfo};

    fn make_radio() -> Arc<dyn Radio> {
        Arc::new(DummyRadio::new())
    }

    fn msg(command: RadioCommand) -> RadioxideMessage {
        RadioxideMessage { command }
    }

    #[tokio::test]
    async fn dispatch_get_status() {
        let radio = make_radio();
        let resp = dispatch(radio, msg(RadioCommand::GetStatus)).await;
        assert!(resp.success);
        let status = resp.status.unwrap();
        assert_eq!(status.frequency_hz, 14_200_000);
        assert_eq!(status.band, Band::Band20m);
    }

    #[tokio::test]
    async fn dispatch_set_frequency() {
        let radio = make_radio();
        let resp = dispatch(radio.clone(), msg(RadioCommand::SetFrequency(7_074_000))).await;
        assert!(resp.success);
        assert!(resp.message.contains("Frequency set"));
        assert_eq!(resp.status.unwrap().frequency_hz, 7_074_000);
    }

    #[tokio::test]
    async fn dispatch_set_frequency_invalid() {
        let radio = make_radio();
        let resp = dispatch(radio, msg(RadioCommand::SetFrequency(0))).await;
        assert!(!resp.success);
    }

    #[tokio::test]
    async fn dispatch_set_band() {
        let radio = make_radio();
        let resp = dispatch(radio, msg(RadioCommand::SetBand(Band::Band40m))).await;
        assert!(resp.success);
        let status = resp.status.unwrap();
        assert_eq!(status.band, Band::Band40m);
        assert_eq!(status.frequency_hz, 7_074_000);
    }

    #[tokio::test]
    async fn dispatch_set_mode() {
        let radio = make_radio();
        let resp = dispatch(radio, msg(RadioCommand::SetMode(Mode::CW))).await;
        assert!(resp.success);
        assert_eq!(resp.status.unwrap().mode, Mode::CW);
    }

    #[tokio::test]
    async fn dispatch_set_power() {
        let radio = make_radio();
        let resp = dispatch(radio, msg(RadioCommand::SetPower(75))).await;
        assert!(resp.success);
        assert_eq!(resp.status.unwrap().power, 75);
    }

    #[tokio::test]
    async fn dispatch_set_volume() {
        let radio = make_radio();
        let resp = dispatch(radio, msg(RadioCommand::SetVolume(30))).await;
        assert!(resp.success);
        assert_eq!(resp.status.unwrap().volume, 30);
    }

    #[tokio::test]
    async fn dispatch_set_agc() {
        let radio = make_radio();
        let resp = dispatch(radio, msg(RadioCommand::SetAgc(Agc::Slow))).await;
        assert!(resp.success);
        assert_eq!(resp.status.unwrap().agc, Agc::Slow);
    }

    #[tokio::test]
    async fn dispatch_get_frequency() {
        let radio = make_radio();
        let resp = dispatch(radio, msg(RadioCommand::GetFrequency)).await;
        assert!(resp.success);
        assert!(resp.message.contains("14200000"));
    }

    #[tokio::test]
    async fn dispatch_get_band() {
        let radio = make_radio();
        let resp = dispatch(radio, msg(RadioCommand::GetBand)).await;
        assert!(resp.success);
        assert!(resp.message.contains("20m"));
    }

    #[tokio::test]
    async fn dispatch_get_mode() {
        let radio = make_radio();
        let resp = dispatch(radio, msg(RadioCommand::GetMode)).await;
        assert!(resp.success);
        assert!(resp.message.contains("USB"));
    }

    #[tokio::test]
    async fn dispatch_ptt_on_off() {
        let radio = make_radio();
        let resp = dispatch(radio.clone(), msg(RadioCommand::PttOn)).await;
        assert!(resp.success);
        assert!(resp.status.unwrap().ptt);

        let resp = dispatch(radio, msg(RadioCommand::PttOff)).await;
        assert!(resp.success);
        assert!(!resp.status.unwrap().ptt);
    }

    #[tokio::test]
    async fn dispatch_tune() {
        let radio = make_radio();
        let resp = dispatch(radio, msg(RadioCommand::Tune)).await;
        assert!(resp.success);
        assert!(resp.status.unwrap().tuning);
    }

    #[tokio::test]
    async fn dispatch_set_vfo() {
        let radio = make_radio();
        let resp = dispatch(radio.clone(), msg(RadioCommand::SetVfo(Vfo::B))).await;
        assert!(resp.success);
        assert!(resp.message.contains("VFO B"));
        assert_eq!(resp.status.unwrap().vfo, Vfo::B);

        let resp = dispatch(radio, msg(RadioCommand::SetVfo(Vfo::A))).await;
        assert!(resp.success);
        assert_eq!(resp.status.unwrap().vfo, Vfo::A);
    }

    #[tokio::test]
    async fn dispatch_get_vfo() {
        let radio = make_radio();
        let resp = dispatch(radio, msg(RadioCommand::GetVfo)).await;
        assert!(resp.success);
        assert!(resp.message.contains("VFO: A"));
    }
}
