use crate::as_str::AsStr;
use enum_iterator::Sequence;
use parse_display::Display;
use std::default::Default;

/// Represents the argument to the
#[derive(Default, Sequence, Display, Copy, Clone, Eq, PartialEq)]
pub enum Target {
    #[default]
    #[display("C")]
    Container,

    #[display("P")]
    Payload,
}

// I know this is horrible, anyone reading this, I'm sorry
impl AsStr for Target {
    fn as_str(&self) -> &'static str {
        match self {
            Target::Container => "Container",
            Target::Payload => "Payload",
        }
    }
}

/// The various states the payload's FSM can be in
#[derive(Sequence, Display, Default, Copy, Clone, Eq, PartialEq)]
#[display(style = "SNAKE_CASE")]
pub enum ContainerState {
    #[default]
    Ascent,
    WaitDeploy,
    #[display("WAIT_PARA")]
    WaitParachute,
    #[display("WAIT_GND")]
    WaitGround,
    OnGround,
}

impl AsStr for ContainerState {
    fn as_str(&self) -> &'static str {
        match self {
            ContainerState::Ascent => "Ascent",
            ContainerState::WaitDeploy => "Wait Deploy",
            ContainerState::WaitParachute => "Wait Parachute",
            ContainerState::WaitGround => "Wait Ground",
            ContainerState::OnGround => "On Ground",
        }
    }
}

/// The various states the payload's FSM can be in
#[derive(Sequence, Display, Default, Copy, Clone, Eq, PartialEq)]
#[display(style = "SNAKE_CASE")]
pub enum PayloadState {
    #[default]
    Ascent,
    WaitDeploy,
    #[display("WAIT_PARA")]
    WaitParachute,
    #[display("WAIT_GND")]
    WaitGround,
    OnGround,
}

impl AsStr for PayloadState {
    fn as_str(&self) -> &'static str {
        match self {
            PayloadState::Ascent => "Ascent",
            PayloadState::WaitDeploy => "Wait Deploy",
            PayloadState::WaitParachute => "Wait Parachute",
            PayloadState::WaitGround => "Wait Ground",
            PayloadState::OnGround => "On Ground",
        }
    }
}
