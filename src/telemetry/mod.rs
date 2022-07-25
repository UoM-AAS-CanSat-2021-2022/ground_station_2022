use parse_display::{Display, FromStr};

mod mode;
mod packet_type;
mod software_state;
mod timestamp;
mod tp_released;
mod tp_software_state;

pub use mode::Mode;
pub use packet_type::PacketType;
pub use software_state::SoftwareState;
pub use timestamp::Timestamp;
pub use tp_released::TpReleased;
pub use tp_software_state::TpSoftwareState;

#[derive(Clone, Debug, PartialEq)]
pub enum Telemetry {
    Container(ContainerTelemetry),
    Payload(PayloadTelemetry),
}

#[derive(Display, FromStr, Clone, Debug, PartialEq)]
#[display(
"{team_id},{timestamp},{packet_no},{packet_type},{mode},{tp_released},\
    {altitude:.1},{temp:.1},{voltage:.2},{gps_time},{gps_latitude:.4},{gps_longitude:.4},\
    {gps_altitude:.4},{gps_sats},{software_state},{cmd_echo}"
)]
pub struct ContainerTelemetry {
    pub team_id: u16,
    pub timestamp: Timestamp,
    pub packet_no: u32,
    pub packet_type: PacketType,
    pub mode: Mode,
    pub tp_released: TpReleased,
    pub altitude: f32,
    pub temp: f32,
    pub voltage: f32,
    pub gps_time: Timestamp,
    pub gps_latitude: f32,
    pub gps_longitude: f32,
    pub gps_altitude: f32,
    pub gps_sats: u8,
    pub software_state: SoftwareState,
    pub cmd_echo: String,
}

#[derive(Display, FromStr, Clone, Debug, PartialEq)]
#[display(
"{team_id},{timestamp},{packet_no},{packet_type},{tp_altitude:.1},{tp_temp:.1},\
    {tp_voltage:.2},{gyro_r:.2},{gyro_p:.2},{gyro_y:.2},{accel_r:.2},{accel_p:.2},{accel_y:.2},\
    {mag_r:.2},{mag_p:.2},{mag_y:.2},{pointing_error:.1},{tp_software_state}"
)]
pub struct PayloadTelemetry {
    pub team_id: u16,
    pub timestamp: Timestamp,
    pub packet_no: u32,
    pub packet_type: PacketType,
    pub tp_altitude: f32,
    pub tp_temp: f32,
    pub tp_voltage: f32,
    pub gyro_r: f32,
    pub gyro_p: f32,
    pub gyro_y: f32,
    pub accel_r: f32,
    pub accel_p: f32,
    pub accel_y: f32,
    pub mag_r: f32,
    pub mag_p: f32,
    pub mag_y: f32,
    pub pointing_error: f32,
    pub tp_software_state: TpSoftwareState,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::io;

    #[test]
    fn test_container_telemetry_parse() {
        let s = "1057,00:00:00.00,0,C,F,R,250.0,20.0,5.0,00:00:00.00,0,0.0,0.0,3,DP1,CXON";
        let telem = s.parse::<ContainerTelemetry>();
        assert_eq!(
            telem,
            Ok(ContainerTelemetry {
                team_id: 1057,
                timestamp: Timestamp {
                    h: 0,
                    m: 0,
                    s: 0,
                    cs: 0
                },
                packet_no: 0,
                packet_type: PacketType::Container,
                mode: Mode::Flight,
                tp_released: TpReleased::Released,
                altitude: 250.0,
                temp: 20.0,
                voltage: 5.0,
                gps_time: Timestamp {
                    h: 0,
                    m: 0,
                    s: 0,
                    cs: 0
                },
                gps_latitude: 0.0,
                gps_longitude: 0.0,
                gps_altitude: 0.0,
                gps_sats: 3,
                software_state: SoftwareState::DescentPar1,
                cmd_echo: "CXON".to_string()
            })
        );
    }

    #[test]
    fn test_container_telemetry_parse_fmt_identical() {
        let s = "1057,00:00:00.00,0,C,F,R,250.0,20.0,5.01,00:00:00.00,\
                       0.0000,0.0000,0.0000,3,DP1,CXON";
        let telem = s.parse::<ContainerTelemetry>().unwrap();
        assert_eq!(format!("{}", telem), s.to_string());
    }

    #[test]
    fn test_payload_telemetry_parse() {
        let s = "1057,00:00:00.20,1,T,254.3,19.8,4.92,0.93,-0.46,0.89,-0.89,\
                       0.29,9.04,0.98,0.7,0.69,1.2,RELEASED";

        let telem = s.parse::<PayloadTelemetry>();
        assert_eq!(
            telem,
            Ok(PayloadTelemetry {
                team_id: 1057,
                timestamp: Timestamp {
                    h: 0,
                    m: 0,
                    s: 0,
                    cs: 20
                },
                packet_no: 1,
                packet_type: PacketType::TetheredPayload,
                tp_altitude: 254.3,
                tp_temp: 19.8,
                tp_voltage: 4.92,
                gyro_r: 0.93,
                gyro_p: -0.46,
                gyro_y: 0.89,
                accel_r: -0.89,
                accel_p: 0.29,
                accel_y: 9.04,
                mag_r: 0.98,
                mag_p: 0.7,
                mag_y: 0.69,
                pointing_error: 1.2,
                tp_software_state: TpSoftwareState::Released
            })
        )
    }

    #[test]
    fn test_payload_telemetry_parse_fmt_identical() {
        let s = "1057,00:00:00.20,1,T,254.3,19.8,4.92,0.93,-0.46,0.89,-0.89,\
                       0.29,9.04,0.98,0.70,0.69,1.2,RELEASED";

        let telem = s.parse::<PayloadTelemetry>().unwrap();
        assert_eq!(format!("{}", telem), s.to_string());
    }
}
