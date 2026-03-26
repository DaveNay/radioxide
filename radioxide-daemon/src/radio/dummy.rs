use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use radioxide_proto::{Agc, Band, Mode, RadioStatus, Vfo};
use tokio::sync::Mutex;

use super::{BackendError, Radio, Result};

struct DummyState {
    status: RadioStatus,
    /// Independent per-VFO frequency storage.
    freq_a: u64,
    freq_b: u64,
    /// Last-used frequency per band, restored when switching back to a band.
    band_freq: HashMap<Band, u64>,
}

impl DummyState {
    fn new() -> Self {
        let status = RadioStatus::default();
        Self {
            freq_a: status.frequency_hz,
            freq_b: status.frequency_hz,
            band_freq: HashMap::new(),
            status,
        }
    }

    /// Return the stored frequency for the given VFO.
    fn freq_for(&self, vfo: Vfo) -> u64 {
        match vfo {
            Vfo::A => self.freq_a,
            Vfo::B => self.freq_b,
        }
    }

    /// Store a frequency for the given VFO and keep status.frequency_hz in sync when the
    /// target VFO is the active one.
    fn set_freq_for(&mut self, vfo: Vfo, hz: u64) {
        match vfo {
            Vfo::A => self.freq_a = hz,
            Vfo::B => self.freq_b = hz,
        }
        if vfo == self.status.vfo {
            self.status.frequency_hz = hz;
        }
    }

    /// Remember hz as the last-used frequency for the current band.
    fn remember_band_freq(&mut self, hz: u64) {
        self.band_freq.insert(self.status.band, hz);
    }

    /// Return the remembered frequency for a band, or its default on first visit.
    fn recalled_freq_for_band(&self, band: Band) -> u64 {
        self.band_freq
            .get(&band)
            .copied()
            .unwrap_or_else(|| default_frequency_for_band(band))
    }
}

pub struct DummyRadio {
    state: Arc<Mutex<DummyState>>,
}

