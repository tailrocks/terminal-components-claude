//! The page contract: what one routed screen owes the shell.
//!
//! Ported from PR #26 `src/shell/page.rs` (commit `f4867e9b5`). The parity
//! adaptation collapses the source's `PageTypes` associated-type machinery:
//! a page updates against the runtime's [`termrock_runtime::cx::Cx`] (it
//! drains its own intents) and draws as a composition leaf. The source's
//! `row_anchor`, `open_context` and `row_choice` helpers are not ported:
//! they stage context menus through the source's `LayerTag`/`LayerResult`
//! machinery, which the parity overlay layer (`RenderCtx::overlay`,
//! `LayerSpec`) replaces; that port belongs to the overlay extraction wave.
//! Focus blur, ticks and messages arrive as intents here rather than as
//! separate trait methods.

use ratatui_core::layout::Rect;
use termrock_runtime::author::{CompositionLeaf, Constraints, Measure, Size, composition_leaf};
use termrock_runtime::cx::Cx;
use termrock_runtime::event::{Key, KeyCode};
use termrock_runtime::response::Response;
use termrock_runtime::ui::Ui;

use crate::quit::Stakes;

/// What one routed screen owes the shell: an update against the runtime's
/// [`Cx`], a draw, and the metadata the chrome, footer and quit guard need.
pub trait Page {
    /// Drain the page's intents and update. Keys, pastes, pointer events,
    /// focus changes and layer events arrive here as intents for the page's
    /// owner ids.
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()>;

    /// Draw into the body region the shell hands over.
    fn draw(&self, ui: &mut Ui<'_>);

    /// Draw into the shell-resolved body bounds.
    ///
    /// Existing pages keep their `draw` implementation. Pages that need
    /// layout-relative cursor or control coordinates can override this hook
    /// and use `body` for registration and drawing.
    fn draw_in(&self, ui: &mut Ui<'_>, body: Rect) {
        let _ = body;
        self.draw(ui);
    }

    /// The crumb segment naming this page.
    fn label(&self) -> String;

    /// What the app stands to lose if this page goes (for the quit guard).
    fn stakes(&self) -> Stakes {
        Stakes::Clean
    }

    /// The control to focus when the page appears.
    fn primary_focus(&self) -> Option<termrock_runtime::id::Id> {
        None
    }

    /// Is a text field editing (the footer hides Tab-next while it is)?
    fn is_editing(&self) -> bool {
        false
    }

    /// Is the page animating (the runtime keeps repainting)?
    fn animating(&self) -> bool {
        false
    }

    /// The page appears: one hook for its enter work.
    fn on_show(&mut self, _cx: &mut Cx<'_>) {}

    /// Ctrl+C with nothing to copy: the page may cancel work in flight.
    fn on_interrupt(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }
}

/// Replay a key as if typed: a menu choice or hint click names a character
/// and the shell turns it into the key event the page would have seen.
///
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

/// Rebuild a page on tick only when its view actually changed, so idle
/// animation frames stay idle.
///
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

/// Adapter presenting a [`Page`] as a composition leaf, so the shell's body
/// slot can host it without a second layout system.
pub struct PageLeaf<'a, P: Page>(&'a P);

impl<P: Page> Measure for PageLeaf<'_, P> {
    fn measure(&self, _ui: &Ui<'_>, constraints: Constraints) -> Size {
        // A page fills whatever body region the shell hands it, but claims
        // no hard minimum: it follows the sibling leaves' `min: 0`
        // convention, so a starved body still resolves instead of failing
        // the layout.
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

/// Borrow a page as a composition leaf for a body slot.
pub fn page_leaf<P: Page>(page: &P) -> impl CompositionLeaf + use<'_, P> {
    composition_leaf(
        move |ui: &Ui<'_>, constraints: Constraints| PageLeaf(page).measure(ui, constraints),
        move |ui: &mut Ui<'_>, logical| PageLeaf(page).draw(ui, logical),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_key_names_the_keys_a_menu_choice_names() {
        assert_eq!(replay_key('d').code, KeyCode::Char('d'));
        assert_eq!(replay_key('\n').code, KeyCode::Enter);
        assert_eq!(replay_key('←').code, KeyCode::Left);
        assert_eq!(replay_key('→').code, KeyCode::Right);
        assert_eq!(replay_key('\u{1b}').code, KeyCode::Esc);
        assert_eq!(
            replay_key('d').mods,
            termrock_runtime::event::KeyModifiers::NONE
        );
    }

    #[test]
    fn rebuild_on_tick_rebuilds_only_when_the_view_moves() {
        let mut page = 3;
        assert!(!rebuild_on_tick(&mut page, |p| *p, |p| *p += 0));
        assert!(rebuild_on_tick(&mut page, |p| *p, |p| *p += 1));
        assert_eq!(page, 4);
    }
}
