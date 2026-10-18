use ratatui::layout::Size;

use crate::ui::ctx::RenderCtx;

pub trait Measure {
    fn measure(&self, avail: Size, ctx: &RenderCtx) -> Size;
}

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
