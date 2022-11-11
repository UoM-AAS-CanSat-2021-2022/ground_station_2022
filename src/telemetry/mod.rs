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

use parse_display::{Display, FromStr};

#[derive(Display, FromStr, Clone, Debug, PartialEq)]
#[display(
    "{team_id},{mission_time},{packet_count},{mode},{state},{altitude:.1},{hs_deployed},{pc_deployed},\
    {mast_raised},{temperature:.1},{voltage:.1},{gps_time},{gps_altitude:.1},\
    {gps_latitude:.4},{gps_longitude:.4},{gps_sats},{tilt_x:.2},{tilt_y:.2},{cmd_echo}"
)]
#[non_exhaustive]
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
    pub altitude: f32,

    /// HS_DEPLOYED: P = probe with heat shield is deployed, N otherwise
    pub hs_deployed: HsDeployed,

    /// PC_DEPLOYED: C = probe parachute deployed (200m), N otherwise
    pub pc_deployed: PcDeployed,

    /// MAST_RAISED: M = flag mast raised after landing N otherwise
    pub mast_raised: MastRaised,

    /// TEMPERATURE: the temperature in celsiubs with a resolution of 0.1 C
    pub temperature: f32,

    /// VOLTAGE: the voltage of the cansat power bus, with a resolution of 0.1 V
    pub voltage: f32,

    /// GPS_TIME: time from the GPS receiver, must be reported in UTC and have a resolution of a second
    pub gps_time: GpsTime,

    /// GPS_ALTITUDE: altitude from the GPS receiver, in metres above mean sea level, resolution 0.1m
    pub gps_altitude: f32,

    /// GPS_LATITUDE: latitude from the GPS receiver, in decimal degrees with a resolution of 0.0001 degrees North
    pub gps_latitude: f32,

    /// GPS_LONGITUDE: longitude from the GPS receiver, in decimal degrees with a resolution of 0.0001 degrees West
    pub gps_longitude: f32,

    /// GPS_SATS: the number of GPS satellites being tracked by the GPS receiver, must be an integer lol
    pub gps_sats: u8,

    /// TILT_X: angle of the CanSat X axes in degrees, with a resolution of 0.01 degrees.
    /// 0 degrees is defined as when the axes are perpendicular to the Z axes,
    /// which is defined as towards the centre of gravity of the earth.
    pub tilt_x: f32,

    /// TILT_Y: angle of the CanSat Y axes in degrees, with a resolution of 0.01 degrees.
    /// 0 degrees is defined as when the axes are perpendicular to the Z axes,
    /// which is defined as towards the centre of gravity of the earth.
    pub tilt_y: f32,

    /// CMD_ECHO: the last command received by the CanSat, e.g. CXON or SP101325.
    pub cmd_echo: String,
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
