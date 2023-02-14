use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::sync::mpsc::Sender;

use crate::telemetry::Telemetry;

use anyhow::{bail, Result};

/// Telem
pub struct TelemetryListener {
    tx: Sender<Telemetry>,
}

impl TelemetryListener {
    pub fn new(tx: Sender<Telemetry>) -> Self {
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
                    if let Err(e) = self.tx.send(telem) {
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
