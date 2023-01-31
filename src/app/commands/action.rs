use enum_iterator::Sequence;

#[derive(Sequence, Default, Debug, Copy, Clone)]
pub enum Action {
    #[default]
    Enable,
    Disable,
    Flag,
    Beacon,
}
