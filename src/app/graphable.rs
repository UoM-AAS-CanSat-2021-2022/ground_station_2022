use crate::telemetry::Telemetry;
use enum_iterator::Sequence;
use parse_display::Display;

/// Enum represents all of the telemetry which is graphable
#[derive(Display, Sequence, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Graphable {
    /// PACKET_COUNT telemetry field
    #[default]
    #[display("packet_count")]
    PacketCount,

    /// ALTITUDE telemetry field
    #[display("altitude")]
    Altitude,

    /// TEMPERATURE telemetry field
    #[display("temperature")]
    Temperature,

    /// VOLTAGE telemetry field
    #[display("voltage")]
    Voltage,

    /// GPS_ALTITUDE telemetry field
    #[display("gps_altitude")]
    GpsAltitude,

    /// GPS_LATITUDE telemetry field
    #[display("gps_latitude")]
    GpsLatitude,

    /// GPS_LONGITUDE telemetry field
    #[display("gps_logitude")]
    GpsLogitude,

    /// GPS_SATS telemetry field
    #[display("gps_sats")]
    GpsSats,

    /// TILT_X telemetry field
    #[display("tilt_x")]
    TiltX,

    /// TILT_Y telemetry field
    #[display("tilt_y")]
    TiltY,
}

impl Graphable {
    pub fn extract_telemetry_value(&self, telem: &Telemetry) -> f32 {
        match self {
            Graphable::PacketCount => telem.packet_count as f32,
            Graphable::Altitude => telem.altitude,
            Graphable::Temperature => telem.temperature,
            Graphable::Voltage => telem.voltage,
            Graphable::GpsAltitude => telem.gps_altitude,
            Graphable::GpsLatitude => telem.gps_latitude,
            Graphable::GpsLogitude => telem.gps_longitude,
            Graphable::GpsSats => telem.gps_sats as f32,
            Graphable::TiltX => telem.tilt_x,
            Graphable::TiltY => telem.tilt_y,
        }
    }
}
