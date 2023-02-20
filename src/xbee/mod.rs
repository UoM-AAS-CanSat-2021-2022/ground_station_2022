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
    frame_type: u8,
    frame_id: u8,
    data: Vec<u8>,
}

impl XbeePacket {
    pub(crate) fn new(frame_type: u8, frame_id: Option<u8>, data: Vec<u8>) -> Self {
        Self {
            frame_id: frame_id.unwrap_or(0u8),
            frame_type,
            data,
        }
    }

    pub fn set_frame_id(&mut self, frame_id: u8) {
        self.frame_id = frame_id;
    }

    /// Serialise the packet out to a vec
    pub fn serialise(self) -> Result<Vec<u8>> {
        let mut buf = vec![];
        let mut checksum = Wrapping(0xff_u8);

        // start delimiter
        buf.write_u8(0x7E)?;

        // packet length
        buf.write_u16::<BigEndian>(2u16 + self.data.len() as u16)?;

        // frame type
        buf.write_u8(self.frame_type)?;
        checksum -= self.frame_type;

        // frame ID
        buf.write_u8(self.frame_id)?;
        checksum -= self.frame_id;

        // packet data
        buf.write(&self.data)?;
        checksum -= self
            .data
            .into_iter()
            .fold(0u8, |acc, x| acc.wrapping_add(x));

        // checksum
        buf.write_u8(checksum.0)?;

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

        let frame_id = cur.read_u8()?;
        checksum -= frame_id;

        let mut data = vec![];
        for _ in 2..len {
            let byte = cur.read_u8()?;
            data.push(byte);
            checksum -= byte;
        }

        // check the checksum
        let sent_checksum = cur.read_u8()?;
        ensure!(checksum.0 == sent_checksum, "Packet checksum didn't match");

        let packet = XbeePacket {
            frame_type,
            frame_id,
            data,
        };
        Ok(packet)
    }
}

/// returns whether the checksum for the given data was valid
///
/// data: [packet data] || checksum
#[inline(always)]
fn is_checksum_invalid(data: &[u8]) -> bool {
    let csum = 0xFF
        - data
            .iter()
            .take(data.len() - 1)
            .copied()
            .map(Wrapping)
            .sum::<Wrapping<u8>>()
            .0;
    let check = *data.last().unwrap();

    csum != check
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
        let packet = XbeePacket::new(0x01, Some(0x01), hex!("FF FE 00 41 42 43 44").to_vec());

        assert_eq!(packet.serialise().unwrap(), CORRECT);
    }
}
