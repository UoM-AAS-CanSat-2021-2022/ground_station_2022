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
    GpsLongitude,

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
            Graphable::PacketCount => "Packet Count",
            Graphable::Altitude => "Altitude",
            Graphable::Temperature => "Temperature",
            Graphable::Voltage => "Voltage",
            Graphable::GpsAltitude => "GPS Altitude",
            Graphable::GpsLatitude => "GPS Latitude",
            Graphable::GpsLongitude => "GPS Longitude",
            Graphable::GpsSats => "GPS Satellites",
            Graphable::TiltX => "Tilt - X axis",
            Graphable::TiltY => "Tilt - Y axis",
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
            Graphable::GpsLongitude => telem.gps_longitude,
            Graphable::GpsSats     => telem.gps_sats as f64,
            Graphable::TiltX       => telem.tilt_x,
            Graphable::TiltY       => telem.tilt_y,
        }
    }

    pub fn format_value(&self, value: f64) -> String {
        match self {
            Graphable::PacketCount => format!("{value}"),
            Graphable::Altitude => format!("{value}m"),
            Graphable::Temperature => format!("{value}°C"),
            Graphable::Voltage => format!("{value}V"),
            Graphable::GpsAltitude => format!("{value}m"),
            Graphable::GpsLatitude => format!("{value}°"),
            Graphable::GpsLongitude => format!("{value}°"),
            Graphable::GpsSats => format!("{value}"),
            Graphable::TiltX => format!("{value}°"),
            Graphable::TiltY => format!("{value}°"),
        }
    }
}

impl fmt::Display for Graphable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
