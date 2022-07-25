use parse_display::{Display, FromStr};

#[derive(Display, FromStr, Debug, Copy, Clone, PartialEq, Eq)]
pub enum TpSoftwareState {
    #[display(style = "UPPERCASE")]
    Released,
}
