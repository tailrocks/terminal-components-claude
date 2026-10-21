/// Policy for entering an editable state when a text editor receives input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EditEntry {
    /// Begin editing when the editor receives keyboard focus.
    #[default]
    Immediate,
    /// Begin editing when the user types a character.
    TypeToEdit,
    /// Wait for an explicit edit command such as Enter or `i`.
    Explicit,
}
