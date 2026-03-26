use async_trait::async_trait;
use radioxide_proto::{Agc, Band, Mode, RadioStatus, Vfo};

use super::cat::{
    agc_to_cat, band_to_cat, cat_to_agc, cat_to_mode, cat_to_vfo, freq_to_band, mode_to_cat,
    vfo_to_cat, CatCommand,
};
use super::serial::{CatPort, SerialConfig};
use crate::radio::{BackendError, Radio, Result};

pub struct Ft450d {
    port: CatPort,
}

impl Ft450d {
    pub fn new(config: SerialConfig) -> Result<Self> {
        let port = CatPort::open(&config)?;
        Ok(Self { port })
    }
}

#[async_trait]
impl Radio for Ft450d {
    async fn set_frequency(&self, hz: u64) -> Result<()> {
        if !(30_000..=60_000_000).contains(&hz) {
            return Err(BackendError::InvalidParameter(format!(
                "frequency {hz} Hz out of range (30000-60000000)"
            )));
        }
        let cmd = CatCommand::set("FA", &format!("{hz:08}"));
        self.port.send(&cmd).await
    }

    async fn get_frequency(&self) -> Result<u64> {
        let resp = self.port.execute(&CatCommand::read("FA")).await?;
        resp.body
            .parse::<u64>()
            .map_err(|_| BackendError::Protocol(format!("bad frequency: {:?}", resp.body)))
    }

    async fn set_band(&self, band: Band) -> Result<()> {
        let p1 = band_to_cat(band)?;
        // BS command takes P1 twice (for VFO-A and VFO-B band select)
        let cmd = CatCommand::set("BS", &format!("{p1}{p1}"));
        self.port.send(&cmd).await
    }

    async fn get_band(&self) -> Result<Band> {
        let hz = self.get_frequency().await?;
        freq_to_band(hz).ok_or_else(|| {
            BackendError::Protocol(format!("cannot determine band for {hz} Hz"))
        })
    }

    async fn set_mode(&self, mode: Mode) -> Result<()> {
        let p2 = mode_to_cat(mode)?;
        // MD command: P1=0 (fixed), P2=mode char
        let cmd = CatCommand::set("MD", &format!("0{p2}"));
        self.port.send(&cmd).await
    }

    async fn get_mode(&self) -> Result<Mode> {
        let resp = self.port.execute(&CatCommand::read("MD")).await?;
        // Response body: "0{P2}"
        let p2 = resp
            .body
            .chars()
            .nth(1)
            .ok_or_else(|| BackendError::Protocol(format!("short MD response: {:?}", resp.body)))?;
        cat_to_mode(p2)
    }

    async fn tune(&self) -> Result<()> {
        // AC command: P1=0, P2=0, P3=2 (start tuning)
        let cmd = CatCommand::set("AC", "002");
        self.port.send(&cmd).await
    }

    async fn set_ptt(&self, on: bool) -> Result<()> {
        // TX1 = CAT PTT on, TX0 = off
        let p1 = if on { "1" } else { "0" };
        let cmd = CatCommand::set("TX", p1);
        self.port.send(&cmd).await
    }

    async fn get_ptt(&self) -> Result<bool> {
        let resp = self.port.execute(&CatCommand::read("TX")).await?;
        // TX0 = not transmitting, TX1 or TX2 = transmitting
        Ok(resp.body != "0")
    }

    async fn set_power(&self, percent: u8) -> Result<()> {
        // Map 0-100% to 0-255
        let raw = (percent.min(100) as u16) * 255 / 100;
        let cmd = CatCommand::set("PC", &format!("{raw:03}"));
        self.port.send(&cmd).await
    }

    async fn get_power(&self) -> Result<u8> {
        let resp = self.port.execute(&CatCommand::read("PC")).await?;
        let raw: u16 = resp
            .body
            .parse()
            .map_err(|_| BackendError::Protocol(format!("bad power: {:?}", resp.body)))?;
        // Map 0-255 back to 0-100%
        Ok(((raw * 100) / 255) as u8)
    }

    async fn set_volume(&self, percent: u8) -> Result<()> {
        // AG command: P1=0 (fixed), P2=000-255
        let raw = (percent.min(100) as u16) * 255 / 100;
        let cmd = CatCommand::set("AG", &format!("0{raw:03}"));
        self.port.send(&cmd).await
    }

    async fn get_volume(&self) -> Result<u8> {
        let resp = self.port.execute(&CatCommand::read("AG")).await?;
        // Response body: "0{3 digits}"
        let raw: u16 = resp.body[1..]
            .parse()
            .map_err(|_| BackendError::Protocol(format!("bad volume: {:?}", resp.body)))?;
        Ok(((raw * 100) / 255) as u8)
    }

    async fn set_agc(&self, agc: Agc) -> Result<()> {
        let p2 = agc_to_cat(agc);
        // GT command: P1=0 (fixed), P2=agc char
        let cmd = CatCommand::set("GT", &format!("0{p2}"));
        self.port.send(&cmd).await
    }

    async fn get_agc(&self) -> Result<Agc> {
        let resp = self.port.execute(&CatCommand::read("GT")).await?;
        let p2 = resp
            .body
            .chars()
            .nth(1)
            .ok_or_else(|| BackendError::Protocol(format!("short GT response: {:?}", resp.body)))?;
        cat_to_agc(p2)
    }

    async fn set_vfo(&self, vfo: Vfo) -> Result<()> {
        // VS command: VS{P1}; where P1=0 (VFO-A) or P1=1 (VFO-B). No response body.
        let p1 = vfo_to_cat(vfo);
        let cmd = CatCommand::set("VS", &p1.to_string());
        self.port.send(&cmd).await
    }

    async fn get_vfo(&self) -> Result<Vfo> {
        let resp = self.port.execute(&CatCommand::read("VS")).await?;
        let p1 = resp
            .body
            .chars()
            .next()
            .ok_or_else(|| BackendError::Protocol("empty VS response".into()))?;
        cat_to_vfo(p1)
    }

    async fn get_status(&self) -> Result<RadioStatus> {
        let freq = self.get_frequency().await?;
        let mode = self.get_mode().await?;
        let power = self.get_power().await?;
        let volume = self.get_volume().await?;
        let agc = self.get_agc().await?;
        let ptt = self.get_ptt().await?;
        let vfo = self.get_vfo().await?;
        let band = freq_to_band(freq).unwrap_or(Band::Band20m);

        Ok(RadioStatus {
            frequency_hz: freq,
            band,
            mode,
            power,
            volume,
            agc,
            vfo,
            ptt,
            tuning: false, // No direct CAT command to read tuning state
        })
    }
}
