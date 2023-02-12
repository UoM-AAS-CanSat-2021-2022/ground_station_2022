use anyhow::Result;
use ground_station::xbee::{TxRequest, XbeePacket};
use hex_literal::hex;
use serialport::SerialPortType;
use std::io::Write;
use std::time::Duration;

fn main() -> Result<()> {
    // let ports = serialport::available_ports().unwrap();
    // let port = ports
    //     .into_iter()
    //     .filter(|port| matches!(port.port_type, SerialPortType::UsbPort(_)))
    //     .next()
    //     .unwrap();

    let req = TxRequest::new(0x00_00, "CMD,1047,CAL");
    let mut packet: XbeePacket = req.try_into().unwrap();
    packet.set_frame_id(1);
    eprintln!("{packet:02X?}");
    let ser = packet.serialise()?;
    eprintln!("{ser:02X?}");

    let mut sport = serialport::new("/dev/ttyUSB0", 230400).open()?;
    sport.write(&ser)?;
    let mut buf = [0u8; 1024];
    sport.set_timeout(Duration::from_secs(2))?;
    let bytes_read = sport.read(&mut buf)?;
    eprintln!("{:02X?}", &buf[..bytes_read]);
    dbg!(XbeePacket::decode(&buf[..bytes_read]));

    Ok(())
}
