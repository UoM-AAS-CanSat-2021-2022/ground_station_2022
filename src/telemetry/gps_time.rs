use anyhow::{ensure, Result};
use parse_display::{Display, FromStr};

/// time from the GPS receiver
/// must be reported in UTC and have a resolution of a second
#[derive(Display, FromStr, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[display("{h:02}:{m:02}:{s:02}")]
#[from_str(new = Self::new(h, m, s))]
pub struct GpsTime {
    /// Hours
    pub h: u8,

    /// Minutes
    pub m: u8,

    /// Seconds
    pub s: u8,
}

impl GpsTime {
    fn new(h: u8, m: u8, s: u8) -> Result<Self> {
        ensure!(h < 24 && m < 60 && s < 60, "Invalid values for gps_time.");

        Ok(Self { h, m, s })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_misstion_time_fromstr_invalid() {
        let s = "24:34:56";
        let ts = s.parse::<GpsTime>();
        ts.unwrap_err();

        let s = "12:60:56";
        let ts = s.parse::<GpsTime>();
        ts.unwrap_err();

        let s = "12:34:60";
        let ts = s.parse::<GpsTime>();
        ts.unwrap_err();
    }

    #[test]
    fn test_misstion_time_display_low_numbers() {
        let gt = GpsTime { h: 1, m: 2, s: 3 };

        assert_eq!(format!("{}", gt), "01:02:03".to_string())
    }
}
