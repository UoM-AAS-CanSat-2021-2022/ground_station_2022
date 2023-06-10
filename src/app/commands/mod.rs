mod action;
mod enabled;
mod hold_release;
mod open_close;
mod raise_stop;
mod sim_mode;
mod state;
mod time;

use action::Action;
use chrono::Timelike;
use eframe::emath::Align;
use egui::{DragValue, Layout, Ui, WidgetText};
use egui_notify::Toasts;
use sim_mode::SimMode;
use state::Target;
use std::fmt::Display;
use time::Time;

use crate::app::commands::hold_release::HoldRelease;
use crate::app::commands::open_close::OpenClose;
use crate::app::commands::raise_stop::RaiseStop;
use crate::constants::SEALEVEL_PA;
use crate::{
    app::commands::{
        enabled::Enabled,
        state::{ContainerState, PayloadState},
    },
    as_str::AsStr,
    constants::TEAM_ID,
    telemetry::GpsTime,
};
use enum_iterator::{all, Sequence};

/// Holds all the state related to sending commands / the command UI
pub struct CommandPanel {
    curr_command: Command,
    telem_enable: Enabled,
    time: Time,
    manual_time: GpsTime,
    sim_state: SimMode,
    sim_pressure: Pascals,
    action: Action,
    setstate_target: Target,
    container_state: ContainerState,
    payload_state: PayloadState,
    cam_enable: Enabled,
    sound_enable: Enabled,
    flaps: OpenClose,
    heat_shield: OpenClose,
    parachute: OpenClose,
    flag: RaiseStop,
    probe: HoldRelease,
    custom_cmd: String,
}

