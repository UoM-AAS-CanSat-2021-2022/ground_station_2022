use parse_display::{Display, FromStr};

#[derive(Display, FromStr, Debug, Copy, Clone, PartialEq, Eq)]
pub enum SoftwareState {
    #[display("LW")]
    LaunchWait,
    #[display("CAL")]
    Calibrating,
    #[display("LWC")]
    LaunchWaitCal,
    #[display("ASC")]
    Ascent,
    #[display("DP1")]
    DescentPar1,
    #[display("DP2")]
    DescentPar2,
    #[display("DTP")]
    DescentTpRel,
    #[display("LAN")]
    Landed,
}
