use crate::as_str::AsStr;
use enum_iterator::Sequence;
use parse_display::Display;

#[derive(Sequence, Display, Default, Debug, Copy, Clone, Eq, PartialEq)]
#[display(style = "SNAKE_CASE")]
pub enum Action {
    #[default]
    Enable,
    Disable,
    Flag,
    Beacon,
}

impl AsStr for Action {
    fn as_str(&self) -> &'static str {
        match self {
            Action::Enable => "Enable",
            Action::Disable => "Disable",
            Action::Flag => "Flag",
            Action::Beacon => "Beacon",
        }
    }
}
