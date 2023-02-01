mod action;
mod sim_mode;
mod state;
mod time;

use action::Action;
use chrono::Timelike;
use eframe::emath::Align;
use egui::{DragValue, Layout, Ui, WidgetText};
use sim_mode::SimMode;
use state::Target;
use std::fmt::Display;
use time::Time;

use crate::app::commands::state::{ContainerState, PayloadState};
use crate::as_str::AsStr;
use crate::telemetry::GpsTime;
use crate::TEAM_ID;
use enum_iterator::{all, Sequence};

/// Holds all the state related to sending commands / the command UI
pub struct CommandPanel {
    curr_command: Command,
    time: Time,
    manual_time: GpsTime,
    sim_state: SimMode,
    sim_pressure: Pascals,
    action: Action,
    setstate_target: Target,
    container_state: ContainerState,
    payload_state: PayloadState,
    custom_cmd: String,
}

impl Default for CommandPanel {
    fn default() -> Self {
        let utc = chrono::Utc::now();
        Self {
            curr_command: Default::default(),
            time: Default::default(),
            // default to the current UTC time
            manual_time: GpsTime {
                h: utc.hour() as u8,
                m: utc.minute() as u8,
                s: utc.second() as u8,
            },
            sim_state: Default::default(),
            // sea level pressure in pascals
            sim_pressure: 101325,
            action: Default::default(),
            setstate_target: Default::default(),
            container_state: Default::default(),
            payload_state: Default::default(),
            // simplest full command, should be nicer to edit from
            custom_cmd: format!("CMD,{TEAM_ID},CAL"),
        }
    }
}

impl CommandPanel {
    fn combobox_row<CONTENT>(
        ui: &mut Ui,
        state: &mut CONTENT,
        label: impl Into<WidgetText>,
        id_source: impl std::hash::Hash + Display,
    ) where
        CONTENT: Sequence + AsStr + PartialEq + Copy,
    {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                egui::ComboBox::from_id_source(id_source)
                    .selected_text(state.as_str())
                    .show_ui(ui, |ui| {
                        for c in all() {
                            ui.selectable_value(state, c, c.as_str());
                        }
                    });
            });
        });
    }

    fn build_cmd(&self) -> String {
        match self.curr_command {
            Command::SetTime => match self.time {
                Time::Manual => format!("CMD,{TEAM_ID},ST,{}", self.manual_time),
                Time::CurrUtc => {
                    let utc = chrono::Utc::now();
                    format!(
                        "CMD,{TEAM_ID},ST,{:02}:{:02}:{:02}",
                        utc.hour(),
                        utc.minute(),
                        utc.second()
                    )
                }
                Time::Gps => format!("CMD,{TEAM_ID},ST,GPS"),
            },
            Command::SimulationMode => format!("CMD,{TEAM_ID},SIM,{}", self.sim_state),
            Command::SimulatedPressure => format!("CMD,{TEAM_ID},SIMP,{}", self.sim_pressure),
            Command::Calibrate => format!("CMD,{TEAM_ID},CAL"),
            Command::Action => format!("CMD,{TEAM_ID},OPTIONAL,ACTION,{}", self.action),
            Command::SetState => match self.setstate_target {
                Target::Payload => format!(
                    "CMD,{TEAM_ID},OPTIONAL,SETSTATE,{},{}",
                    self.setstate_target, self.payload_state
                ),
                Target::Container => format!(
                    "CMD,{TEAM_ID},OPTIONAL,SETSTATE,{},{}",
                    self.setstate_target, self.container_state
                ),
            },
            Command::Custom => self.custom_cmd.clone(),
        }
    }

    fn send_packet(&mut self) {
        let cmd = self.build_cmd();
        tracing::info!("Sending command: {cmd}");
        // TODO: actually send the packet lol
    }

    pub fn show(&mut self, ui: &mut Ui) {
        Self::combobox_row(ui, &mut self.curr_command, "Command:", "command_combobox");

        match self.curr_command {
            Command::SetTime => self.set_time_view(ui),
            Command::SimulationMode => self.simulation_mode_view(ui),
            Command::SimulatedPressure => self.simulation_pressure_view(ui),
            Command::Calibrate => (),
            Command::Action => self.action_view(ui),
            Command::SetState => self.setstate_view(ui),
            Command::Custom => self.custom_view(ui),
        };

        ui.separator();
        ui.vertical_centered(|ui| {
            ui.label(self.build_cmd());
            ui.separator();
            if ui.button("Send").clicked() {
                self.send_packet();
            }
        });
    }

    fn set_time_view(&mut self, ui: &mut Ui) {
        Self::combobox_row(ui, &mut self.time, "Argument:", "time_combobox");

        // if its manual time, draw time picker kind of thing
        if self.time == Time::Manual {
            ui.horizontal(|ui| {
                ui.label("H:");
                ui.add(DragValue::new(&mut self.manual_time.h).clamp_range(0..=23));
                ui.label("M:");
                ui.add(DragValue::new(&mut self.manual_time.m).clamp_range(0..=59));
                ui.label("S:");
                ui.add(DragValue::new(&mut self.manual_time.s).clamp_range(0..=59));
            });
        }
    }

    fn simulation_mode_view(&mut self, ui: &mut Ui) {
        Self::combobox_row(
            ui,
            &mut self.sim_state,
            "Simulation Mode",
            "sim_mode_picker",
        );
    }

    fn simulation_pressure_view(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Simulated Pressure:");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add(
                    DragValue::new(&mut self.sim_pressure)
                        .clamp_range(0..=200_000)
                        .speed(10.0),
                );
            });
        });
    }

    fn action_view(&mut self, ui: &mut Ui) {
        Self::combobox_row(ui, &mut self.action, "Action:", "action_combobox");
    }

    fn setstate_view(&mut self, ui: &mut Ui) {
        Self::combobox_row(ui, &mut self.setstate_target, "Target:", "state_combobox");
        match self.setstate_target {
            Target::Payload => {
                Self::combobox_row(
                    ui,
                    &mut self.payload_state,
                    "Payload State:",
                    "payload_state_combobox",
                );
            }
            Target::Container => {
                Self::combobox_row(
                    ui,
                    &mut self.container_state,
                    "Container State:",
                    "container_state_combobox",
                );
            }
        }
    }

    fn custom_view(&mut self, ui: &mut Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.text_edit_singleline(&mut self.custom_cmd);
        });
    }
}

type Pascals = u32;

#[derive(Sequence, Default, Debug, Copy, Clone, Eq, PartialEq)]
enum Command {
    /// ST - Set time command, can be any of: manual, current_utc, gps
    #[default]
    SetTime,

    /// SIM - Simulation mode command, can be any one of: enable, disable, activate
    SimulationMode,

    /// SIMP - Simulated pressure command, can be any valid value in pascals
    SimulatedPressure,

    /// CAl - calibrate the sensors and reset the EEPROM
    Calibrate,

    /// ACTION - force the container / payload to perform a certian action
    Action,

    /// SETSTATE - forcibly change the payload/container FSM state
    SetState,

    // Custom - allow the user to send anything to the CanSat
    Custom,
}

impl AsStr for Command {
    fn as_str(&self) -> &'static str {
        match self {
            Command::SetTime => "Set Time",
            Command::SimulationMode => "Simulation Mode",
            Command::SimulatedPressure => "Simulated Pressure",
            Command::Calibrate => "Calibrate",
            Command::Action => "Action",
            Command::SetState => "Set State",
            Command::Custom => "Custom",
        }
    }
}
