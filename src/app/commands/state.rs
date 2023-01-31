use enum_iterator::Sequence;
use std::default::Default;

/// Represents the argument to the
#[derive(Sequence, Copy, Clone, Eq, PartialEq)]
pub enum State {
    Container(ContainerState),
    Payload(PayloadState),
}

impl Default for State {
    fn default() -> Self {
        Self::Container(Default::default())
    }
}

/// The various states the payload's FSM can be in
#[derive(Sequence, Default, Copy, Clone, Eq, PartialEq)]
pub enum ContainerState {
    #[default]
    Ascent,
    WaitDeploy,
    WaitParachute,
    WaitGround,
    OnGround,
}

/// The various states the payload's FSM can be in
#[derive(Sequence, Default, Copy, Clone, Eq, PartialEq)]
pub enum PayloadState {
    #[default]
    Ascent,
    WaitDeploy,
    WaitParachute,
    WaitGround,
    OnGround,
}
