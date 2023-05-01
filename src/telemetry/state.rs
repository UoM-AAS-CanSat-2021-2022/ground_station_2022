use parse_display::Display;
use std::str::FromStr;

/// The different states for the CanSat Software
#[derive(Display, Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// State: YEETED (self explanatory)
    #[display("YEETED")]
    Yeeted,
    #[display("{0}")]
    Other(String),
}

impl FromStr for State {
    type Err = !;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "YEETED" => Self::Yeeted,
            _ => Self::Other(s.to_string()),
        })
    }
}
