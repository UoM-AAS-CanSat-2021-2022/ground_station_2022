use std::sync::mpsc::Receiver;
use std::unreachable;

use derive_builder::Builder;
use eframe::egui;
use egui::plot::{Line, Plot, Value, Values};
use egui::WidgetText;
use enum_iterator::{all, Sequence};
use parse_display::Display;

use crate::telemetry::{ContainerTelemetry, PayloadTelemetry, Telemetry};

#[derive(Builder)]
#[builder(pattern = "owned", default)]
#[derive(Default)]
pub struct GroundStationGui {
    #[builder(setter(strip_option))]
    rx: Option<Receiver<Telemetry>>,
    container_telem: Vec<ContainerTelemetry>,
    payload_telem: Vec<PayloadTelemetry>,
    #[builder(default = "40")]
    main_graph_len: usize,
    main_graph_shows: Graphable,
}

impl eframe::App for GroundStationGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // receive anything sent down the channel
        while let Some(telem) = self.rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            log::trace!("{:?}", telem);
            match telem {
                Telemetry::Container(telem) => self.container_telem.push(telem),
                Telemetry::Payload(telem) => self.payload_telem.push(telem),
            }
        }

        egui::TopBottomPanel::top("title_bar").show(ctx, |ui| {
            ui.heading("🚀 Manchester CanSat Project");
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
                            ui.selectable_value(&mut self.main_graph_shows, e, format!("{}", e));
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
            let line = Line::new(if self.main_graph_shows.is_container_member() {
                let to_skip = self
                    .container_telem
                    .len()
                    .saturating_sub(self.main_graph_len);
                Values::from_values_iter(self.container_telem.iter().skip(to_skip).cloned().map(
                    |telem| {
                        let y = match self.main_graph_shows {
                            Graphable::ContainerAltitude => telem.altitude,
                            Graphable::ContainerTemp => telem.temp,
                            Graphable::ContainerVoltage => telem.voltage,
                            Graphable::ContainerGpsLatitude => telem.gps_latitude,
                            Graphable::ContainerGpsLongitude => telem.gps_longitude,
                            Graphable::ContainerGpsAltitude => telem.gps_altitude,
                            Graphable::ContainerGpsSats => telem.gps_sats.into(),
                            _ => {
                                unreachable!("payload variants can never reach this branch")
                            }
                        };
                        Value::new(telem.timestamp.as_seconds(), y)
                    },
                ))
            } else {
                let to_skip = self.payload_telem.len().saturating_sub(self.main_graph_len);
                Values::from_values_iter(self.payload_telem.iter().skip(to_skip).cloned().map(
                    |telem| {
                        let y = match self.main_graph_shows {
                            Graphable::PayloadTpAltitude => telem.tp_altitude,
                            Graphable::PayloadTpTemp => telem.tp_temp,
                            Graphable::PayloadTpVoltage => telem.tp_voltage,
                            Graphable::PayloadGyroR => telem.gyro_r,
                            Graphable::PayloadGyroP => telem.gyro_p,
                            Graphable::PayloadGyroY => telem.gyro_y,
                            Graphable::PayloadAccelR => telem.accel_r,
                            Graphable::PayloadAccelP => telem.accel_p,
                            Graphable::PayloadAccelY => telem.accel_y,
                            Graphable::PayloadMagR => telem.mag_r,
                            Graphable::PayloadMagP => telem.mag_p,
                            Graphable::PayloadMagY => telem.mag_y,
                            Graphable::PayloadPointingError => telem.pointing_error,
                            _ => unreachable!("container variants can never reach this branch"),
                        };
                        Value::new(telem.timestamp.as_seconds(), y)
                    },
                ))
            });
            Plot::new("main_plot").show(ui, |plot_ui| plot_ui.line(line));
        });

        // we must request a repaint otherwise we do not receive any data
        ctx.request_repaint();
    }
}

/// Enum represents all of the telemetry which is graphable
#[derive(Display, Sequence, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Graphable {
    #[default]
    #[display("Container Altitude")]
    ContainerAltitude,
    #[display("Container Temperature")]
    ContainerTemp,
    #[display("Container Voltage")]
    ContainerVoltage,
    #[display("Container GPS Latitude")]
    ContainerGpsLatitude,
    #[display("Container GPS Longitude")]
    ContainerGpsLongitude,
    #[display("Container GPS Altitude")]
    ContainerGpsAltitude,
    #[display("Container GPS Satellites")]
    ContainerGpsSats,
    #[display("Payload Altitude")]
    PayloadTpAltitude,
    #[display("Payload Temperature")]
    PayloadTpTemp,
    #[display("Payload Voltage")]
    PayloadTpVoltage,
    #[display("Payload Gyroscope Roll")]
    PayloadGyroR,
    #[display("Payload Gyroscope Pitch")]
    PayloadGyroP,
    #[display("Payload Gyroscope Yaw")]
    PayloadGyroY,
    #[display("Payload Acceleration Roll")]
    PayloadAccelR,
    #[display("Payload Acceleration Pitch")]
    PayloadAccelP,
    #[display("Payload Acceleration Yaw")]
    PayloadAccelY,
    #[display("Payload Magnetometer Roll")]
    PayloadMagR,
    #[display("Payload Magnetometer Pitch")]
    PayloadMagP,
    #[display("Payload Magnetometer Yaw")]
    PayloadMagY,
    #[display("Payload Pointing Error")]
    PayloadPointingError,
}

impl Graphable {
    fn is_container_member(&self) -> bool {
        use Graphable::*;

        matches!(
            self,
            ContainerAltitude
                | ContainerTemp
                | ContainerVoltage
                | ContainerGpsLatitude
                | ContainerGpsLongitude
                | ContainerGpsAltitude
                | ContainerGpsSats
        )
    }
}
