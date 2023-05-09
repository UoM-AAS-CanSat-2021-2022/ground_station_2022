use anyhow::{anyhow, ensure, Result};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
// #[display("{h:02}:{m:02}:{s:02}.{cs:02}")]
pub struct MissionTime {
    pub h: u8,
    pub m: u8,
    pub s: u8,
    // centiseconds
    pub cs: u8,
}

impl FromStr for MissionTime {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (h, s) = s
            .split_once(":")
            .ok_or_else(|| anyhow!("Invalid mission time."))?;
        let (m, s) = s
            .split_once(":")
            .ok_or_else(|| anyhow!("Invalid mission time."))?;

        let h = h.parse()?;
        let m = m.parse()?;

        let (s, cs) = if let Some((s, cs)) = s.split_once(".") {
            (s.parse()?, cs.parse()?)
        } else {
            (s.parse()?, u8::MAX)
        };

        Self::new(h, m, s, cs)
    }
}

impl Display for MissionTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let MissionTime { h, m, s, cs } = *self;

        if cs < 100 {
            write!(f, "{h:02}:{m:02}:{s:02}.{cs:02}")
        } else {
            write!(f, "{h:02}:{m:02}:{s:02}")
        }
    }
}

impl MissionTime {
    fn new(h: u8, m: u8, s: u8, cs: u8) -> Result<Self> {
        ensure!(
            h < 24 && m < 60 && s < 60 && (cs < 100 || cs == u8::MAX),
            "Invalid values for mission_time."
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

    pub fn from_seconds(sec: f64) -> Self {
        let h = sec / 3600.0;
        let m = (sec / 60.0) % 60.0;
        let s = sec % 60.0;
        let cs = (sec * 100.0) % 100.0;

        Self {
            h: h as _,
            m: m as _,
            s: s as _,
            cs: cs as _,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Uniform;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_mission_time_fromstr_invalid() {
        let s = "24:34:56.78";
        let ts = s.parse::<MissionTime>();
        ts.unwrap_err();

        let s = "12:60:56.78";
        let ts = s.parse::<MissionTime>();
        ts.unwrap_err();

        let s = "12:34:60.78";
        let ts = s.parse::<MissionTime>();
        ts.unwrap_err();

        let s = "12:34:56.100";
        let ts = s.parse::<MissionTime>();
        ts.unwrap_err();
    }

    #[test]
    fn test_mission_time_fromstr_no_cs() {
        let s = "14:58:56";
        let ts = s.parse::<MissionTime>().unwrap();
        assert_eq!(
            MissionTime {
                h: 14,
                m: 58,
                s: 56,
                cs: u8::MAX,
            },
            ts
        );
        assert_eq!(format!("{ts}"), s);
    }

    #[test]
    fn test_mission_time_display_low_numbers() {
        let mt = MissionTime {
            h: 1,
            m: 2,
            s: 3,
            cs: 4,
        };

        assert_eq!(format!("{}", mt), "01:02:03.04".to_string())
    }

    #[test]
    fn test_from_seconds_recovers_exact_time() {
        let mut rng = thread_rng();

        let range_24 = Uniform::new(0, 24u8);
        let range_60 = Uniform::new(0, 60u8);
        let range_100 = Uniform::new(0, 100u8);

        // generate a random time and check it worked
        for _ in 0..1000 {
            let mt = MissionTime {
                h: rng.sample(range_24),
                m: rng.sample(range_60),
                s: rng.sample(range_60),
                cs: rng.sample(range_100),
            };
            let rt = MissionTime::from_seconds(mt.as_seconds());

            // must recovert hour, minute, second, centisecond can have some leniency
            assert_eq!(mt.h, rt.h);
            assert_eq!(mt.m, rt.m);
            assert_eq!(mt.s, rt.s);
            assert!(mt.cs.abs_diff(rt.cs) <= 1);
        }
    }
}
