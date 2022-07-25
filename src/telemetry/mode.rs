use parse_display::{Display, FromStr};

#[derive(Display, FromStr, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    #[display("F")]
    Flight = b'F',
    #[display("S")]
    Simulation = b'S',
}
