use parse_display::{Display, FromStr};

/// The different states for the CanSat Software
#[derive(Display, FromStr, Debug, Copy, Clone, PartialEq, Eq)]
pub enum State {
    /// State: YEETED (self explanatory)
    #[display("YEETED")]
    Yeeted,
}
