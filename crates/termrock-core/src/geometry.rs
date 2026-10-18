#![doc = r" Scroll headroom and axis selection for scroll regions."]
#![doc = r""]
#![doc = r" Owns [`Headroom`], the remaining space inside an area along all four"]
#![doc = r" sides, and [`Axes`], which axes a scroll region handles."]

#[doc = r" Remaining space inside an area along all four sides."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Headroom {
    #[doc = r" Rows above."]
    pub up: u16,
    #[doc = r" Rows below."]
    pub down: u16,
    #[doc = r" Columns to the left."]
    pub left: u16,
    #[doc = r" Columns to the right."]
    pub right: u16,
}

#[doc = r" Which axes a scroll region handles."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Axes {
    #[doc = r" Vertical only."]
    #[default]
    V,
    #[doc = r" Horizontal only."]
    H,
    #[doc = r" Both axes."]
    Both,
}

impl Axes {
    #[doc = r" Whether the region handles `axis`."]
    pub const fn handles(self, axis: crate::event::Axis) -> bool {
        matches!(
            (self, axis),
            (Axes::Both, _) | (Axes::V, crate::event::Axis::V) | (Axes::H, crate::event::Axis::H)
        )
    }
}
