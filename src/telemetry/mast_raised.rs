use parse_display::{Display, FromStr};

/// enum representing the MAST_RAISED telemetry field
#[derive(Display, FromStr, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MastRaised {
    /// Flag mast raised after landing
    #[display("M")]
    Raised = b'M',

    /// Flag mast not raised
    #[display("N")]
    NotRaised = b'N',
}
