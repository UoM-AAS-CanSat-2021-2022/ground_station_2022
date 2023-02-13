mod commands;
mod graphable;

use graphable::Graphable;

use std::collections::HashMap;
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc::Receiver;

use crate::app::commands::CommandPanel;
use crate::as_str::AsStr;
use derive_builder::Builder;
use eframe::{egui, emath::Align};
use egui::{
    plot::{Line, Plot},
    plot::{PlotPoint, PlotPoints},
    Color32, Grid, Layout, Ui,
};
use egui_extras::{Column, TableBuilder};
use enum_iterator::{all, Sequence};
use serialport::{SerialPort, SerialPortType};

use crate::telemetry::{Telemetry, TelemetryField};
use crate::xbee::BAUD_RATES;

const TELEMETRY_FILE: &'static str = "telemetry.csv";

#[derive(Builder)]
#[builder(pattern = "owned", default)]
#[derive(Default)]
pub struct GroundStationGui {
    /// the receiving end of the channel
    #[builder(setter(strip_option))]
    rx: Option<Receiver<Telemetry>>,

    /// the collected telemetry from the current run
    telemetry: Vec<Telemetry>,

    /// the values for displaying in the graphs
    graph_values: HashMap<Graphable, Vec<PlotPoint>>,

    /// how many telemetry points does the one graph view show?
    #[builder(default = "40")]
    one_graph_points: usize,

    /// how many telemetry points does the all graphs view show?
    #[builder(default = "40")]
    all_graphs_points: usize,

    /// show all the points in the one graph view?
    one_graph_shows_all: bool,

    /// do we show all the points in the all graphg view?
    all_graphs_show_all: bool,

    /// what does the main graph view show
    one_graph_shows: Graphable,

    /// what does the main view show
    main_view: MainPanelView,

    /// Show the settings window?
    show_settings_window: bool,

    /// Show the command window?
    show_command_window: bool,

    /// Show the radio window?
    show_radio_window: bool,

    /// The command center
    command_center: CommandPanel,

    /// The radio's serial port name
    radio_port: String,

    /// The radio's baud rate
    #[builder(default = "230400")]
    radio_baud: u32,

    /// The XBee radio serial port connection
    radio: Option<Box<dyn SerialPort>>,
}

// TODO: add a commands sent view
// TODO: add a packets view
#[derive(Sequence, Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum MainPanelView {
    #[default]
    AllGraphs,
    OneGraph,
    Table,
}

impl AsStr for MainPanelView {
    fn as_str(&self) -> &'static str {
        match self {
            MainPanelView::OneGraph => "One Graph",
            MainPanelView::AllGraphs => "All Graphs",
            MainPanelView::Table => "Data Table",
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
        self.telemetry.push(telem.clone());

        // save the telemetry to the graph points
        let time = telem.mission_time.as_seconds();
        for field in all::<Graphable>() {
            self.graph_values
                .entry(field)
                .or_default()
                .push(PlotPoint::new(time, field.extract_telemetry_value(&telem)));
        }

        // save the telemetry out to the telemetry file
        let handle = OpenOptions::new()
            .append(true)
            .create(true)
            .open(TELEMETRY_FILE);

        let result = match handle {
            Ok(mut file) => writeln!(file, "{telem}"),
            Err(e) => {
                tracing::warn!("Failed to open `{TELEMETRY_FILE}` - {e}.");
                Ok(())
            }
        };

        if let Err(e) = result {
            tracing::warn!("Encountered error while writing to file: {e}");
        }
    }

    fn open_radio_connection(&mut self) {
        // try to open the new radio
        match serialport::new(&self.radio_port, self.radio_baud).open() {
            Ok(port) => {
                self.radio = Some(port);
                tracing::info!("Successfully opened port.");
            }
            Err(e) => {
                self.radio = None;
                tracing::error!("Failed to open port - {e:?}");
            }
        }
    }
}

/// GUI components
impl GroundStationGui {
    fn graph(&mut self, ui: &mut Ui, id_source: &str, field: Graphable, to_show: usize) {
        let to_skip = self.telemetry.len().saturating_sub(to_show);
        let points: Vec<PlotPoint> = self
            .graph_values
            .entry(field)
            .or_default()
            .iter()
            .skip(to_skip)
            .copied()
            .collect();
        let line = Line::new(PlotPoints::Owned(points)).name(field.as_str());
        Plot::new(id_source).show(ui, |plot_ui| plot_ui.line(line));
    }

