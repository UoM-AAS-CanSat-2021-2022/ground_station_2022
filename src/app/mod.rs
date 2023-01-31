mod graphable;

use graphable::Graphable;

use std::fmt;
use std::sync::mpsc::Receiver;

use derive_builder::Builder;
use eframe::egui;
use egui::plot::{Line, Plot};
use egui::{Grid, Ui, Widget, WidgetText};
use egui_extras::{Column, TableBuilder};
use enum_iterator::{all, Sequence};
use tracing_egui::LogPanel;

use crate::telemetry::{Telemetry, TelemetryField};

#[derive(Builder)]
#[builder(pattern = "owned", default)]
#[derive(Default)]
pub struct GroundStationGui {
    /// the receiving end of the channel
    #[builder(setter(strip_option))]
    rx: Option<Receiver<Telemetry>>,

    /// the collected telemetry from the current run
    telemetry: Vec<Telemetry>,

    /// the values for displaying in the data table
    /// MUST be kept up to date with `telemetry`
    data_table_values: Vec<Vec<String>>,

    // TODO: switch this for showing the last X seconds of telemetry
    #[builder(default = "40")]
    main_graph_len: usize,

    /// what does the main graph view show
    main_graph_shows: Graphable,

    /// what does the main view show
    main_view: MainPanelView,
}

#[derive(Sequence, Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum MainPanelView {
    #[default]
    AllGraphs,
    OneGraph,
    Table,
    Statistics,
    Log,
}

impl MainPanelView {
    fn as_str(&self) -> &'static str {
        match self {
            MainPanelView::OneGraph => "One Graph",
            MainPanelView::AllGraphs => "All Graphs",
            MainPanelView::Table => "Data Table",
            MainPanelView::Statistics => "Statistics",
            MainPanelView::Log => "Log View",
        }
    }
}

