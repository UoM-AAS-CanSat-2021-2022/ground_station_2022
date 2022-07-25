use parse_display::{Display, FromStr};

#[derive(Display, FromStr, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum TpReleased {
    #[display("N")]
    NotReleased = b'N',
    #[display("R")]
    Released = b'R',
}