    fn one_graph_view(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Graph showing: ");
            egui::ComboBox::from_id_source("main_graph")
                .selected_text(self.one_graph_shows.as_str())
                .width(120.0)
                .wrap(false)
                .show_ui(ui, |ui| {
                    for e in all::<Graphable>() {
                        ui.selectable_value(&mut self.one_graph_shows, e, e.as_str());
                    }
                });

            ui.label("No. Points: ");
            ui.add_enabled_ui(!self.one_graph_shows_all, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.one_graph_points, 5..=100).clamp_to_range(false),
                );
            });

            ui.label("Show all: ");
            ui.add(egui::Checkbox::new(&mut self.one_graph_shows_all, ""));
        });

        let to_show = if self.one_graph_shows_all {
            usize::MAX
        } else {
            self.one_graph_points
        };
        self.graph(ui, "main_plot", self.one_graph_shows, to_show);
    }

    fn all_graphs_view(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("No. Points: ");
            ui.add_enabled_ui(!self.all_graphs_show_all, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.all_graphs_points, 5..=100).clamp_to_range(false),
                );
            });

            ui.label("Show all: ");
            ui.add(egui::Checkbox::new(&mut self.all_graphs_show_all, ""));
        });

        let to_show = if self.all_graphs_show_all {
            usize::MAX
        } else {
            self.all_graphs_points
        };
        let width = ui.available_width() / 5.0;
        let height = ui.available_height() / 2.0;

        Grid::new("all_graphs")
            .min_col_width(width)
            .max_col_width(width)
            .min_row_height(height)
            .spacing([5.0, 5.0])
            .show(ui, |ui| {
                for (i, field) in all::<Graphable>().enumerate() {
                    ui.vertical_centered(|ui| {
                        ui.heading(field.as_str());
                        self.graph(ui, field.as_str(), field, to_show);
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

    fn radio_window(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Serial port: ");
            ui.vertical_centered(|ui| {
                let Ok(ports) = serialport::available_ports() else {
                        ui.label("Failed to get availble ports.");
                        return;
                    };

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::ComboBox::from_id_source("radio_baud_combobox")
                        .selected_text(&self.radio_port)
                        .show_ui(ui, |ui| {
                            for port in ports {
                                if matches!(port.port_type, SerialPortType::UsbPort(_)) {
                                    let value = ui.selectable_value(
                                        &mut self.radio_port,
                                        port.port_name.clone(),
                                        &port.port_name,
                                    );

                                    if value.changed() {
                                        tracing::info!("Set radio port to {:?}", port.port_name);
                                    }
                                }
                            }
                        });
                });
            });
        });

        ui.horizontal(|ui| {
            ui.label("Baud rate: ");
            ui.vertical_centered(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::ComboBox::from_id_source("radio_port_combobox")
                        .selected_text(self.radio_baud.to_string())
                        .show_ui(ui, |ui| {
                            for baud in BAUD_RATES {
                                let value = ui.selectable_value(
                                    &mut self.radio_baud,
                                    baud,
                                    baud.to_string(),
                                );

                                if value.changed() {
                                    tracing::info!("Set radio baud to {baud}");
                                }
                            }
                        });
                });
            });
        });

        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            if ui.button("Open port").clicked() {
                self.open_radio_connection();
            }
        });

        ui.separator();

        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            if self.radio.is_some() {
                ui.colored_label(Color32::GREEN, "Connected");
            } else {
                ui.colored_label(Color32::RED, "Disconnected");
            }
        });
    }
}

// TODO: add view for controlling the radio
// TODO: add view for all graphs
// TODO: add changing the font size to the settings
// TODO: add statistics view (e.g. number of dropped packets)
// TODO: eventually use toasts for notifications https://github.com/ItsEthra/egui-notify
//       this also looks pretty cool :) https://github.com/n00kii/egui-modal
// TODO: add the telemetry file to the settings
// TODO: add clearing the current telemetry to the settings
// TODO: add a status indicator for whether we are still connected to the telemetry sender
// TODO: add a status window for replaying simulated pressure data (with pause + play?)
impl eframe::App for GroundStationGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.recv_telem();

        egui::TopBottomPanel::top("title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🚀 Manchester CanSat Project");
                ui.separator();

                egui::global_dark_light_mode_switch(ui);
                ui.separator();

                // main view buttons
                for view in all::<MainPanelView>() {
                    let label = ui.selectable_label(self.main_view == view, view.as_str());
                    if label.clicked() {
                        self.main_view = view;
                    }
                }

                // optional windows
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        // rightmost
                        ui.checkbox(&mut self.show_command_window, "🖧 Commands");
                        ui.checkbox(&mut self.show_radio_window, "📻 Radio");
                        ui.checkbox(&mut self.show_settings_window, "⚙ Settings");
                        // leftmost
                    });
                });
            });
        });

        // scuffed but yeah
        let mut open;
        if self.show_settings_window {
            open = true;
            egui::Window::new("settings")
                .open(&mut open)
                .show(ctx, |ui| {
                    ctx.settings_ui(ui);
                });
            self.show_settings_window = open;
        }

        if self.show_command_window {
            open = true;
            egui::Window::new("commands")
                .open(&mut open)
                .show(ctx, |ui| self.command_center.show(ui));
            self.show_radio_window = open;
        }

        if self.show_radio_window {
            open = true;
            egui::Window::new("radio")
                .open(&mut open)
                .show(ctx, |ui| self.radio_window(ui));
            self.show_radio_window = open;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // match on the current view to decide what to draw
            match self.main_view {
                MainPanelView::OneGraph => self.one_graph_view(ui),
                MainPanelView::AllGraphs => self.all_graphs_view(ui),
                MainPanelView::Table => self.data_table_view(ui),
            }
        });

        // we must request a repaint otherwise we do not receive any data
        ctx.request_repaint();
    }
}
