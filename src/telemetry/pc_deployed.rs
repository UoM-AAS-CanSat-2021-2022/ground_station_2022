use parse_display::{Display, FromStr};

/// enum representing the PC_DEPLOYED telemetry field
#[derive(Display, FromStr, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PcDeployed {
    /// Probe parachute deployed (200m)
    #[display("C")]
    Deployed = b'C',

    /// Probe parachute not deployed
    #[display("N")]
    NotDeployed = b'N',
}
