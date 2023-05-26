use chrono::{Timelike, Utc};
use ground_station::telemetry::*;
use rand::{
    distributions::{Open01, Slice, Uniform},
    prelude::*,
};
use std::io::ErrorKind;
use std::ops::AddAssign;
use std::{
    io::{self, Write},
    net::TcpStream,
    thread, time,
};
use tracing::Level;

fn main() -> anyhow::Result<()> {
    // setup logging
    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_max_level(Level::DEBUG)
        .with_writer(io::stderr)
        .init();

    tracing::info!("Available ports:");
    for port in serialport::available_ports()? {
        tracing::info!("port: {port:?}");
    }

    let usb_port = std::env::args()
        .nth(1)
        .expect("FATAL: Missing first argument - serial port");
    let mut sport = serialport::new(dbg!(usb_port), 9600).open()?;

    // connect to the frontend, retry after 1 second
    let address = "127.0.0.1:10470";
    let mut stream = loop {
        match TcpStream::connect(address) {
            Ok(s) => break s,
            Err(e) => {
                tracing::warn!("Failed to connect to frontend on {address} - {e}");
                thread::sleep(time::Duration::from_millis(200));
            }
        }
    };

    let mut error_count = 0;
    loop {
        // read telem
        sport.read

        if let Err(e) = writeln!(stream, "{telem}") {
            if matches!(e.kind(), ErrorKind::BrokenPipe | ErrorKind::ConnectionReset) {
                tracing::info!("Client has disconnected, exiting.");
                break Ok(());
            }

            tracing::warn!("Failed to send telemetry packet: {e} - {error_count} errors so far");
            error_count += 1;
        }
    }
}
