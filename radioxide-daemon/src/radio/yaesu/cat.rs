use radioxide_proto::{Agc, Band, Mode, Vfo};

use crate::radio::BackendError;

/// A CAT command to send to the radio.
pub struct CatCommand {
    data: String,
}

impl CatCommand {
    /// Build a read command: e.g., `FA;`
    pub fn read(verb: &str) -> Self {
        Self {
            data: format!("{verb};"),
        }
    }

    /// Build a set command: e.g., `FA14250000;`
    pub fn set(verb: &str, params: &str) -> Self {
        Self {
            data: format!("{verb}{params};"),
        }
    }

    /// Get the wire bytes to send.
    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_bytes()
    }

    /// Get the 2-character verb for response matching.
    pub fn verb(&self) -> &str {
        &self.data[..2]
    }
}

/// A parsed CAT response from the radio.
#[derive(Debug)]
pub struct CatResponse {
    #[allow(dead_code)]
    pub verb: String,
    pub body: String,
}

impl CatResponse {
    /// Parse a raw response string (including the trailing `;`).
    /// Validates that the verb matches `expected_verb`.
    pub fn parse(raw: &str, expected_verb: &str) -> crate::radio::Result<Self> {
        let raw = raw.trim_end_matches(';');
        if raw.len() < 2 {
            return Err(BackendError::Protocol(format!(
                "response too short: {raw:?}"
            )));
        }
        let verb = &raw[..2];
        if !verb.eq_ignore_ascii_case(expected_verb) {
            return Err(BackendError::Protocol(format!(
                "expected verb '{expected_verb}', got '{verb}'"
            )));
        }
        Ok(Self {
            verb: verb.to_string(),
            body: raw[2..].to_string(),
        })
    }
}

// --- Mode mapping ---

/// Map proto::Mode to FT-450D MD command P2 character.
pub fn mode_to_cat(mode: Mode) -> crate::radio::Result<char> {
    match mode {
        Mode::LSB => Ok('1'),
        Mode::USB => Ok('2'),
        Mode::CW => Ok('3'),
        Mode::FM => Ok('4'),
        Mode::AM => Ok('5'),
        Mode::Digital => Ok('6'),
        Mode::CWR => Ok('7'),
        Mode::DigitalR => Ok('9'),
    }
}

/// Map FT-450D MD response P2 character to proto::Mode.
pub fn cat_to_mode(c: char) -> crate::radio::Result<Mode> {
    match c {
        '1' => Ok(Mode::LSB),
        '2' => Ok(Mode::USB),
        '3' => Ok(Mode::CW),
        '4' => Ok(Mode::FM),
        '5' => Ok(Mode::AM),
        '6' => Ok(Mode::Digital),
        '7' => Ok(Mode::CWR),
        '9' => Ok(Mode::DigitalR),
        _ => Err(BackendError::Protocol(format!("unknown mode char: {c}"))),
    }
}

// --- Band mapping ---

/// Map proto::Band to FT-450D BS command P1 value.
pub fn band_to_cat(band: Band) -> crate::radio::Result<&'static str> {
    match band {
        Band::Band160m => Ok("00"),
        Band::Band80m => Ok("01"),
        Band::Band40m => Ok("03"),
        Band::Band30m => Ok("04"),
        Band::Band20m => Ok("05"),
        Band::Band17m => Ok("06"),
        Band::Band15m => Ok("07"),
        Band::Band12m => Ok("08"),
        Band::Band10m => Ok("09"),
        Band::Band6m => Ok("10"),
        Band::Band60m | Band::Band2m | Band::Band70cm => Err(BackendError::InvalidParameter(
            format!("band {band} is not supported by the FT-450D"),
        )),
    }
}

