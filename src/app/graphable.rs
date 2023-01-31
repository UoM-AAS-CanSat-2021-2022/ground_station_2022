use crate::as_str::AsStr;
use crate::telemetry::Telemetry;
use enum_iterator::Sequence;
use std::fmt;

/// Enum represents all of the telemetry which is graphable
#[derive(Sequence, Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Graphable {
    /// PACKET_COUNT telemetry field
    #[default]
    PacketCount,

    /// ALTITUDE telemetry field
    Altitude,

    /// TEMPERATURE telemetry field
    Temperature,

    /// VOLTAGE telemetry field
    Voltage,

    /// GPS_ALTITUDE telemetry field
    GpsAltitude,

    /// GPS_LATITUDE telemetry field
    GpsLatitude,

    /// GPS_LONGITUDE telemetry field
    GpsLogitude,

    /// GPS_SATS telemetry field
    GpsSats,

    /// TILT_X telemetry field
    TiltX,

    /// TILT_Y telemetry field
    TiltY,
}

impl AsStr for Graphable {
    #[rustfmt::skip]
    fn as_str(&self) -> &'static str {
        match self {
            Graphable::PacketCount => "packet_count",
            Graphable::Altitude => "altitude",
            Graphable::Temperature => "temperature",
            Graphable::Voltage => "voltage",
            Graphable::GpsAltitude => "gps_altitude",
            Graphable::GpsLatitude => "gps_latitude",
            Graphable::GpsLogitude => "gps_logitude",
            Graphable::GpsSats => "gps_sats",
            Graphable::TiltX => "tilt_x",
            Graphable::TiltY => "tilt_y",
        }
    }
}

impl Graphable {
    #[rustfmt::skip]
    pub fn extract_telemetry_value(&self, telem: &Telemetry) -> f64 {
        match self {
            Graphable::PacketCount => telem.packet_count as f64,
            Graphable::Altitude    => telem.altitude,
            Graphable::Temperature => telem.temperature,
            Graphable::Voltage     => telem.voltage,
            Graphable::GpsAltitude => telem.gps_altitude,
            Graphable::GpsLatitude => telem.gps_latitude,
            Graphable::GpsLogitude => telem.gps_longitude,
            Graphable::GpsSats     => telem.gps_sats as f64,
            Graphable::TiltX       => telem.tilt_x,
            Graphable::TiltY       => telem.tilt_y,
        }
    }
}

impl fmt::Display for Graphable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
