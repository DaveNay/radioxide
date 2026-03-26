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

/// VFO (Variable Frequency Oscillator) selector.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vfo {
    A,
    B,
}

impl std::fmt::Display for Vfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Vfo::A => write!(f, "A"),
            Vfo::B => write!(f, "B"),
        }
    }
}

impl std::str::FromStr for Vfo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "A" => Ok(Vfo::A),
            "B" => Ok(Vfo::B),
            other => Err(format!("unknown VFO: {other}")),
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
    /// Select active VFO.
    SetVfo(Vfo),
    /// Get active VFO.
    GetVfo,
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
    pub vfo: Vfo,
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
            vfo: Vfo::A,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_display() {
        let expected = [
            (Band::Band160m, "160m"),
            (Band::Band80m, "80m"),
            (Band::Band60m, "60m"),
            (Band::Band40m, "40m"),
            (Band::Band30m, "30m"),
            (Band::Band20m, "20m"),
            (Band::Band17m, "17m"),
            (Band::Band15m, "15m"),
            (Band::Band12m, "12m"),
            (Band::Band10m, "10m"),
            (Band::Band6m, "6m"),
            (Band::Band2m, "2m"),
            (Band::Band70cm, "70cm"),
        ];
        for (band, s) in expected {
            assert_eq!(band.to_string(), s);
        }
    }

    #[test]
    fn band_from_str() {
        for s in ["160m", "80m", "60m", "40m", "30m", "20m", "17m", "15m", "12m", "10m", "6m", "2m", "70cm"] {
            let band: Band = s.parse().unwrap();
            assert_eq!(band.to_string().to_lowercase(), s.to_lowercase());
        }
        // Case insensitivity
        assert_eq!("20M".parse::<Band>().unwrap(), Band::Band20m);
        assert_eq!("70CM".parse::<Band>().unwrap(), Band::Band70cm);
    }

    #[test]
    fn band_from_str_invalid() {
        assert!("99m".parse::<Band>().is_err());
        assert!("".parse::<Band>().is_err());
        assert!("HF".parse::<Band>().is_err());
    }

    #[test]
    fn mode_display() {
        let expected = [
            (Mode::LSB, "LSB"),
            (Mode::USB, "USB"),
            (Mode::CW, "CW"),
            (Mode::AM, "AM"),
            (Mode::FM, "FM"),
            (Mode::Digital, "DIG"),
            (Mode::CWR, "CW-R"),
            (Mode::DigitalR, "DIG-R"),
        ];
        for (mode, s) in expected {
            assert_eq!(mode.to_string(), s);
        }
    }

    #[test]
    fn mode_from_str() {
        assert_eq!("LSB".parse::<Mode>().unwrap(), Mode::LSB);
        assert_eq!("USB".parse::<Mode>().unwrap(), Mode::USB);
        assert_eq!("CW".parse::<Mode>().unwrap(), Mode::CW);
        assert_eq!("AM".parse::<Mode>().unwrap(), Mode::AM);
        assert_eq!("FM".parse::<Mode>().unwrap(), Mode::FM);
        assert_eq!("DIG".parse::<Mode>().unwrap(), Mode::Digital);
        assert_eq!("DIGITAL".parse::<Mode>().unwrap(), Mode::Digital);
        assert_eq!("CW-R".parse::<Mode>().unwrap(), Mode::CWR);
        assert_eq!("CWR".parse::<Mode>().unwrap(), Mode::CWR);
        assert_eq!("DIG-R".parse::<Mode>().unwrap(), Mode::DigitalR);
        assert_eq!("DIGITALR".parse::<Mode>().unwrap(), Mode::DigitalR);
        assert_eq!("DIGITAL-R".parse::<Mode>().unwrap(), Mode::DigitalR);
        // Case insensitivity
        assert_eq!("lsb".parse::<Mode>().unwrap(), Mode::LSB);
        assert_eq!("dig".parse::<Mode>().unwrap(), Mode::Digital);
    }

    #[test]
    fn mode_from_str_invalid() {
        assert!("SSB".parse::<Mode>().is_err());
        assert!("".parse::<Mode>().is_err());
    }

    #[test]
    fn agc_display() {
        assert_eq!(Agc::Off.to_string(), "OFF");
        assert_eq!(Agc::Fast.to_string(), "FAST");
        assert_eq!(Agc::Medium.to_string(), "MED");
        assert_eq!(Agc::Slow.to_string(), "SLOW");
    }

    #[test]
    fn agc_from_str() {
        assert_eq!("OFF".parse::<Agc>().unwrap(), Agc::Off);
        assert_eq!("FAST".parse::<Agc>().unwrap(), Agc::Fast);
        assert_eq!("MED".parse::<Agc>().unwrap(), Agc::Medium);
        assert_eq!("MEDIUM".parse::<Agc>().unwrap(), Agc::Medium);
        assert_eq!("SLOW".parse::<Agc>().unwrap(), Agc::Slow);
        // Case insensitivity
        assert_eq!("off".parse::<Agc>().unwrap(), Agc::Off);
        assert_eq!("slow".parse::<Agc>().unwrap(), Agc::Slow);
    }

    #[test]
    fn agc_from_str_invalid() {
        assert!("TURBO".parse::<Agc>().is_err());
        assert!("".parse::<Agc>().is_err());
    }

    #[test]
    fn radio_status_default() {
        let s = RadioStatus::default();
        assert_eq!(s.frequency_hz, 14_200_000);
        assert_eq!(s.band, Band::Band20m);
        assert_eq!(s.mode, Mode::USB);
        assert_eq!(s.power, 50);
        assert_eq!(s.volume, 50);
        assert_eq!(s.agc, Agc::Medium);
        assert!(!s.ptt);
        assert!(!s.tuning);
    }

    #[test]
    fn message_json_roundtrip() {
        let msg = RadioxideMessage {
            command: RadioCommand::SetFrequency(14_074_000),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: RadioxideMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.command, msg.command);
    }

    #[test]
    fn response_json_roundtrip() {
        let resp = RadioxideResponse {
            success: true,
            message: "OK".into(),
            status: Some(RadioStatus::default()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: RadioxideResponse = serde_json::from_str(&json).unwrap();
        assert!(back.success);
        assert_eq!(back.status.unwrap().frequency_hz, 14_200_000);
    }

    #[test]
    fn command_variants_serialize() {
        let commands = vec![
            RadioCommand::SetFrequency(7_074_000),
            RadioCommand::GetFrequency,
            RadioCommand::SetBand(Band::Band40m),
            RadioCommand::GetBand,
            RadioCommand::SetMode(Mode::CW),
            RadioCommand::GetMode,
            RadioCommand::Tune,
            RadioCommand::PttOn,
            RadioCommand::PttOff,
            RadioCommand::SetPower(75),
            RadioCommand::GetPower,
            RadioCommand::SetVolume(30),
            RadioCommand::GetVolume,
            RadioCommand::SetAgc(Agc::Fast),
            RadioCommand::GetAgc,
            RadioCommand::SetVfo(Vfo::A),
            RadioCommand::SetVfo(Vfo::B),
            RadioCommand::GetVfo,
            RadioCommand::GetStatus,
        ];
        for cmd in commands {
            let json = serde_json::to_string(&cmd).unwrap();
            let back: RadioCommand = serde_json::from_str(&json).unwrap();
            assert_eq!(back, cmd);
        }
    }

    #[test]
    fn vfo_display() {
        assert_eq!(Vfo::A.to_string(), "A");
        assert_eq!(Vfo::B.to_string(), "B");
    }

    #[test]
    fn vfo_from_str() {
        assert_eq!("A".parse::<Vfo>().unwrap(), Vfo::A);
        assert_eq!("B".parse::<Vfo>().unwrap(), Vfo::B);
        assert_eq!("a".parse::<Vfo>().unwrap(), Vfo::A);
        assert_eq!("b".parse::<Vfo>().unwrap(), Vfo::B);
        assert!("C".parse::<Vfo>().is_err());
        assert!("".parse::<Vfo>().is_err());
    }

    #[test]
    fn radio_status_default_vfo() {
        let status = RadioStatus::default();
        assert_eq!(status.vfo, Vfo::A);
    }
}
