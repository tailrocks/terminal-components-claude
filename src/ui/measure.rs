//! Natural size (plan §1.4 `Measure`, the part S15 builds on).
//!
//! A child that can size itself says how much room it needs to show all of
//! its content without squeezing: a container lays it out at that size and
//! scrolls when the size is larger than the room it has
//! ([`ScrollView`](crate::ui::scroll_view::ScrollView)). `avail` is the room
//! the container offers; a child that wraps reads its width from it, a
//! child that fills whatever it gets answers `avail`.

use ratatui::layout::Size;

use crate::ui::ctx::RenderCtx;

/// A child that knows its natural size.
pub trait Measure {
    /// The cells (width, height) the child needs to show all of its
    /// content when offered `avail`. It may be larger than `avail` on
    /// either axis: the container then scrolls instead of squeezing it.
    fn measure(&self, avail: Size, ctx: &RenderCtx) -> Size;
}

/// The larger of two sizes on each axis: content laid out at its natural
/// size, but never smaller than the room it fills.
pub fn at_least(size: Size, min: Size) -> Size {
    Size::new(size.width.max(min.width), size.height.max(min.height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_least_takes_the_larger_side_on_each_axis() {
        assert_eq!(
            at_least(Size::new(10, 50), Size::new(30, 20)),
            Size::new(30, 50)
        );
    }
}
