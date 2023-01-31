mod action;
mod sim_mode;
mod state;
mod time;

use action::Action;
use egui::{Ui, WidgetText};
use sim_mode::SimMode;
use state::State;
use std::fmt::Display;
use time::Time;

use crate::as_str::AsStr;
use enum_iterator::{all, Sequence};

#[derive(Default)]
pub struct CommandPanel {
    curr_command: Command,

    // state
    time: Time,
    sim_state: SimMode,
    sim_pressure: Pascals,
    action: Action,
    state: State,
    custom_cmd: String,
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
            egui::SidePanel::right(format!("right_{id_source}")).show_inside(ui, |ui| {
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

    pub fn show(&mut self, ui: &mut Ui) {
        Self::combobox_row(ui, &mut self.curr_command, "Command:", "command_combobox");

        match self.curr_command {
            Command::SetTime => self.set_time_view(ui),
            _ => {
                ui.heading(self.curr_command.as_str());
            }
        };
    }

    fn set_time_view(&mut self, ui: &mut Ui) {
        Self::combobox_row(ui, &mut self.time, "Argument: ", "time_combobox");
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
