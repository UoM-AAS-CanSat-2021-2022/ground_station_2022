use std::env::args;
use std::io;
use std::sync::mpsc::channel;
use std::thread::{self, JoinHandle};

use anyhow::Result;
use eframe::egui;
use ground_station::app::{GroundStationGui, GroundStationGuiBuilder};
use ground_station::listener::TelemetryListener;
use ground_station::reader::TelemetryReader;
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;

fn main() -> Result<()> {
    // initialise the logger
    let log_file_name = format!("{}.log", env!("CARGO_PKG_NAME"));
    std::fs::write(&log_file_name, "")?;

    let file_appender = tracing_appender::rolling::never(".", log_file_name);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_max_level(Level::DEBUG)
        .with_thread_names(true)
        .with_writer(non_blocking.and(io::stderr))
        .init();

    // create a channel for communicating between the reader thread and the main thread
    let arg = args().nth(1).unwrap_or_else(|| String::from("radio"));

    let my_app = match arg.as_str() {
        "reader" => {
            // read the telementry from a file
            let (tx, rx) = channel();
            let mut reader = TelemetryReader::new(tx);
            let _handle: JoinHandle<Result<()>> = thread::Builder::new()
                .name("reader".to_string())
                .spawn(move || reader.run())?;
            GroundStationGuiBuilder::default().packet_rx(rx).build()?
        }
        "listener" => {
            // listen on a port for telemetry
            let (tx, rx) = channel();
            let mut listener = TelemetryListener::new(tx);
            let _handle: JoinHandle<Result<()>> = thread::Builder::new()
                .name("listener".to_string())
                .spawn(move || listener.run())?;
            GroundStationGuiBuilder::default().packet_rx(rx).build()?
        }
        _ => {
            if arg != "radio" {
                tracing::warn!("Unrecognised first argument - {arg:?} - starting in radio mode.");
            }

            GroundStationGui::default()
        }
    };

    // run GUI
    eframe::run_native(
        "MCP Ground Station",
        Default::default(),
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(my_app)
        }),
    )
    .unwrap();

    Ok(())
}
