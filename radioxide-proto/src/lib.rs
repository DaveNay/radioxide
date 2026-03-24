use serde::{Deserialize, Serialize};

pub const DEFAULT_PORT: u16 = 7600;
pub const DEFAULT_ADDR: &str = "127.0.0.1:7600";

/// Amateur radio frequency band.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// 160 meters (1.8–2.0 MHz)
    Band160m,
    /// 80 meters (3.5–4.0 MHz)
    Band80m,
    /// 60 meters (5.3–5.4 MHz)
    Band60m,
    /// 40 meters (7.0–7.3 MHz)
    Band40m,
    /// 30 meters (10.1–10.15 MHz)
    Band30m,
    /// 20 meters (14.0–14.35 MHz)
    Band20m,
    /// 17 meters (18.068–18.168 MHz)
    Band17m,
    /// 15 meters (21.0–21.45 MHz)
    Band15m,
    /// 12 meters (24.89–24.99 MHz)
    Band12m,
    /// 10 meters (28.0–29.7 MHz)
    Band10m,
    /// 6 meters (50–54 MHz)
    Band6m,
    /// 2 meters (144–148 MHz)
    Band2m,
    /// 70 centimeters (420–450 MHz)
    Band70cm,
}

impl std::fmt::Display for Band {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Band::Band160m => write!(f, "160m"),
            Band::Band80m => write!(f, "80m"),
            Band::Band60m => write!(f, "60m"),
            Band::Band40m => write!(f, "40m"),
            Band::Band30m => write!(f, "30m"),
            Band::Band20m => write!(f, "20m"),
            Band::Band17m => write!(f, "17m"),
            Band::Band15m => write!(f, "15m"),
            Band::Band12m => write!(f, "12m"),
            Band::Band10m => write!(f, "10m"),
            Band::Band6m => write!(f, "6m"),
            Band::Band2m => write!(f, "2m"),
            Band::Band70cm => write!(f, "70cm"),
        }
    }
}

impl std::str::FromStr for Band {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "160m" => Ok(Band::Band160m),
            "80m" => Ok(Band::Band80m),
            "60m" => Ok(Band::Band60m),
            "40m" => Ok(Band::Band40m),
            "30m" => Ok(Band::Band30m),
            "20m" => Ok(Band::Band20m),
            "17m" => Ok(Band::Band17m),
            "15m" => Ok(Band::Band15m),
            "12m" => Ok(Band::Band12m),
            "10m" => Ok(Band::Band10m),
            "6m" => Ok(Band::Band6m),
            "2m" => Ok(Band::Band2m),
            "70cm" => Ok(Band::Band70cm),
            other => Err(format!("unknown band: {other}")),
        }
    }
}

/// Operating mode.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Lower sideband
    LSB,
    /// Upper sideband
    USB,
    /// Continuous wave
    CW,
    /// Amplitude modulation
    AM,
    /// Frequency modulation
    FM,
    /// Digital (e.g., FT8, RTTY)
    Digital,
    /// CW reverse
    CWR,
    /// Digital reverse
    DigitalR,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::LSB => write!(f, "LSB"),
            Mode::USB => write!(f, "USB"),
            Mode::CW => write!(f, "CW"),
            Mode::AM => write!(f, "AM"),
            Mode::FM => write!(f, "FM"),
            Mode::Digital => write!(f, "DIG"),
            Mode::CWR => write!(f, "CW-R"),
            Mode::DigitalR => write!(f, "DIG-R"),
        }
    }
}

impl std::str::FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "LSB" => Ok(Mode::LSB),
            "USB" => Ok(Mode::USB),
            "CW" => Ok(Mode::CW),
            "AM" => Ok(Mode::AM),
            "FM" => Ok(Mode::FM),
            "DIG" | "DIGITAL" => Ok(Mode::Digital),
            "CW-R" | "CWR" => Ok(Mode::CWR),
            "DIG-R" | "DIGITALR" | "DIGITAL-R" => Ok(Mode::DigitalR),
            other => Err(format!("unknown mode: {other}")),
        }
    }
}

/// AGC (Automatic Gain Control) setting.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agc {
    Off,
    Fast,
    Medium,
    Slow,
}

impl std::fmt::Display for Agc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Agc::Off => write!(f, "OFF"),
            Agc::Fast => write!(f, "FAST"),
            Agc::Medium => write!(f, "MED"),
            Agc::Slow => write!(f, "SLOW"),
        }
    }
}

impl std::str::FromStr for Agc {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "OFF" => Ok(Agc::Off),
            "FAST" => Ok(Agc::Fast),
            "MED" | "MEDIUM" => Ok(Agc::Medium),
            "SLOW" => Ok(Agc::Slow),
            other => Err(format!("unknown AGC setting: {other}")),
        }
    }
}

/// Command sent from a client to the daemon.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RadioCommand {
    /// Set the VFO frequency in Hz.
    SetFrequency(u64),
    /// Get the current VFO frequency.
    GetFrequency,
    /// Select a band (moves frequency to that band's default).
    SetBand(Band),
    /// Get the current band.
    GetBand,
    /// Set the operating mode.
    SetMode(Mode),
    /// Get the current operating mode.
    GetMode,
    /// Trigger the antenna tuner.
    Tune,
    /// Key the transmitter (PTT on).
    PttOn,
    /// Unkey the transmitter (PTT off).
    PttOff,
    /// Set RF power output (0–100 as a percentage).
    SetPower(u8),
    /// Get current RF power level.
    GetPower,
    /// Set AF (audio) volume (0–100 as a percentage).
    SetVolume(u8),
    /// Get current AF volume.
    GetVolume,
    /// Set AGC mode.
    SetAgc(Agc),
    /// Get current AGC mode.
    GetAgc,
    /// Request full radio status.
    GetStatus,
}

/// Current state of the radio, returned by GetStatus.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RadioStatus {
    pub frequency_hz: u64,
    pub band: Band,
    pub mode: Mode,
    pub power: u8,
    pub volume: u8,
    pub agc: Agc,
    pub ptt: bool,
    pub tuning: bool,
}

impl Default for RadioStatus {
    fn default() -> Self {
        Self {
            frequency_hz: 14_200_000,
            band: Band::Band20m,
            mode: Mode::USB,
            power: 50,
            volume: 50,
            agc: Agc::Medium,
            ptt: false,
            tuning: false,
        }
    }
}

/// Message envelope sent over the wire.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RadioxideMessage {
    pub command: RadioCommand,
}

/// Response from the daemon.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RadioxideResponse {
    pub success: bool,
    pub message: String,
    /// Full radio status, included when relevant (e.g., GetStatus, after Set* commands).
    pub status: Option<RadioStatus>,
}
