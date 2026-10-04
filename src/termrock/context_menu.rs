//! Termrock ContextMenu component, stable target capture, and anchored popover placement.

use crate::termrock::identity::{Id, ItemKey, Revision};
use crate::termrock::layers::Anchor;
use crate::termrock::layout::{Constraints, Rect, Size};
use crate::termrock::menu::{Menu, MenuAction, MenuItem, MenuState};
use crate::termrock::response::Response;
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::theme::StylePatch;

/// An anchored context menu associated with a stable target ItemKey.
#[derive(Debug, Clone)]
pub struct ContextMenu<'a> {
    pub id: Id,
    pub target: ItemKey,
    pub items: &'a [MenuItem<'a>],
    pub revision: Revision,
    pub anchor: Option<Anchor>,
    pub title: Option<&'a str>,
    pub patch: Option<StylePatch>,
}

impl<'a> ContextMenu<'a> {
    pub fn new(id: Id, target: ItemKey, items: &'a [MenuItem<'a>], revision: Revision) -> Self {
        Self {
            id,
            target,
            items,
            revision,
            anchor: None,
            title: None,
            patch: None,
        }
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = Some(anchor);
        self
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    pub const fn target(&self) -> ItemKey {
        self.target
    }

    pub fn as_menu(&self) -> Menu<'a> {
        let mut m = Menu::new(self.id.clone(), self.items, self.revision);
        if let Some(t) = self.title {
            m = m.title(t);
        }
        if let Some(p) = self.patch {
            m = m.patch(p);
        }
        m
    }

    pub fn measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        self.as_menu().measure(cx, constraints)
    }

    pub fn update(&self, cx: &mut Cx<'_>, state: &mut MenuState) -> Response<MenuAction> {
        self.as_menu().update(cx, state)
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &MenuState) -> Rect {
        let menu = self.as_menu();
        let target_area = if let Some(Anchor::Position(pos)) = self.anchor {
            // Anchor at specific cell position, clamped within area
            let measure_cx = MeasureCx::new(
                Constraints::loose(Size::new(area.width, area.height)),
                ui.theme,
                crate::termrock::theme::ColorLevel::TrueColor,
            );
            let size = menu.measure(
                &measure_cx,
                Constraints::loose(Size::new(area.width, area.height)),
            );
            let mut x = pos.x;
            let mut y = pos.y;
            if x + size.width > area.x + area.width {
                x = (area.x + area.width).saturating_sub(size.width);
            }
            if y + size.height > area.y + area.height {
                y = (area.y + area.height).saturating_sub(size.height);
            }
            Rect::new(
                x,
                y,
                size.width.min(area.width),
                size.height.min(area.height),
            )
        } else {
            area
        };

        menu.draw(ui, target_area, state)
    }
}
