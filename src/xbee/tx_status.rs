use crate::xbee::ParsePacketError::IncorrectFrameType;
use crate::xbee::{is_checksum_invalid, ParsePacketError, XbeePacket};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use std::num::Wrapping;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TxStatus {
    src_addr: u16,
    rssi: u8,
    options: u8,
    data: Vec<u8>,
}

impl TryFrom<XbeePacket> for TxStatus {
    type Error = ParsePacketError;

    fn try_from(xbp: XbeePacket) -> Result<Self, Self::Error> {
        let XbeePacket {
            frame_type,
            ref data,
            ..
        } = xbp;

        // check the frame type
        if frame_type != 0x89 {
            return Err(IncorrectFrameType);
        }

        let mut cur = Cursor::new(data.as_slice());
        let src_addr = cur.read_u16::<BigEndian>()?;
        let rssi = cur.read_u8()?;
        let options = cur.read_u8()?;
        let pos = cur.position() as usize;
        let inner_data = data[pos..data.len() - 1].to_vec();

        if is_checksum_invalid(data) {
            tracing::warn!("Invalid checksum on TxStatus packet")
        }

        Ok(TxStatus {
            src_addr,
            rssi,
            options,
            data: inner_data,
        })
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
            frame_id: 0,
            data: hex!("FF FE 00 01 41 42 43 44 76").to_vec(),
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
            frame_id: 0,
            data: hex!("FF FE 00 01 41 42 43 44 76").to_vec(),
        };

        let _packet = RxPacket::try_from(xbp).unwrap_err();
    }
}
