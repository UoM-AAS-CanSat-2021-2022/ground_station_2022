use enum_iterator::Sequence;

#[derive(Sequence, Default, Debug, Copy, Clone)]
pub enum SimMode {
    #[default]
    Disable,
    Activate,
    Enable,
}
