use anyhow::Result;
use ground_station::telemetry::Telemetry;
use ground_station::xbee::{TxRequest, XbeePacket};
use std::io::Write;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    let usb_port = std::env::args()
        .nth(1)
        .expect("Need at least 2 arguments - first is port, second is the telemetry file");
    let mut sport = serialport::new(dbg!(usb_port), 230400).open()?;

    let fname = std::env::args()
        .nth(2)
        .expect("Need one more argument - the telemetry file");
    let file_data = std::fs::read_to_string(fname)?;
    let telemetry: Vec<Telemetry> = file_data
        .lines()
        .map(str::parse)
        .collect::<Result<_, _>>()?;

    let mut frame_id = 0;
    for telem in telemetry {
        let req = TxRequest::new(0xFFFF, format!("{telem}"));
        let mut packet: XbeePacket = req.try_into().unwrap();
        packet.set_frame_id(frame_id);
        frame_id = frame_id.wrapping_add(1);
        eprintln!("created packet: {packet:02X?}");
        let ser = packet.serialise()?;
        eprintln!("sending: {ser:02X?}");
        sport.write(&ser)?;

        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}
