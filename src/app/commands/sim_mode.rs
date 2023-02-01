use crate::as_str::AsStr;
use enum_iterator::Sequence;
use parse_display::Display;

#[derive(Sequence, Display, Default, Debug, Copy, Clone, Eq, PartialEq)]
#[display(style = "SNAKE_CASE")]
pub enum SimMode {
    #[default]
    Disable,
    Activate,
    Enable,
}

impl AsStr for SimMode {
    fn as_str(&self) -> &'static str {
        match self {
            SimMode::Disable => "Disable",
            SimMode::Activate => "Activate",
            SimMode::Enable => "Enable",
        }
    }
}
