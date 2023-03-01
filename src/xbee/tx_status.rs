use crate::as_str::AsStr;
use crate::xbee::ParsePacketError::IncorrectFrameType;
use crate::xbee::{is_checksum_invalid, ParsePacketError, XbeePacket};
use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;
use std::fmt;

// definitely don't need most of these but I thought I might as well implement them
#[derive(Debug, Clone, Eq, PartialEq, Primitive)]
pub enum TxStatus {
    Success = 0x00,
    NoAck = 0x01,
    CcaFailure = 0x02,
    Purged = 0x03,
    WifiPhysicalErrro = 0x04,
    InvalidDestination = 0x15,
    NoBuffers = 0x18,
    NetworkAckFailure = 0x21,
    NotJoinedNetwork = 0x22,
    SelfAddressed = 0x23,
    AddressNotFound = 0x24,
    RouteNotFound = 0x25,
    BroadcastFailed = 0x26,
    InvalidBindingTableIndex = 0x2B,
    InvalidEndpoint = 0x2C,
    BroadcastErrorAps = 0x2D,
    BroadcastErrorApsEe0 = 0x2E,
    SoftwareError = 0x31,
    ResourceError = 0x32,
    NoSecureSession = 0x34,
    EncFailure = 0x35,
    PayloadTooLarge = 0x74,
    IndirectMessageUnrequested = 0x75,
    SocketCreationFailed = 0x76,
    IpPortNotExist = 0x77,
    UdpSrcPortNotMatchListeningPort = 0x78,
    TcpSrcPortNotMatchListeningPort = 0x79,
    InvalidIpAddress = 0x7A,
    InvalidIpProtocol = 0x7B,
    RelayInterfaceInvalid = 0x7C,
    RelayInterfaceRejected = 0x7D,
    ModemUpdateInProgress = 0x7E,
    SocketConnectionRefused = 0x80,
    SocketConnectionLost = 0x81,
    SocketErrorNoServer = 0x82,
    SocketErrorClosed = 0x83,
    SocketErrorUnknownServer = 0x84,
    SocketErrorUnknownError = 0x85,
    InvalidTlsConfiguration = 0x86,
    SocketNotConnected = 0x87,
    SocketNotBound = 0x88,
    KeyNotAuthorized = 0xBB,
    UNKNOWN = 0xFF,
}

impl AsStr for TxStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::NoAck => "No acknowledgement received",
            Self::CcaFailure => "CCA failure",
            Self::Purged => "Transmission purged, it was attempted before stack was up",
            Self::WifiPhysicalErrro => "Transceiver was unable to complete the transmission",
            Self::InvalidDestination => "Invalid destination endpoint",
            Self::NoBuffers => "No buffers",
            Self::NetworkAckFailure => "Network ACK Failure",
            Self::NotJoinedNetwork => "Not joined to network",
            Self::SelfAddressed => "Self-addressed",
            Self::AddressNotFound => "Address not found",
            Self::RouteNotFound => "Route not found",
            Self::BroadcastFailed => "Broadcast source failed to hear a neighbor relay the message",
            Self::InvalidBindingTableIndex => "Invalid binding table index",
            Self::InvalidEndpoint => "Invalid endpoint",
            Self::BroadcastErrorAps => "Attempted broadcast with APS transmission",
            Self::BroadcastErrorApsEe0 => "Attempted broadcast with APS transmission, but EE=0",
            Self::SoftwareError => "A software error occurred",
            Self::ResourceError => "Resource error lack of free buffers, timers, etc",
            Self::NoSecureSession => "No Secure session connection",
            Self::EncFailure => "Encryption failure",
            Self::PayloadTooLarge => "Data payload too large",
            Self::IndirectMessageUnrequested => "Indirect message unrequested",
            Self::SocketCreationFailed => "Attempt to create a client socket failed",
            Self::IpPortNotExist => "TCP connection to given IP address and port does not exist. Source port is non-zero, so a new connection is not attempted",
            Self::UdpSrcPortNotMatchListeningPort => "Source port on a UDP transmission does not match a listening port on the transmitting module",
            Self::TcpSrcPortNotMatchListeningPort => "Source port on a TCP transmission does not match a listening port on the transmitting module",
            Self::InvalidIpAddress => "Destination IPv4 address is invalid",
            Self::InvalidIpProtocol => "Protocol on an IPv4 transmission is invalid",
            Self::RelayInterfaceInvalid => "Destination interface on a User Data Relay Frame does not exist",
            Self::RelayInterfaceRejected => "Destination interface on a User Data Relay Frame exists, but the interface is not accepting data",
            Self::ModemUpdateInProgress => "Modem update in progress. Try again after update completion.",
            Self::SocketConnectionRefused => "Destination server refused the connection",
            Self::SocketConnectionLost => "The existing connection was lost before the data was sent",
            Self::SocketErrorNoServer => "No server",
            Self::SocketErrorClosed => "The existing connection was closed",
            Self::SocketErrorUnknownServer => "The server could not be found",
            Self::SocketErrorUnknownError => "An unknown error occurred",
            Self::InvalidTlsConfiguration => "TLS Profile on a 0x23 API request does not exist, or one or more certificates is invalid",
            Self::SocketNotConnected => "Socket not connected",
            Self::SocketNotBound => "Socket not bound",
            Self::KeyNotAuthorized => "Key not authorized",
            Self::UNKNOWN => "Unknown",
        }
    }
}

impl fmt::Display for TxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
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

        let status = TxStatus::from_u8(data[0]).unwrap_or(TxStatus::UNKNOWN);

        if is_checksum_invalid(data) {
            tracing::warn!("Invalid checksum on TxStatus packet")
        }

        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use crate::xbee::{TxStatus, XbeePacket};
    use hex_literal::hex;

    #[test]
    fn test_rx_packet_parse() {
        let xbp = XbeePacket {
            frame_type: 0x89,
            data: hex!("00 75").to_vec(),
        };

        let packet = TxStatus::try_from(xbp).unwrap();

        assert_eq!(packet, TxStatus::Success,)
    }

    #[test]
    fn test_rx_packet_parse_fails_invalid_frame_type() {
        let xbp = XbeePacket {
            frame_type: 0x90,
            data: hex!("00 75").to_vec(),
        };

        let _packet = TxStatus::try_from(xbp).unwrap_err();
    }
}
