mod graphable;
use graphable::Graphable;

use std::sync::mpsc::Receiver;

use derive_builder::Builder;
use eframe::egui;
use egui::plot::{Line, Plot};
use egui::WidgetText;
use enum_iterator::all;

use crate::telemetry::Telemetry;

#[derive(Builder)]
#[builder(pattern = "owned", default)]
#[derive(Default)]
pub struct GroundStationGui {
    #[builder(setter(strip_option))]
    rx: Option<Receiver<Telemetry>>,

    telemetry: Vec<Telemetry>,

    // TODO: switch this for showing the last X seconds of telemetry
    #[builder(default = "40")]
    main_graph_len: usize,

    main_graph_shows: Graphable,
}

impl GroundStationGui {
    /// Receive any telemetry that is waiting on the incoming channel
    fn recv_telem(&mut self) {
        // receive anything sent down the channel
        if let Some(rx) = &self.rx {
            loop {
                match rx.try_recv() {
                    Ok(telem) => {
                        tracing::debug!("{:?}", telem);
                        self.telemetry.push(telem);
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        tracing::warn!("Telemetry Receiver disconnected.");
                        // remove the reader so that it doesn't try to read from the disconnected channel
                        self.rx = None;
                        break;
                    }
                    _ => break,
                }
            }
        }
    }
}

// TODO: add view for all graphs
// TODO: add a table view for all incoming data
// TODO: add statistics view (e.g. number of dropped packets)
impl eframe::App for GroundStationGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.recv_telem();

        egui::TopBottomPanel::top("title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🚀 Manchester CanSat Project");
            });
        });

        // handy for changing all the rows at once
        fn settings_row(
            ui: &mut egui::Ui,
            label_text: impl Into<WidgetText>,
            setting: impl FnOnce(&mut egui::Ui),
        ) {
            ui.horizontal(|ui| {
                ui.label(label_text);
                setting(ui);
            });
        }

        egui::SidePanel::left("settings").show(ctx, |ui| {
            ui.heading("Settings");
            settings_row(ui, "theme", egui::widgets::global_dark_light_mode_buttons);
            settings_row(ui, "main graph", |ui| {
                egui::ComboBox::from_id_source("main_graph")
                    .selected_text(format!("{}", self.main_graph_shows))
                    .show_ui(ui, |ui| {
                        for e in all::<Graphable>() {
                            ui.selectable_value(&mut self.main_graph_shows, e, format!("{e}"));
                        }
                    });
            });
            settings_row(ui, "graph points", |ui| {
                let max = usize::max(100, self.main_graph_len);
                ui.add(egui::Slider::new(&mut self.main_graph_len, 1..=max).clamp_to_range(false));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("{}", self.main_graph_shows));
            let to_skip = self.telemetry.len().saturating_sub(self.main_graph_len);
            let points: Vec<[f64; 2]> = self
                .telemetry
                .iter()
                .skip(to_skip)
                .map(|telem| {
                    [
                        telem.mission_time.as_seconds(),
                        self.main_graph_shows.extract_telemetry_value(telem),
                    ]
                })
                .collect();
            let line = Line::new(points);
            Plot::new("main_plot").show(ui, |plot_ui| plot_ui.line(line));
        });

        // we must request a repaint otherwise we do not receive any data
        ctx.request_repaint();
    }
}
