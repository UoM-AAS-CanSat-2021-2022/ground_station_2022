use crate::as_str::AsStr;
use enum_iterator::Sequence;

#[derive(Sequence, Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Time {
    /// Set the time to a custom manual time
    Manual,

    /// Set the time automatically from the current UTC time
    #[default]
    CurrUtc,

    /// Set the time from the time read by the GPS
    Gps,
}

impl AsStr for Time {
    fn as_str(&self) -> &'static str {
        match self {
            Time::Manual => "Manual",
            Time::CurrUtc => "Current UTC",
            Time::Gps => "GPS",
        }
    }
}
