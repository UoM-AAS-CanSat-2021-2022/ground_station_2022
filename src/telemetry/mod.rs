mod gps_time;
mod hs_deployed;
mod mast_raised;
mod mission_time;
mod mode;
mod pc_deployed;
mod state;

pub use gps_time::GpsTime;
pub use hs_deployed::HsDeployed;
pub use mast_raised::MastRaised;
pub use mission_time::MissionTime;
pub use mode::Mode;
pub use pc_deployed::PcDeployed;
pub use state::State;

use crate::as_str::AsStr;
use enum_iterator::Sequence;
use parse_display::{Display, FromStr};
use std::fmt;

#[derive(Display, FromStr, Clone, Debug, PartialEq)]
#[display(
    "{team_id},{mission_time},{packet_count},{mode},{state},{altitude:.1},{hs_deployed},{pc_deployed},\
    {mast_raised},{temperature:.1},{voltage:.1},{gps_time},{gps_altitude:.1},\
    {gps_latitude:.4},{gps_longitude:.4},{gps_sats},{tilt_x:.2},{tilt_y:.2},{cmd_echo}"
)]
pub struct Telemetry {
    /// TEAM_ID: four digit team identification number
    pub team_id: u16,

    /// MISSION_TIME: UTC time - hh:mm:ss.ss
    pub mission_time: MissionTime,

    /// PACKET_COUNT: count of transmitted packets, must be maintained through processor resets - EEPROM.
    pub packet_count: u32,

    /// MODE: F for flight, S for simulation
    pub mode: Mode,

    /// STATE: the operating state of the software
    pub state: State,

    /// ALTITUDE: height in metres relative to the launch site, resolution of 0.1m.
    pub altitude: f64,

    /// HS_DEPLOYED: P = probe with heat shield is deployed, N otherwise
    pub hs_deployed: HsDeployed,

    /// PC_DEPLOYED: C = probe parachute deployed (200m), N otherwise
    pub pc_deployed: PcDeployed,

    /// MAST_RAISED: M = flag mast raised after landing N otherwise
    pub mast_raised: MastRaised,

    /// TEMPERATURE: the temperature in celsiubs with a resolution of 0.1 C
    pub temperature: f64,

    /// VOLTAGE: the voltage of the cansat power bus, with a resolution of 0.1 V
    pub voltage: f64,

    /// GPS_TIME: time from the GPS receiver, must be reported in UTC and have a resolution of a second
    pub gps_time: GpsTime,

    /// GPS_ALTITUDE: altitude from the GPS receiver, in metres above mean sea level, resolution 0.1m
    pub gps_altitude: f64,

    /// GPS_LATITUDE: latitude from the GPS receiver, in decimal degrees with a resolution of 0.0001 degrees North
    pub gps_latitude: f64,

    /// GPS_LONGITUDE: longitude from the GPS receiver, in decimal degrees with a resolution of 0.0001 degrees West
    pub gps_longitude: f64,

    /// GPS_SATS: the number of GPS satellites being tracked by the GPS receiver, must be an integer lol
    pub gps_sats: u8,

    /// TILT_X: angle of the CanSat X axes in degrees, with a resolution of 0.01 degrees.
    /// 0 degrees is defined as when the axes are perpendicular to the Z axes,
    /// which is defined as towards the centre of gravity of the earth.
    pub tilt_x: f64,

    /// TILT_Y: angle of the CanSat Y axes in degrees, with a resolution of 0.01 degrees.
    /// 0 degrees is defined as when the axes are perpendicular to the Z axes,
    /// which is defined as towards the centre of gravity of the earth.
    pub tilt_y: f64,

    /// CMD_ECHO: the last command received by the CanSat, e.g. CXON or SP101325.
    pub cmd_echo: String,
}

