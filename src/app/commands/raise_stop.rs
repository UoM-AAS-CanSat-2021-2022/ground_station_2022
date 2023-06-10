use crate::as_str::AsStr;
use enum_iterator::Sequence;
use parse_display::Display;
use std::default::Default;

/// Represents the argument to the
#[derive(Default, Sequence, Display, Copy, Clone, Eq, PartialEq)]
#[display(style = "UPPERCASE")]
pub enum RaiseStop {
    #[default]
    Raise,
    Stop,
}

// I know this is horrible, anyone reading this, I'm sorry
impl AsStr for RaiseStop {
    fn as_str(&self) -> &'static str {
        match self {
            RaiseStop::Raise => "RAISE",
            RaiseStop::Stop => "STOP",
        }
    }
}
