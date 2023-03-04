use crate::xbee::XbeePacket;
use byteorder::{BigEndian, WriteBytesExt};
use std::fmt;
use std::io::Write;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TxRequest {
    /// The frame ID
    pub frame_id: u8,
    /// The destination address
    pub dst: u16,
    /// The data to send
    pub data: Vec<u8>,
}

impl TxRequest {
    pub fn new(frame_id: u8, dst: u16, data: impl AsRef<[u8]>) -> Self {
        Self {
            frame_id,
            dst,
            data: data.as_ref().to_vec(),
        }
    }
}

impl TryFrom<TxRequest> for XbeePacket {
    type Error = std::io::Error;

    fn try_from(req: TxRequest) -> Result<Self, Self::Error> {
        let mut buf = vec![];

        // frame ID
        buf.write_u8(req.frame_id)?;

        // dst addr
        buf.write_u16::<BigEndian>(req.dst)?;

        // options
        buf.write_u8(0)?;

        // data
        buf.write(&req.data)?;

        Ok(XbeePacket::new(0x01, buf))
    }
}

impl fmt::Display for TxRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TxRequest {{ frame_id: {}, dst: {:02X}:{:02X}, data: {:?} }}",
            self.frame_id,
            self.dst >> 8,
            self.dst & 0xFF,
            String::from_utf8_lossy(self.data.as_slice())
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_tx_req_serialisation() {
        const CAL: &[u8] =
            &hex!("7E 00 14 01 01 00 01 00 43 4D 44 2C 31 30 34 37 2C 53 54 2C 47 50 53 47");

        let req = TxRequest::new(1, 0x00_01, "CMD,1047,ST,GPS");
        let packet: XbeePacket = req.try_into().unwrap();
        assert_eq!(packet.serialise().unwrap(), CAL);
    }
}
