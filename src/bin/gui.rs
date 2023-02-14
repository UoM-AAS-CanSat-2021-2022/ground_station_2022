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
    let (tx, rx) = channel();

    if args().nth(1) == Some(String::from("listen")) {
        // allow listening from the
        let mut listener = TelemetryListener::new(tx.clone());
        let _handle: JoinHandle<Result<()>> = thread::Builder::new()
            .name("listener".to_string())
            .spawn(move || listener.run())?;
    } else {
        // default to just repeating the data from the file
        let mut reader = TelemetryReader::new(tx.clone());
        let _handle: JoinHandle<Result<()>> = thread::Builder::new()
            .name("reader".to_string())
            .spawn(move || reader.run())?;
    }

    // run GUI
    let options = eframe::NativeOptions::default();
    let my_app: GroundStationGui = GroundStationGuiBuilder::default().rx(rx).build()?;

    eframe::run_native(
        "MCP Ground Station",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(my_app)
        }),
    )
    .unwrap();

    Ok(())
}
