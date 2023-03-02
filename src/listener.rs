use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::sync::mpsc::Sender;

use crate::app::ReceivedPacket;
use crate::xbee::{RxPacket, XbeePacket};
use anyhow::{bail, Result};

/// Telem
pub struct TelemetryListener {
    tx: Sender<ReceivedPacket>,
}

impl TelemetryListener {
    pub fn new(tx: Sender<ReceivedPacket>) -> Self {
        Self { tx }
    }

    pub fn run(&mut self) -> Result<()> {
        // start the listener
        let listener = TcpListener::bind("127.0.0.1:10470")?;
        let (conn, addr) = listener.accept()?;
        tracing::info!("Accepted connection from {addr:?}");
        let buf_reader = BufReader::new(conn);

        for line in buf_reader.lines() {
            let line = match line {
                Err(e) => {
                    tracing::error!("Encountered error while reading line: {e:?}");
                    bail!("Failed to read from socket - {e:?}");
                }
                Ok(line) => line,
            };
            tracing::trace!("line = {:?}", line);

            match line.parse() {
                Ok(telem) => {
                    let packet = ReceivedPacket::Telemetry {
                        packet: XbeePacket {
                            frame_type: 0x81,
                            data: vec![],
                            checksum: 0,
                        },
                        frame: RxPacket {
                            src_addr: 0xFFFF,
                            rssi: 0,
                            options: 0,
                            data: vec![],
                        },
                        telem,
                    };
                    if let Err(e) = self.tx.send(packet) {
                        tracing::warn!(
                            "Encountered error sending telemtry over the channel: {e:?}"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse received telemetry: {e:?}");
                }
            }
        }

        Ok(())
    }
}
