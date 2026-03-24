use std::sync::{Arc, Mutex};

use radioxide_proto::{
    Band, RadioCommand, RadioStatus, RadioxideMessage, RadioxideResponse, DEFAULT_ADDR,
};
use radioxide_transports::tcp;

/// Shared radio state — will eventually be backed by real hardware.
struct RadioState {
    status: RadioStatus,
}

impl RadioState {
    fn new() -> Self {
        Self {
            status: RadioStatus::default(),
        }
    }

    fn handle(&mut self, cmd: RadioCommand) -> RadioxideResponse {
        match cmd {
            RadioCommand::SetFrequency(hz) => {
                self.status.frequency_hz = hz;
                self.ok(format!("Frequency set to {} Hz", hz))
            }
            RadioCommand::GetFrequency => {
                self.ok(format!("Frequency: {} Hz", self.status.frequency_hz))
            }
            RadioCommand::SetBand(band) => {
                self.status.band = band;
                self.status.frequency_hz = default_frequency_for_band(band);
                self.ok(format!("Band set to {band}"))
            }
            RadioCommand::GetBand => self.ok(format!("Band: {}", self.status.band)),
            RadioCommand::SetMode(mode) => {
                self.status.mode = mode;
                self.ok(format!("Mode set to {mode}"))
            }
            RadioCommand::GetMode => self.ok(format!("Mode: {}", self.status.mode)),
            RadioCommand::Tune => {
                self.status.tuning = true;
                self.ok("Tuning started".into())
            }
            RadioCommand::PttOn => {
                self.status.ptt = true;
                self.ok("PTT on".into())
            }
            RadioCommand::PttOff => {
                self.status.ptt = false;
                self.ok("PTT off".into())
            }
            RadioCommand::SetPower(pct) => {
                self.status.power = pct.min(100);
                self.ok(format!("Power set to {}%", self.status.power))
            }
            RadioCommand::GetPower => self.ok(format!("Power: {}%", self.status.power)),
            RadioCommand::SetVolume(pct) => {
                self.status.volume = pct.min(100);
                self.ok(format!("Volume set to {}%", self.status.volume))
            }
            RadioCommand::GetVolume => self.ok(format!("Volume: {}%", self.status.volume)),
            RadioCommand::SetAgc(agc) => {
                self.status.agc = agc;
                self.ok(format!("AGC set to {agc}"))
            }
            RadioCommand::GetAgc => self.ok(format!("AGC: {}", self.status.agc)),
            RadioCommand::GetStatus => self.ok("Radio status".into()),
        }
    }

    fn ok(&self, message: String) -> RadioxideResponse {
        RadioxideResponse {
            success: true,
            message,
            status: Some(self.status.clone()),
        }
    }
}

fn default_frequency_for_band(band: Band) -> u64 {
    match band {
        Band::Band160m => 1_840_000,
        Band::Band80m => 3_573_000,
        Band::Band60m => 5_357_000,
        Band::Band40m => 7_074_000,
        Band::Band30m => 10_136_000,
        Band::Band20m => 14_074_000,
        Band::Band17m => 18_100_000,
        Band::Band15m => 21_074_000,
        Band::Band12m => 24_915_000,
        Band::Band10m => 28_074_000,
        Band::Band6m => 50_313_000,
        Band::Band2m => 144_200_000,
        Band::Band70cm => 432_200_000,
    }
}

#[tokio::main]
async fn main() {
    println!("Radioxide daemon starting...");

    let state = Arc::new(Mutex::new(RadioState::new()));

    let handler = move |msg: RadioxideMessage| -> RadioxideResponse {
        let mut state = state.lock().unwrap();
        println!("Received: {:?}", msg.command);
        state.handle(msg.command)
    };

    if let Err(e) = tcp::start_server(DEFAULT_ADDR, handler).await {
        eprintln!("Daemon error: {e}");
    }
}
