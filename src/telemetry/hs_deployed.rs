use parse_display::{Display, FromStr};

/// enum representing the HS_DEPLOYED telemetry field
#[derive(Display, FromStr, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum HsDeployed {
    /// Probe with heat shield is deployed
    #[display("P")]
    Deployed = b'P',

    /// Probe is not deployed
    #[display("N")]
    NotDeployed = b'N',
}
