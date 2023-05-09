use crate::xbee::ParsePacketError::IncorrectFrameType;
use crate::xbee::{ParsePacketError, XbeePacket};
use byteorder::{BigEndian, ReadBytesExt};
use core::fmt;
use std::io::Cursor;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RxPacket {
    pub src_addr: u16,
    pub rssi: i8,
    pub options: u8,
    pub data: Vec<u8>,
}

impl TryFrom<XbeePacket> for RxPacket {
    type Error = ParsePacketError;

    fn try_from(xbp: XbeePacket) -> Result<Self, Self::Error> {
        let XbeePacket {
            frame_type,
            ref data,
            ..
        } = xbp;

        // check the frame type
        if frame_type != 0x81 {
            return Err(IncorrectFrameType);
        }

        let mut cur = Cursor::new(data.as_slice());
        let src_addr = cur.read_u16::<BigEndian>()?;
        let rssi = cur.read_i8()?;
        let options = cur.read_u8()?;
        let pos = cur.position() as usize;
        let inner_data = data[pos..data.len()].to_vec();

        Ok(RxPacket {
            src_addr,
            rssi,
            options,
            data: inner_data,
        })
    }
}

impl fmt::Display for RxPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RxPacket {{ src: {:02X}:{:02X}, rssi: -{}dBm, options: {:02}, data: {:?} }}",
            self.src_addr >> 8,
            self.src_addr & 0xFF,
            self.rssi,
            self.options,
            String::from_utf8_lossy(self.data.as_slice()),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::xbee::rx_packet::RxPacket;
    use crate::xbee::XbeePacket;
    use hex_literal::hex;

    #[test]
    fn test_rx_packet_parse() {
        let xbp = XbeePacket {
            frame_type: 0x81,
            data: hex!("FF FE 00 01 41 42 43 44").to_vec(),
            checksum: 1,
        };

        let packet = RxPacket::try_from(xbp).unwrap();

        assert_eq!(
            packet,
            RxPacket {
                src_addr: 0xFFFE,
                rssi: 0,
                options: 1,
                data: hex!("41 42 43 44").to_vec(),
            }
        )
    }

    #[test]
    fn test_rx_packet_parse_fails_invalid_frame_type() {
        let xbp = XbeePacket {
            frame_type: 0x82,
            data: hex!("FF FE 00 01 41 42 43 44 76").to_vec(),
            checksum: 1,
        };

        let _packet = RxPacket::try_from(xbp).unwrap_err();
    }
}