impl fmt::Display for MainPanelView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl GroundStationGui {
    /// Receive any telemetry that is waiting on the incoming channel
    fn recv_telem(&mut self) {
        // take ownership of the receiver so we can mutate self
        if let Some(rx) = self.rx.take() {
            // receive anything sent down the channel
            loop {
                match rx.try_recv() {
                    Ok(telem) => self.add_telem(telem),

                    // don't replace the reader if the receiver is disconnected
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        tracing::warn!("Telemetry Receiver disconnected.");
                        break;
                    }

                    // if the receiver has no more telemetry then give
                    // ownership of the receiver back to self
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        self.rx = Some(rx);
                        break;
                    }
                }
            }
        }
    }

    // handles all the logic / state that must be kept in sync when adding telemetry
    fn add_telem(&mut self, telem: Telemetry) {
        tracing::debug!("{:?}", telem);
        self.telemetry.push(telem);
    }

    fn settings(&mut self, ui: &mut Ui) {
        // handy for changing all the rows at once
        fn settings_row(
            ui: &mut Ui,
            label_text: impl Into<WidgetText>,
            setting: impl FnOnce(&mut Ui),
        ) {
            ui.horizontal(|ui| {
                ui.label(label_text);
                setting(ui);
            });
        }

        ui.heading("Settings");
        settings_row(ui, "theme", egui::widgets::global_dark_light_mode_buttons);
        settings_row(ui, "main view shows", |ui| {
            egui::ComboBox::from_id_source("main_graph")
                .selected_text(self.main_view.as_str())
                .show_ui(ui, |ui| {
                    for e in all::<MainPanelView>() {
                        ui.selectable_value(&mut self.main_view, e, e.as_str());
                    }
                });
        });
        settings_row(ui, "one graph shows", |ui| {
            egui::ComboBox::from_id_source("one_graph")
                .selected_text(self.main_graph_shows.as_str())
                .show_ui(ui, |ui| {
                    for e in all::<Graphable>() {
                        ui.selectable_value(&mut self.main_graph_shows, e, e.as_str());
                    }
                });
        });
        settings_row(ui, "graph points", |ui| {
            let max = usize::max(100, self.main_graph_len);
            ui.add(egui::Slider::new(&mut self.main_graph_len, 1..=max).clamp_to_range(false));
        });
    }

    fn graph(&mut self, ui: &mut Ui, id_source: &str, field: Graphable) {
        let to_skip = self.telemetry.len().saturating_sub(self.main_graph_len);
        let points: Vec<[f64; 2]> = self
            .telemetry
            .iter()
            .skip(to_skip)
            .map(|telem| {
                [
                    telem.mission_time.as_seconds(),
                    field.extract_telemetry_value(telem),
                ]
            })
            .collect();
        let line = Line::new(points);
        Plot::new(id_source).show(ui, |plot_ui| plot_ui.line(line));
    }

    fn one_graph_view(&mut self, ui: &mut Ui) {
        ui.heading(format!("Main graph showing: {}", self.main_graph_shows));
        self.graph(ui, "main_plot", self.main_graph_shows);
    }

    fn all_graphs_view(&mut self, ui: &mut Ui) {
        let avail_height = ui.available_height();
        let avail_width = ui.available_height();
        Grid::new("all_graphs")
            .min_col_width(avail_width / 6.0)
            .min_row_height(avail_height / 2.5)
            .show(ui, |ui| {
                for (i, field) in all::<Graphable>().enumerate() {
                    ui.vertical_centered_justified(|ui| {
                        self.graph(ui, field.as_str(), field);
                    });
                    if i == 4 || i == 9 {
                        ui.end_row();
                    }
                }
            });
    }

    fn data_table_view(&mut self, ui: &mut Ui) {
        const ROW_HEIGHT: f32 = 20.0;
        const COL_WIDTH_MULT: f32 = 13.0;

        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .max_height(f32::INFINITY)
            .show(ui, |ui| {
                let mut builder = TableBuilder::new(ui).striped(true).stick_to_bottom(true);

                for field in all::<TelemetryField>() {
                    let min_width = field.as_str().len() as f32 * COL_WIDTH_MULT;
                    builder = builder.column(
                        Column::initial(min_width)
                            .at_least(min_width)
                            .resizable(true),
                    );
                }

                builder
                    .auto_shrink([false, false])
                    .max_scroll_height(f32::INFINITY)
                    .header(ROW_HEIGHT + 5.0, |mut header| {
                        for field in all::<TelemetryField>() {
                            header.col(|ui| {
                                ui.heading(field.as_str());
                            });
                        }
                    })
                    .body(|body| {
                        body.rows(ROW_HEIGHT, self.telemetry.len(), |row_index, mut row| {
                            let telem = &self.telemetry[row_index];

                            for field in all::<TelemetryField>() {
                                row.col(|ui| {
                                    ui.label(telem.get_field(field));
                                });
                            }
                        });
                    });
            });
    }

    fn stats_view(&mut self, ui: &mut Ui) {
        ui.heading("Statistics view");
    }

    fn log_view(&mut self, ui: &mut Ui) {
        LogPanel.ui(ui);
    }
}

// TODO: add view for all graphs
// TODO: add statistics view (e.g. number of dropped packets)
// TODO: eventually use toasts for notifications https://github.com/ItsEthra/egui-notify
//       this also looks pretty cool :) https://github.com/n00kii/egui-modal
impl eframe::App for GroundStationGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.recv_telem();

        egui::TopBottomPanel::top("title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🚀 Manchester CanSat Project");
            });
        });

        egui::Window::new("settings").show(ctx, |ui| {
            self.settings(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // match on the current view to decide what to draw
            match self.main_view {
                MainPanelView::OneGraph => self.one_graph_view(ui),
                MainPanelView::AllGraphs => self.all_graphs_view(ui),
                MainPanelView::Table => self.data_table_view(ui),
                MainPanelView::Statistics => self.stats_view(ui),
                MainPanelView::Log => self.log_view(ui),
            }
        });

        // we must request a repaint otherwise we do not receive any data
        ctx.request_repaint();
    }
}
