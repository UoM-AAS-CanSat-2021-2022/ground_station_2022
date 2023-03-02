use anyhow::{bail, ensure};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Result, Write};
use std::num::Wrapping;

mod rx_packet;
mod tx_request;
mod tx_status;

pub use rx_packet::RxPacket;
pub use tx_request::TxRequest;
pub use tx_status::TxStatus;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct XbeePacket {
    pub frame_type: u8,
    pub data: Vec<u8>,
    pub checksum: u8,
}

impl XbeePacket {
    pub(crate) fn new(frame_type: u8, data: Vec<u8>) -> Self {
        let checksum = 0xFF - frame_type - data.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        Self {
            frame_type,
            data,
            checksum,
        }
    }

    /// Serialise the packet out to a vec
    pub fn serialise(self) -> Result<Vec<u8>> {
        let mut buf = vec![];

        // start delimiter
        buf.write_u8(0x7E)?;

        // packet length
        buf.write_u16::<BigEndian>(1u16 + self.data.len() as u16)?;

        // frame type
        buf.write_u8(self.frame_type)?;

        // packet data
        buf.write(&self.data)?;

        // checksum
        buf.write_u8(self.checksum)?;

        Ok(buf)
    }

    /// Attempt to decode a packet from a slice of bytes
    pub fn decode(bytes: &[u8]) -> anyhow::Result<Self> {
        let mut cur = Cursor::new(bytes);
        let mut checksum = Wrapping(0xFF_u8);

        ensure!(cur.read_u8()? == 0x7E, "Invalid packet start byte");

        let mut len = cur.read_u16::<BigEndian>()?;
        // this is some weird fucking edge case :(
        if len == 0x7D {
            // MATE I DON'T FUCKING KNOW THIS SHIT IS BS
            let next = cur.read_u8()?;
            if next == 0x31 {
                len = 0x11;
            } else {
                bail!("Attempted to fix edge case for length of 0x11, found invalid next - next={next:?}");
            }
        }

        let frame_type = cur.read_u8()?;
        checksum -= frame_type;

        let mut data = vec![];
        for _ in 0..len - 1 {
            let byte = cur.read_u8()?;
            data.push(byte);
            checksum -= byte;
        }

        // check the checksum
        let sent_checksum = cur.read_u8()?;
        ensure!(checksum.0 == sent_checksum, "Packet checksum didn't match");

        let packet = XbeePacket {
            frame_type,
            data,
            checksum: sent_checksum,
        };
        Ok(packet)
    }
}

#[derive(Debug)]
pub enum ParsePacketError {
    // indicates that the frame type was wrong
    IncorrectFrameType,
    // a wrapper around an internal IO error
    IoError(std::io::Error),
}

impl From<std::io::Error> for ParsePacketError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_basic_packet_serialise() {
        const CORRECT: &[u8] = &hex!("7E 00 09 01 01 FF FE 00 41 42 43 44 F6");
        let packet = XbeePacket::new(0x01, hex!("01 FF FE 00 41 42 43 44").to_vec());

        assert_eq!(packet.serialise().unwrap(), CORRECT);
    }
}