/// Determine band from frequency in Hz.
pub fn freq_to_band(hz: u64) -> Option<Band> {
    match hz {
        1_800_000..=2_000_000 => Some(Band::Band160m),
        3_500_000..=4_000_000 => Some(Band::Band80m),
        5_300_000..=5_400_000 => Some(Band::Band60m),
        7_000_000..=7_300_000 => Some(Band::Band40m),
        10_100_000..=10_150_000 => Some(Band::Band30m),
        14_000_000..=14_350_000 => Some(Band::Band20m),
        18_068_000..=18_168_000 => Some(Band::Band17m),
        21_000_000..=21_450_000 => Some(Band::Band15m),
        24_890_000..=24_990_000 => Some(Band::Band12m),
        28_000_000..=29_700_000 => Some(Band::Band10m),
        50_000_000..=54_000_000 => Some(Band::Band6m),
        144_000_000..=148_000_000 => Some(Band::Band2m),
        420_000_000..=450_000_000 => Some(Band::Band70cm),
        _ => None,
    }
}

// --- AGC mapping ---

/// Map proto::Agc to FT-450D GT command P2 character.
/// FT-450D: 0=OFF, 1=FAST, 2=SLOW, 3=SLOW, 4=AUTO
/// We map Medium → AUTO (4) since the FT-450D has no explicit medium AGC.
pub fn agc_to_cat(agc: Agc) -> char {
    match agc {
        Agc::Off => '0',
        Agc::Fast => '1',
        Agc::Slow => '2',
        Agc::Medium => '4', // AUTO
    }
}

/// Map FT-450D GT response P2 character to proto::Agc.
pub fn cat_to_agc(c: char) -> crate::radio::Result<Agc> {
    match c {
        '0' => Ok(Agc::Off),
        '1' => Ok(Agc::Fast),
        '2' | '3' => Ok(Agc::Slow),
        '4' => Ok(Agc::Medium), // AUTO
        _ => Err(BackendError::Protocol(format!("unknown AGC char: {c}"))),
    }
}

// --- VFO mapping ---

/// Map proto::Vfo to FT-450D VS command P1 character.
/// VS0 = VFO-A, VS1 = VFO-B.
pub fn vfo_to_cat(vfo: Vfo) -> char {
    match vfo {
        Vfo::A => '0',
        Vfo::B => '1',
    }
}

