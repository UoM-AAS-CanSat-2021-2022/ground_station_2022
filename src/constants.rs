/// our TEAM ID
pub const TEAM_ID: u16 = 1047;
pub const TEAM_ID_STR: &str = "1047";

/// Sea level pressure in HPA
pub const SEALEVEL_HPA: f64 = 1013.25;
pub const SEALEVEL_PA: u32 = 101325;

/// The address of the container
pub const CONTAINER_ADDR: u16 = 0x00_01;

/// The address of the probe
pub const PROBE_ADDR: u16 = 0x00_02;

/// The broadcast address
pub const BROADCAST_ADDR: u16 = 0xFF_FF;

/// The list of valid baud rates
pub const BAUD_RATES: [u32; 9] = [1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400];

/// The file to save the telemetry to
pub const TELEMETRY_FILE: &str = "Flight_1047.csv";
