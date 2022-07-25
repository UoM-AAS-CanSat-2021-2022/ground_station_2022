use parse_display::{Display, FromStr};

#[derive(Display, FromStr, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    #[display("C")]
    Container = b'C',
    #[display("T")]
    TetheredPayload = b'T',
}
