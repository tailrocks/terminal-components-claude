#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Headroom {
    pub up: u16,
    pub down: u16,
    pub left: u16,
    pub right: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Axes {
    #[default]
    V,
    H,
    Both,
}

impl Axes {
    pub const fn handles(self, axis: crate::event::Axis) -> bool {
        matches!(
            (self, axis),
            (Axes::Both, _) | (Axes::V, crate::event::Axis::V) | (Axes::H, crate::event::Axis::H)
        )
    }
}
