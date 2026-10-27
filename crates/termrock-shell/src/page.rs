//! Ported from PR #26 `src/shell/page.rs` (commit `f4867e9b5`).

use ratatui_core::layout::Rect;
use termrock_runtime::author::{CompositionLeaf, Constraints, Measure, Size, composition_leaf};
use termrock_runtime::cx::Cx;
use termrock_runtime::event::{Key, KeyCode};
use termrock_runtime::response::Response;
use termrock_runtime::ui::Ui;

use crate::quit::Stakes;

pub trait Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()>;

    fn draw(&self, ui: &mut Ui<'_>);

    fn draw_in(&self, ui: &mut Ui<'_>, body: Rect) {
        let _ = body;
        self.draw(ui);
    }

    fn label(&self) -> String;

    fn stakes(&self) -> Stakes {
        Stakes::Clean
    }

    fn primary_focus(&self) -> Option<termrock_runtime::id::Id> {
        None
    }

    fn is_editing(&self) -> bool {
        false
    }

    fn animating(&self) -> bool {
        false
    }

    fn on_show(&mut self, _cx: &mut Cx<'_>) {}

    fn on_interrupt(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }
}

/// Ported from PR #26 `page.rs::replay_key`.
#[must_use]
pub fn replay_key(c: char) -> Key {
    let code = match c {
        '\n' => KeyCode::Enter,
        '←' => KeyCode::Left,
        '→' => KeyCode::Right,
        '\u{1b}' => KeyCode::Esc,
        c => KeyCode::Char(c),
    };
    Key {
        code,
        mods: termrock_runtime::event::KeyModifiers::NONE,
    }
}

/// Ported from PR #26 `page.rs::rebuild_on_tick`.
pub fn rebuild_on_tick<P, V: PartialEq>(
    page: &mut P,
    view: impl Fn(&P) -> V,
    rebuild: impl FnOnce(&mut P),
) -> bool {
    let before = view(page);
    rebuild(page);
    view(page) != before
}

pub struct PageLeaf<'a, P: Page>(&'a P);

impl<P: Page> Measure for PageLeaf<'_, P> {
    fn measure(&self, _ui: &Ui<'_>, constraints: Constraints) -> Size {
        Size {
            min: (0, 0),
            preferred: constraints.max,
        }
    }
}

impl<P: Page> CompositionLeaf for PageLeaf<'_, P> {
    fn draw(&self, ui: &mut Ui<'_>, logical: ratatui_core::layout::Rect) {
        self.0.draw_in(ui, logical);
    }
}

pub fn page_leaf<P: Page>(page: &P) -> impl CompositionLeaf + use<'_, P> {
    composition_leaf(
        move |ui: &Ui<'_>, constraints: Constraints| PageLeaf(page).measure(ui, constraints),
        move |ui: &mut Ui<'_>, logical| PageLeaf(page).draw(ui, logical),
    )
}
