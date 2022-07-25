use anyhow::{ensure, Result};
use parse_display::{Display, FromStr};

#[derive(Display, FromStr, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[display("{h:02}:{m:02}:{s:02}.{cs:02}")]
#[from_str(new = Self::new(h, m, s, cs))]
pub struct Timestamp {
    pub h: u8,
    pub m: u8,
    pub s: u8,
    // centiseconds
    pub cs: u8,
}

impl Timestamp {
    fn new(h: u8, m: u8, s: u8, cs: u8) -> Result<Self> {
        ensure!(
            h < 24 && m < 60 && s < 60 && cs < 100,
            "Invalid values for timestamp."
        );

        Ok(Self { h, m, s, cs })
    }

    #[rustfmt::skip]
    pub fn as_seconds(&self) -> f64 {
        self.h as f64 * 3600.0
            + self.m as f64 * 60.0
            + self.s as f64
            + self.cs as f64 / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_fromstr_invalid() {
        let s = "24:34:56.78";
        let ts = s.parse::<Timestamp>();
        ts.unwrap_err();

        let s = "12:60:56.78";
        let ts = s.parse::<Timestamp>();
        ts.unwrap_err();

        let s = "12:34:60.78";
        let ts = s.parse::<Timestamp>();
        ts.unwrap_err();

        let s = "12:34:56.100";
        let ts = s.parse::<Timestamp>();
        ts.unwrap_err();
    }

    #[test]
    fn test_timestamp_display_low_numbers() {
        let ts = Timestamp {
            h: 1,
            m: 2,
            s: 3,
            cs: 4,
        };

        assert_eq!(format!("{}", ts), "01:02:03.04".to_string())
    }
}