/// Map FT-450D VS response P1 character to proto::Vfo.
pub fn cat_to_vfo(c: char) -> crate::radio::Result<Vfo> {
    match c {
        '0' => Ok(Vfo::A),
        '1' => Ok(Vfo::B),
        _ => Err(BackendError::Protocol(format!("unknown VFO char: {c}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cat_command_read() {
        let cmd = CatCommand::read("FA");
        assert_eq!(cmd.as_bytes(), b"FA;");
        assert_eq!(cmd.verb(), "FA");
    }

    #[test]
    fn test_cat_command_set() {
        let cmd = CatCommand::set("FA", "14250000");
        assert_eq!(cmd.as_bytes(), b"FA14250000;");
        assert_eq!(cmd.verb(), "FA");
    }

    #[test]
    fn test_cat_response_parse() {
        let resp = CatResponse::parse("FA14250000;", "FA").unwrap();
        assert_eq!(resp.verb, "FA");
        assert_eq!(resp.body, "14250000");
    }

    #[test]
    fn test_cat_response_wrong_verb() {
        assert!(CatResponse::parse("FB14250000;", "FA").is_err());
    }

    #[test]
    fn test_cat_response_too_short() {
        assert!(CatResponse::parse("F;", "FA").is_err());
    }

    #[test]
    fn test_mode_roundtrip() {
        let modes = [
            Mode::LSB,
            Mode::USB,
            Mode::CW,
            Mode::FM,
            Mode::AM,
            Mode::Digital,
            Mode::CWR,
            Mode::DigitalR,
        ];
        for mode in modes {
            let c = mode_to_cat(mode).unwrap();
            let back = cat_to_mode(c).unwrap();
            assert_eq!(mode, back, "roundtrip failed for {mode:?}");
        }
    }

    #[test]
    fn test_band_to_cat() {
        assert_eq!(band_to_cat(Band::Band20m).unwrap(), "05");
        assert_eq!(band_to_cat(Band::Band40m).unwrap(), "03");
        assert!(band_to_cat(Band::Band2m).is_err());
        assert!(band_to_cat(Band::Band70cm).is_err());
    }

    #[test]
    fn test_freq_to_band() {
        assert_eq!(freq_to_band(14_074_000), Some(Band::Band20m));
        assert_eq!(freq_to_band(7_074_000), Some(Band::Band40m));
        assert_eq!(freq_to_band(1_840_000), Some(Band::Band160m));
        assert_eq!(freq_to_band(100_000), None);
    }

    #[test]
    fn test_agc_roundtrip() {
        for agc in [Agc::Off, Agc::Fast, Agc::Slow] {
            let c = agc_to_cat(agc);
            let back = cat_to_agc(c).unwrap();
            assert_eq!(agc, back);
        }
        // Medium maps to AUTO ('4') which maps back to Medium
        let c = agc_to_cat(Agc::Medium);
        assert_eq!(c, '4');
        assert_eq!(cat_to_agc(c).unwrap(), Agc::Medium);
    }

    #[test]
    fn test_freq_to_band_all_bands() {
        let cases = [
            (1_900_000, Band::Band160m),
            (3_700_000, Band::Band80m),
            (5_350_000, Band::Band60m),
            (7_100_000, Band::Band40m),
            (10_120_000, Band::Band30m),
            (14_200_000, Band::Band20m),
            (18_100_000, Band::Band17m),
            (21_200_000, Band::Band15m),
            (24_950_000, Band::Band12m),
            (28_500_000, Band::Band10m),
            (50_100_000, Band::Band6m),
            (146_000_000, Band::Band2m),
            (435_000_000, Band::Band70cm),
        ];
        for (hz, expected) in cases {
            assert_eq!(freq_to_band(hz), Some(expected), "wrong band for {hz} Hz");
        }
    }

    #[test]
    fn test_freq_to_band_unknown() {
        assert_eq!(freq_to_band(500_000), None);
        assert_eq!(freq_to_band(100_000_000), None);
        assert_eq!(freq_to_band(0), None);
    }

    #[test]
    fn test_mode_to_cat_all_modes() {
        let expected = [
            (Mode::LSB, '1'),
            (Mode::USB, '2'),
            (Mode::CW, '3'),
            (Mode::FM, '4'),
            (Mode::AM, '5'),
            (Mode::Digital, '6'),
            (Mode::CWR, '7'),
            (Mode::DigitalR, '9'),
        ];
        for (mode, c) in expected {
            assert_eq!(mode_to_cat(mode).unwrap(), c, "wrong CAT char for {mode:?}");
        }
    }

    #[test]
    fn test_cat_to_mode_all() {
        let expected = [
            ('1', Mode::LSB),
            ('2', Mode::USB),
            ('3', Mode::CW),
            ('4', Mode::FM),
            ('5', Mode::AM),
            ('6', Mode::Digital),
            ('7', Mode::CWR),
            ('9', Mode::DigitalR),
        ];
        for (c, mode) in expected {
            assert_eq!(cat_to_mode(c).unwrap(), mode, "wrong mode for CAT char '{c}'");
        }
        assert!(cat_to_mode('8').is_err());
        assert!(cat_to_mode('0').is_err());
    }

    #[test]
    fn test_agc_to_cat_all() {
        assert_eq!(agc_to_cat(Agc::Off), '0');
        assert_eq!(agc_to_cat(Agc::Fast), '1');
        assert_eq!(agc_to_cat(Agc::Slow), '2');
        assert_eq!(agc_to_cat(Agc::Medium), '4');
    }

    #[test]
    fn test_band_to_cat_unsupported() {
        assert!(band_to_cat(Band::Band60m).is_err());
        assert!(band_to_cat(Band::Band2m).is_err());
        assert!(band_to_cat(Band::Band70cm).is_err());
    }

    #[test]
    fn test_vfo_roundtrip() {
        for vfo in [Vfo::A, Vfo::B] {
            let c = vfo_to_cat(vfo);
            let back = cat_to_vfo(c).unwrap();
            assert_eq!(vfo, back, "roundtrip failed for {vfo:?}");
        }
    }

    #[test]
    fn test_vfo_to_cat() {
        assert_eq!(vfo_to_cat(Vfo::A), '0');
        assert_eq!(vfo_to_cat(Vfo::B), '1');
    }

    #[test]
    fn test_cat_to_vfo_invalid() {
        assert!(cat_to_vfo('2').is_err());
        assert!(cat_to_vfo('X').is_err());
    }
}
