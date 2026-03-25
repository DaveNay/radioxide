use std::sync::Arc;

use async_trait::async_trait;
use radioxide_proto::{Agc, Band, Mode, RadioStatus};
use tokio::sync::Mutex;

use super::{BackendError, Radio, Result};

pub struct DummyRadio {
    state: Arc<Mutex<RadioStatus>>,
}

impl DummyRadio {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RadioStatus::default())),
        }
    }
}

#[async_trait]
impl Radio for DummyRadio {
    async fn set_frequency(&self, hz: u64) -> Result<()> {
        if !(30_000..=60_000_000).contains(&hz) {
            return Err(BackendError::InvalidParameter(format!(
                "frequency {hz} Hz out of range (30000-60000000)"
            )));
        }
        self.state.lock().await.frequency_hz = hz;
        Ok(())
    }

    async fn get_frequency(&self) -> Result<u64> {
        Ok(self.state.lock().await.frequency_hz)
    }

    async fn set_band(&self, band: Band) -> Result<()> {
        let mut state = self.state.lock().await;
        state.band = band;
        state.frequency_hz = default_frequency_for_band(band);
        Ok(())
    }

    async fn get_band(&self) -> Result<Band> {
        Ok(self.state.lock().await.band)
    }

    async fn set_mode(&self, mode: Mode) -> Result<()> {
        self.state.lock().await.mode = mode;
        Ok(())
    }

    async fn get_mode(&self) -> Result<Mode> {
        Ok(self.state.lock().await.mode)
    }

    async fn tune(&self) -> Result<()> {
        self.state.lock().await.tuning = true;
        // Simulate tuner completing after 2 seconds.
        let state = self.state.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            state.lock().await.tuning = false;
        });
        Ok(())
    }

    async fn set_ptt(&self, on: bool) -> Result<()> {
        self.state.lock().await.ptt = on;
        Ok(())
    }

    async fn get_ptt(&self) -> Result<bool> {
        Ok(self.state.lock().await.ptt)
    }

    async fn set_power(&self, percent: u8) -> Result<()> {
        self.state.lock().await.power = percent.min(100);
        Ok(())
    }

    async fn get_power(&self) -> Result<u8> {
        Ok(self.state.lock().await.power)
    }

    async fn set_volume(&self, percent: u8) -> Result<()> {
        self.state.lock().await.volume = percent.min(100);
        Ok(())
    }

    async fn get_volume(&self) -> Result<u8> {
        Ok(self.state.lock().await.volume)
    }

    async fn set_agc(&self, agc: Agc) -> Result<()> {
        self.state.lock().await.agc = agc;
        Ok(())
    }

    async fn get_agc(&self) -> Result<Agc> {
        Ok(self.state.lock().await.agc)
    }

    async fn get_status(&self) -> Result<RadioStatus> {
        Ok(self.state.lock().await.clone())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_frequency_roundtrip() {
        let radio = DummyRadio::new();
        radio.set_frequency(7_074_000).await.unwrap();
        assert_eq!(radio.get_frequency().await.unwrap(), 7_074_000);
    }

    #[tokio::test]
    async fn test_mode_roundtrip() {
        let radio = DummyRadio::new();
        radio.set_mode(Mode::CW).await.unwrap();
        assert_eq!(radio.get_mode().await.unwrap(), Mode::CW);
    }

    #[tokio::test]
    async fn test_band_sets_default_frequency() {
        let radio = DummyRadio::new();
        radio.set_band(Band::Band40m).await.unwrap();
        assert_eq!(radio.get_band().await.unwrap(), Band::Band40m);
        assert_eq!(radio.get_frequency().await.unwrap(), 7_074_000);
    }

    #[tokio::test]
    async fn test_ptt_roundtrip() {
        let radio = DummyRadio::new();
        assert!(!radio.get_ptt().await.unwrap());
        radio.set_ptt(true).await.unwrap();
        assert!(radio.get_ptt().await.unwrap());
    }

    #[tokio::test]
    async fn test_power_clamped() {
        let radio = DummyRadio::new();
        radio.set_power(200).await.unwrap();
        assert_eq!(radio.get_power().await.unwrap(), 100);
    }

    #[tokio::test]
    async fn test_get_status() {
        let radio = DummyRadio::new();
        radio.set_frequency(21_074_000).await.unwrap();
        radio.set_mode(Mode::USB).await.unwrap();
        let status = radio.get_status().await.unwrap();
        assert_eq!(status.frequency_hz, 21_074_000);
        assert_eq!(status.mode, Mode::USB);
    }

    #[tokio::test]
    async fn test_frequency_out_of_range_low() {
        let radio = DummyRadio::new();
        assert!(radio.set_frequency(100).await.is_err());
    }

    #[tokio::test]
    async fn test_frequency_out_of_range_high() {
        let radio = DummyRadio::new();
        assert!(radio.set_frequency(100_000_000).await.is_err());
    }

    #[tokio::test]
    async fn test_volume_clamped() {
        let radio = DummyRadio::new();
        radio.set_volume(200).await.unwrap();
        assert_eq!(radio.get_volume().await.unwrap(), 100);
    }

    #[tokio::test]
    async fn test_agc_roundtrip() {
        let radio = DummyRadio::new();
        for agc in [Agc::Off, Agc::Fast, Agc::Medium, Agc::Slow] {
            radio.set_agc(agc).await.unwrap();
            assert_eq!(radio.get_agc().await.unwrap(), agc);
        }
    }

    #[tokio::test]
    async fn test_band_sets_correct_default_freqs() {
        let radio = DummyRadio::new();
        let expected = [
            (Band::Band160m, 1_840_000),
            (Band::Band80m, 3_573_000),
            (Band::Band60m, 5_357_000),
            (Band::Band40m, 7_074_000),
            (Band::Band30m, 10_136_000),
            (Band::Band20m, 14_074_000),
            (Band::Band17m, 18_100_000),
            (Band::Band15m, 21_074_000),
            (Band::Band12m, 24_915_000),
            (Band::Band10m, 28_074_000),
            (Band::Band6m, 50_313_000),
            (Band::Band2m, 144_200_000),
            (Band::Band70cm, 432_200_000),
        ];
        for (band, freq) in expected {
            radio.set_band(band).await.unwrap();
            assert_eq!(radio.get_frequency().await.unwrap(), freq, "wrong default freq for {band}");
        }
    }

    #[tokio::test]
    async fn test_tune_sets_tuning_flag() {
        let radio = DummyRadio::new();
        radio.tune().await.unwrap();
        let status = radio.get_status().await.unwrap();
        assert!(status.tuning);
    }
}
