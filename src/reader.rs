use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use crate::telemetry::Telemetry;

use anyhow::Result;

/// Telem
pub struct TelemetryReader {
    tx: Sender<Telemetry>,
}

impl TelemetryReader {
    pub fn new(tx: Sender<Telemetry>) -> Self {
        Self { tx }
    }

    pub fn run(&mut self) -> Result<()> {
        // start the reader thread
        let file = File::open("test_data/test_2022.csv")?;
        let buf_reader = BufReader::new(file);

        // collect all the lines so we can cycle them
        let lines: Vec<_> = buf_reader.lines().collect();

        for line in lines.iter().cycle() {
            let line = match line {
                Err(e) => {
                    log::warn!("Encountered error while reading line: {e:?}");
                    continue;
                }
                Ok(line) => line,
            };
            log::trace!("line = {:?}", line);

            match line.parse() {
                Ok(telem) => {
                    if let Err(e) = self.tx.send(telem) {
                        log::warn!("Encountered error sending telemtry over the channel: {e:?}");
                    }
                }
                Err(e) => {
                    log::warn!("Failed to parse received telemetry: {e:?}");
                }
            }

            thread::sleep(Duration::from_millis(200));
        }

        Ok(())
    }
}