impl DummyRadio {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(DummyState::new())),
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
        let mut state = self.state.lock().await;
        let active = state.status.vfo;
        state.set_freq_for(active, hz);
        state.remember_band_freq(hz);
        Ok(())
    }

    async fn get_frequency(&self) -> Result<u64> {
        let state = self.state.lock().await;
        Ok(state.freq_for(state.status.vfo))
    }

    async fn set_band(&self, band: Band) -> Result<()> {
        let mut state = self.state.lock().await;
        state.status.band = band;
        let freq = state.recalled_freq_for_band(band);
        let active = state.status.vfo;
        state.set_freq_for(active, freq);
        Ok(())
    }

    async fn get_band(&self) -> Result<Band> {
        Ok(self.state.lock().await.status.band)
    }

    async fn set_mode(&self, mode: Mode) -> Result<()> {
        self.state.lock().await.status.mode = mode;
        Ok(())
    }

    async fn get_mode(&self) -> Result<Mode> {
        Ok(self.state.lock().await.status.mode)
    }

    async fn tune(&self) -> Result<()> {
        self.state.lock().await.status.tuning = true;
        // Simulate tuner completing after 2 seconds.
        let state = self.state.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            state.lock().await.status.tuning = false;
        });
        Ok(())
    }

    async fn set_ptt(&self, on: bool) -> Result<()> {
        self.state.lock().await.status.ptt = on;
        Ok(())
    }

    async fn get_ptt(&self) -> Result<bool> {
        Ok(self.state.lock().await.status.ptt)
    }

    async fn set_power(&self, percent: u8) -> Result<()> {
        self.state.lock().await.status.power = percent.min(100);
        Ok(())
    }

    async fn get_power(&self) -> Result<u8> {
        Ok(self.state.lock().await.status.power)
    }

    async fn set_volume(&self, percent: u8) -> Result<()> {
        self.state.lock().await.status.volume = percent.min(100);
        Ok(())
    }

    async fn get_volume(&self) -> Result<u8> {
        Ok(self.state.lock().await.status.volume)
    }

    async fn set_agc(&self, agc: Agc) -> Result<()> {
        self.state.lock().await.status.agc = agc;
        Ok(())
    }

    async fn get_agc(&self) -> Result<Agc> {
        Ok(self.state.lock().await.status.agc)
    }

    async fn set_vfo(&self, vfo: Vfo) -> Result<()> {
        let mut state = self.state.lock().await;
        state.status.vfo = vfo;
        // Update frequency_hz so status always reflects the active VFO's frequency.
        state.status.frequency_hz = state.freq_for(vfo);
        Ok(())
    }

    async fn get_vfo(&self) -> Result<Vfo> {
        Ok(self.state.lock().await.status.vfo)
    }

    async fn get_status(&self) -> Result<RadioStatus> {
        Ok(self.state.lock().await.status.clone())
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
            assert_eq!(
                radio.get_frequency().await.unwrap(),
                freq,
                "wrong default freq for {band}"
            );
        }
    }

    #[tokio::test]
    async fn test_tune_sets_tuning_flag() {
        let radio = DummyRadio::new();
        radio.tune().await.unwrap();
        let status = radio.get_status().await.unwrap();
        assert!(status.tuning);
    }

    #[tokio::test]
    async fn test_band_remembers_frequency() {
        let radio = DummyRadio::new();

        // Tune 40m to a non-default frequency.
        radio.set_band(Band::Band40m).await.unwrap();
        radio.set_frequency(7_200_000).await.unwrap();

        // Switch to 20m and tune there.
        radio.set_band(Band::Band20m).await.unwrap();
        radio.set_frequency(14_225_000).await.unwrap();

        // Return to 40m — should recall 7.200 MHz, not the 40m default.
        radio.set_band(Band::Band40m).await.unwrap();
        assert_eq!(radio.get_frequency().await.unwrap(), 7_200_000);

        // Return to 20m — should recall 14.225 MHz.
        radio.set_band(Band::Band20m).await.unwrap();
        assert_eq!(radio.get_frequency().await.unwrap(), 14_225_000);
    }

    #[tokio::test]
    async fn test_band_first_visit_uses_default() {
        let radio = DummyRadio::new();
        // First visit to 17m should give the default frequency.
        radio.set_band(Band::Band17m).await.unwrap();
        assert_eq!(radio.get_frequency().await.unwrap(), 18_100_000);
    }

    #[tokio::test]
    async fn test_vfo_roundtrip() {
        let radio = DummyRadio::new();
        assert_eq!(radio.get_vfo().await.unwrap(), Vfo::A);
        radio.set_vfo(Vfo::B).await.unwrap();
        assert_eq!(radio.get_vfo().await.unwrap(), Vfo::B);
        radio.set_vfo(Vfo::A).await.unwrap();
        assert_eq!(radio.get_vfo().await.unwrap(), Vfo::A);
    }

    #[tokio::test]
    async fn test_vfo_in_status() {
        let radio = DummyRadio::new();
        radio.set_vfo(Vfo::B).await.unwrap();
        assert_eq!(radio.get_status().await.unwrap().vfo, Vfo::B);
    }

    #[tokio::test]
    async fn test_vfos_store_independent_frequencies() {
        let radio = DummyRadio::new();

        // Set VFO-A frequency.
        radio.set_vfo(Vfo::A).await.unwrap();
        radio.set_frequency(14_074_000).await.unwrap();

        // Switch to VFO-B and set a different frequency.
        radio.set_vfo(Vfo::B).await.unwrap();
        radio.set_frequency(7_074_000).await.unwrap();
        assert_eq!(radio.get_frequency().await.unwrap(), 7_074_000);

        // Switching back to VFO-A should restore its frequency.
        radio.set_vfo(Vfo::A).await.unwrap();
        assert_eq!(radio.get_frequency().await.unwrap(), 14_074_000);

        // Status frequency_hz reflects the active VFO.
        let status = radio.get_status().await.unwrap();
        assert_eq!(status.frequency_hz, 14_074_000);
        assert_eq!(status.vfo, Vfo::A);
    }

    #[tokio::test]
    async fn test_set_band_only_affects_active_vfo() {
        let radio = DummyRadio::new();

        // Set VFO-A to 40m.
        radio.set_vfo(Vfo::A).await.unwrap();
        radio.set_band(Band::Band40m).await.unwrap();
        let freq_a = radio.get_frequency().await.unwrap();
        assert_eq!(freq_a, 7_074_000);

        // Switch to VFO-B — its frequency is unchanged.
        radio.set_vfo(Vfo::B).await.unwrap();
        let freq_b = radio.get_frequency().await.unwrap();
        assert_ne!(freq_b, freq_a);
    }
}
