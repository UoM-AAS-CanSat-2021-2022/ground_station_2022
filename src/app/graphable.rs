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

    /// PRESSURE telemetry field
    Pressure,

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
            Graphable::Pressure => "Pressure",
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
            Graphable::PacketCount  => telem.packet_count as f64,
            Graphable::Altitude     => telem.altitude,
            Graphable::Temperature  => telem.temperature,
            Graphable::Voltage      => telem.voltage,
            Graphable::Pressure     => telem.pressure,
            Graphable::GpsAltitude  => telem.gps_altitude,
            Graphable::GpsLatitude  => telem.gps_latitude,
            Graphable::GpsLongitude => telem.gps_longitude,
            Graphable::GpsSats      => telem.gps_sats as f64,
            Graphable::TiltX        => telem.tilt_x,
            Graphable::TiltY        => telem.tilt_y,
        }
    }

    pub fn format_value(&self, value: f64) -> String {
        match self {
            Graphable::PacketCount => format!("{value:.0}"),
            Graphable::Altitude => format!("{value:.1}m"),
            Graphable::Temperature => format!("{value:.1}°C"),
            Graphable::Voltage => format!("{value:.1}V"),
            Graphable::Pressure => format!("{value:.1}kPa"),
            Graphable::GpsAltitude => format!("{value:.1}m"),
            Graphable::GpsLatitude => format!("{value:.4}°N"),
            Graphable::GpsLongitude => format!("{value:.4}°W"),
            Graphable::GpsSats => format!("{value:.0}"),
            Graphable::TiltX => format!("{value:.2}°"),
            Graphable::TiltY => format!("{value:.2}°"),
        }
    }
}

impl fmt::Display for Graphable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
