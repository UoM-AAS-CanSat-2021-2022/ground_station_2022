use std::env::args;
use std::sync::mpsc::channel;
use std::thread::{self, JoinHandle};

use anyhow::Result;
use eframe::{egui, NativeOptions};
use ground_station::app::GroundStationGui;
use ground_station::listener::TelemetryListener;
use ground_station::reader::TelemetryReader;
use termcolor::ColorChoice;
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;

fn main() -> Result<()> {
    // initialise the file writer
    let log_file_name = format!("{}.log", env!("CARGO_PKG_NAME"));
    let file_appender = tracing_appender::rolling::never(".", log_file_name);
    let (file_writer, _file_guard) = tracing_appender::non_blocking(file_appender);

    // initialise the colored stdout logger
    let colored_stderr = termcolor::StandardStream::stderr(ColorChoice::Always);
    let (stderr_writer, _stderr_guard) = tracing_appender::non_blocking(colored_stderr);

    // initialise the logging system
    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_max_level(Level::DEBUG)
        .with_thread_names(true)
        .with_writer(file_writer.and(stderr_writer))
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
            GroundStationGui::new_with_receiver(rx)
        }
        "listener" => {
            // listen on a port for telemetry
            let (tx, rx) = channel();
            let mut listener = TelemetryListener::new(tx);
            let _handle: JoinHandle<Result<()>> = thread::Builder::new()
                .name("listener".to_string())
                .spawn(move || listener.run())?;
            GroundStationGui::new_with_receiver(rx)
        }
        _ => {
            if arg != "radio" {
                tracing::warn!("Unrecognised first argument - {arg:?} - starting in radio mode.");
            }

            GroundStationGui::default()
        }
    };

    // run GUI
    let options = NativeOptions {
        maximized: true,
        ..Default::default()
    };
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