impl Default for CommandPanel {
    fn default() -> Self {
        let utc = chrono::Utc::now();
        Self {
            curr_command: Default::default(),
            telem_enable: Default::default(),
            time: Default::default(),
            // default to the current UTC time
            manual_time: GpsTime {
                h: utc.hour() as u8,
                m: utc.minute() as u8,
                s: utc.second() as u8,
            },
            sim_state: Default::default(),
            sim_pressure: SEALEVEL_PA,
            action: Default::default(),
            setstate_target: Default::default(),
            container_state: Default::default(),
            payload_state: Default::default(),
            cam_enable: Default::default(),
            sound_enable: Default::default(),
            flaps: Default::default(),
            heat_shield: Default::default(),
            parachute: Default::default(),
            probe: Default::default(),
            flag: Default::default(),
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
                    .width(150.0)
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
            Command::TelemetryEnable => format!("CMD,{TEAM_ID},CX,{}", self.telem_enable),
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
            Command::Reset => format!("CMD,{TEAM_ID},OPTIONAL,RESET"),
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
            Command::SoundEnable => format!("CMD,{TEAM_ID},OPTIONAL,SOUND.{}", self.telem_enable),
            Command::CamEnable => format!("CMD,{TEAM_ID},OPTIONAL,CAM.{}", self.telem_enable),
            Command::Flaps => format!("CMD,{TEAM_ID},OPTIONAL,FLAP.{}", self.flaps),
            Command::HeatShield => format!("CMD,{TEAM_ID},OPTIONAL,HS.{}", self.heat_shield),
            Command::Parachute => format!("CMD,{TEAM_ID},OPTIONAL,CHUTE.{}", self.parachute),
            Command::Probe => format!("CMD,{TEAM_ID},OPTIONAL,PROBE.{}", self.probe),
            Command::Flag => format!("CMD,{TEAM_ID},OPTIONAL,FLAG.{}", self.flag),
            Command::Custom => self.custom_cmd.clone(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui, notif: &mut Toasts) -> Option<String> {
        Self::combobox_row(ui, &mut self.curr_command, "Command:", "command_combobox");

        match self.curr_command {
            Command::TelemetryEnable => self.telemetry_enable_view(ui),
            Command::SetTime => self.set_time_view(ui),
            Command::SimulationMode => self.simulation_mode_view(ui),
            Command::SimulatedPressure => self.simulation_pressure_view(ui),
            Command::Calibrate | Command::Reset => (),
            Command::Action => self.action_view(ui),
            Command::SetState => self.setstate_view(ui),
            Command::CamEnable => self.cam_enable_view(ui),
            Command::SoundEnable => self.sound_enable_view(ui),
            Command::Flaps => self.flaps_view(ui),
            Command::HeatShield => self.heat_shield_view(ui),
            Command::Parachute => self.parachute_view(ui),
            Command::Flag => self.flag_view(ui),
            Command::Probe => self.probe_view(ui),
            Command::Custom => self.custom_view(ui),
        };

        ui.separator();
        ui.vertical_centered(|ui| {
            ui.label(self.build_cmd());
            ui.separator();
            if ui.button("Send").clicked() {
                let cmd = self.build_cmd();
                notif.info(format!("Sent: {cmd}"));
                return Some(cmd);
            }
            None
        })
        .inner
    }

    fn telemetry_enable_view(&mut self, ui: &mut Ui) {
        Self::combobox_row(ui, &mut self.telem_enable, "Enable:", "cx_combobox");
    }

    fn sound_enable_view(&mut self, ui: &mut Ui) {
        Self::combobox_row(ui, &mut self.sound_enable, "Enable:", "cx_combobox");
    }

    fn cam_enable_view(&mut self, ui: &mut Ui) {
        Self::combobox_row(ui, &mut self.cam_enable, "Enable:", "cx_combobox");
    }

    fn flaps_view(&mut self, ui: &mut Ui) {
        Self::combobox_row(ui, &mut self.flaps, "Open:", "cx_combobox");
    }

    fn heat_shield_view(&mut self, ui: &mut Ui) {
        Self::combobox_row(ui, &mut self.heat_shield, "Open:", "cx_combobox");
    }

    fn parachute_view(&mut self, ui: &mut Ui) {
        Self::combobox_row(ui, &mut self.parachute, "Open:", "cx_combobox");
    }

    fn flag_view(&mut self, ui: &mut Ui) {
        Self::combobox_row(ui, &mut self.flag, "Raise:", "cx_combobox");
    }

    fn probe_view(&mut self, ui: &mut Ui) {
        Self::combobox_row(ui, &mut self.probe, "Detatch:", "cx_combobox");
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
            "Simulation Mode:",
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
    /// CX - Enable/Disable container telemetry
    TelemetryEnable,

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

    /// RESET
    Reset,

    /// CAM.ON/OFF
    CamEnable,

    /// SOUND.ON/OFF
    SoundEnable,

    /// FLAP.OPEN/CLOSE
    Flaps,

    /// HS.OPEN/CLOSE
    HeatShield,

    /// CHUTE.OPEN/CLOSE
    Parachute,

    /// FLAG.RAISE/STOP
    Flag,

    /// PROBE.HOLD/RELEASE
    Probe,

    // Custom - allow the user to send anything to the CanSat
    Custom,
}

impl AsStr for Command {
    fn as_str(&self) -> &'static str {
        match self {
            Command::TelemetryEnable => "Telemetry Enable",
            Command::SetTime => "Set Time",
            Command::SimulationMode => "Simulation Mode",
            Command::SimulatedPressure => "Simulated Pressure",
            Command::Calibrate => "Calibrate",
            Command::Action => "Action",
            Command::SetState => "Set State",
            Command::Reset => "Reset",
            Command::CamEnable => "Camera",
            Command::SoundEnable => "Buzzer",
            Command::Flaps => "Container Door",
            Command::HeatShield => "Heat Shield",
            Command::Parachute => "Parachute",
            Command::Probe => "Payload Release",
            Command::Flag => "Flag",
            Command::Custom => "Custom",
        }
    }
}
