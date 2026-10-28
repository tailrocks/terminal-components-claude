#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EditEntry {
    #[default]
    Immediate,

    TypeToEdit,

    Explicit,
}