impl Telemetry {
    #[rustfmt::skip]
    #[allow(clippy::useless_format)]
    pub fn get_field(&self, field: TelemetryField) -> String {
        match field {
            TelemetryField::TeamId       => format!("{}", self.team_id),
            TelemetryField::MissionTime  => format!("{}", self.mission_time),
            TelemetryField::PacketCount  => format!("{}", self.packet_count),
            TelemetryField::Mode         => format!("{}", self.mode),
            TelemetryField::State        => format!("{}", self.state),
            TelemetryField::Altitude     => format!("{}", self.altitude),
            TelemetryField::HsDeployed   => format!("{}", self.hs_deployed),
            TelemetryField::PcDeployed   => format!("{}", self.pc_deployed),
            TelemetryField::MastRaised   => format!("{}", self.mast_raised),
            TelemetryField::Temperature  => format!("{}", self.temperature),
            TelemetryField::Voltage      => format!("{}", self.voltage),
            TelemetryField::GpsTime      => format!("{}", self.gps_time),
            TelemetryField::GpsAltitude  => format!("{}", self.gps_altitude),
            TelemetryField::GpsLatitude  => format!("{}", self.gps_latitude),
            TelemetryField::GpsLongitude => format!("{}", self.gps_longitude),
            TelemetryField::GpsSats      => format!("{}", self.gps_sats),
            TelemetryField::TiltX        => format!("{}", self.tilt_x),
            TelemetryField::TiltY        => format!("{}", self.tilt_y),
            TelemetryField::CmdEcho      => format!("{}", self.cmd_echo),
        }
    }
}

#[derive(Sequence, Debug, Copy, Clone, Eq, PartialEq)]
pub enum TelemetryField {
    TeamId,
    MissionTime,
    PacketCount,
    Mode,
    State,
    Altitude,
    HsDeployed,
    PcDeployed,
    MastRaised,
    Temperature,
    Voltage,
    GpsTime,
    GpsAltitude,
    GpsLatitude,
    GpsLongitude,
    GpsSats,
    TiltX,
    TiltY,
    CmdEcho,
}

impl AsStr for TelemetryField {
    #[rustfmt::skip]
    fn as_str(&self) -> &'static str {
        match self {
            TelemetryField::TeamId       => "TEAM_ID",
            TelemetryField::MissionTime  => "MISSION_TIME",
            TelemetryField::PacketCount  => "PACKET_COUNT",
            TelemetryField::Mode         => "MODE",
            TelemetryField::State        => "STATE",
            TelemetryField::Altitude     => "ALTITUDE",
            TelemetryField::HsDeployed   => "HS_DEPLOYED",
            TelemetryField::PcDeployed   => "PC_DEPLOYED",
            TelemetryField::MastRaised   => "MAST_RAISED",
            TelemetryField::Temperature  => "TEMPERATURE",
            TelemetryField::Voltage      => "VOLTAGE",
            TelemetryField::GpsTime      => "GPS_TIME",
            TelemetryField::GpsAltitude  => "GPS_ALTITUDE",
            TelemetryField::GpsLatitude  => "GPS_LATITUDE",
            TelemetryField::GpsLongitude => "GPS_LONGITUDE",
            TelemetryField::GpsSats      => "GPS_SATS",
            TelemetryField::TiltX        => "TILT_X",
            TelemetryField::TiltY        => "TILT_Y",
            TelemetryField::CmdEcho      => "CMD_ECHO",
        }
    }
}

impl fmt::Display for TelemetryField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_parse() {
        let s = "1047,15:12:02.99,123,F,YEETED,356.2,P,C,N,37.8,5.1,15:12:03,1623.3,37.2249,-80.4249,14,2.36,-5.49,CXON";
        let telem = s.parse::<Telemetry>();
        assert_eq!(
            telem,
            Ok(Telemetry {
                team_id: 1047,
                mission_time: MissionTime {
                    h: 15,
                    m: 12,
                    s: 2,
                    cs: 99
                },
                packet_count: 123,
                mode: Mode::Flight,
                state: State::Yeeted,
                altitude: 356.2,
                hs_deployed: HsDeployed::Deployed,
                pc_deployed: PcDeployed::Deployed,
                mast_raised: MastRaised::NotRaised,
                temperature: 37.8,
                voltage: 5.1,
                gps_time: GpsTime { h: 15, m: 12, s: 3 },
                gps_altitude: 1623.3,
                gps_latitude: 37.2249,
                gps_longitude: -80.4249,
                gps_sats: 14,
                tilt_x: 2.36,
                tilt_y: -5.49,
                cmd_echo: "CXON".to_string(),
            })
        );
    }

    #[test]
    fn test_telemetry_parse_fmt_identical() {
        let s = "1047,15:12:02.99,123,F,YEETED,356.2,P,C,N,37.8,5.1,15:12:03,1623.3,37.2249,-80.4249,14,2.36,-5.49,CXON";
        let telem = s.parse::<Telemetry>().unwrap();
        assert_eq!(format!("{}", telem), s.to_string());
    }
}
