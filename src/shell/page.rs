//! What every page of every app can do (plan §1.8, `Page`): the input,
//! rendering and lifecycle contract the shell drives, so an app's own
//! screen trait only adds what is specific to it.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;

use super::Cx;
use crate::core::event::{Key, Outcome};
use crate::core::focus::FocusChange;
use crate::core::id::WidgetId;
use crate::core::pointer::PointerEvent;
use crate::ui::context::ContextAnchor;
use crate::ui::ctx::RenderCtx;
use crate::ui::layer::LayerTag;
use crate::ui::layers::LayerResult;
use crate::ui::leave::Stakes;
use crate::widgets::keyhint::Hint;
use crate::widgets::menu::{ContextMenu, MenuItem};

/// The types an app's pages work with, named once so [`Page`] takes one
/// parameter.
pub trait PageTypes {
    /// The app state a page reads and changes (its world or model).
    type State: ?Sized;
    /// A message the app delivers to its pages (a finished job, a tick of
    /// a simulation).
    type Msg;
    /// What a layer reports to the page that opened it.
    type Result;
    /// What the app's layers read while they draw and handle input.
    type LayerState: ?Sized;
    /// The app's navigation.
    type Go;
}

/// The event context a page of `A` handles input in.
pub type PageCx<'a, A> =
    Cx<'a, <A as PageTypes>::Result, <A as PageTypes>::LayerState, <A as PageTypes>::Go>;

/// Everything the shell asks of a page. Default bodies keep small pages
/// small; an app's own screen trait extends this with what only it needs.
pub trait Page<A: PageTypes> {
    fn on_key(&mut self, key: &Key, s: &mut A::State, cx: &mut PageCx<A>) -> Outcome;
    /// One pointer event the router resolved for this page
    /// ([`crate::core::pointer::PointerRouter`]): a press (with the click
    /// count it starts), a completed click (its count is `ev.count()`), a
    /// drag of the held press, a release away from it, a secondary or
    /// middle click, or a wheel notch on either axis (offered to each
    /// owner under the pointer, innermost first, until one takes it).
    /// Hand it to the owning widget
    /// ([`crate::core::pointer::MouseTarget::route`]).
    fn pointer(&mut self, _ev: &PointerEvent, _s: &mut A::State, _cx: &mut PageCx<A>) -> Outcome {
        Outcome::Ignored
    }
    fn on_paste(&mut self, _text: &str, _s: &mut A::State, _cx: &mut PageCx<A>) -> Outcome {
        Outcome::Ignored
    }
    /// A control lost focus (or a click on dead space ended its edit): the
    /// page that owns `ch.from` ends the edit through the control's
    /// `on_blur` and runs its own commit hooks.
    fn on_blur(&mut self, _ch: FocusChange, _s: &mut A::State, _cx: &mut PageCx<A>) -> Outcome {
        Outcome::Ignored
    }
    fn on_tick(&mut self, _s: &mut A::State, _cx: &mut PageCx<A>) -> Outcome {
        Outcome::Ignored
    }
    fn on_msg(&mut self, _msg: &A::Msg, _s: &mut A::State, _cx: &mut PageCx<A>) -> Outcome {
        Outcome::Ignored
    }
    /// A layer this page opened reported `result` under `tag`.
    fn on_layer_result(
        &mut self,
        _tag: &LayerTag,
        _result: A::Result,
        _s: &mut A::State,
        _cx: &mut PageCx<A>,
    ) -> Outcome {
        Outcome::Ignored
    }
    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, s: &A::State);
    fn hints(&self, focus: Option<WidgetId>, s: &A::State) -> Vec<Hint>;
    /// The page's breadcrumb segment.
    fn crumb(&self, s: &A::State) -> String;
    /// Where keyboard focus lands when nothing better is known.
    fn primary_focus(&self) -> Option<WidgetId>;
    /// A control of the page is editing text (the shell's letter keys
    /// type instead of acting).
    fn is_editing(&self) -> bool {
        false
    }
    /// The page animates and needs ticks to redraw.
    fn animating(&self, _s: &A::State) -> bool {
        false
    }
    /// The page became the shown one.
    fn enter(&mut self, _s: &mut A::State, _cx: &mut PageCx<A>) {}
    /// Esc at the top of the page's ladder: by default the page asks to
    /// go back ([`Cx::back`]). `Ignored` means the page has nowhere to go
    /// back to, and the shell decides whether Esc may quit.
    fn on_esc_top(&mut self, _s: &mut A::State, cx: &mut PageCx<A>) -> Outcome {
        cx.back();
        Outcome::Changed
    }
    /// What leaving this page (or quitting) would lose.
    fn stakes(&mut self, _s: &A::State) -> Stakes {
        Stakes::Clean
    }
}

/// Where the context menu of hand-drawn list row `i` opens for the
/// keyboard: below the row as drawn this frame (`list.child(i)`), else
/// below its slot in `area` scrolled by `offset`.
pub fn row_anchor(list: WidgetId, i: usize, area: Rect, offset: usize) -> ContextAnchor {
    let dy = i
        .saturating_sub(offset)
        .min(area.height.saturating_sub(1) as usize) as u16;
    ContextAnchor::Control {
        id: list.child(i),
        fallback: Rect::new(area.x, area.y + dy, area.width.max(1), 1),
    }
}

/// Open a context menu through the shell's layer stack: `items` titled
/// `title` at `anchor`, answered to the page as a menu result under `tag`.
pub fn open_context<R, W, G>(
    cx: &mut Cx<R, W, G>,
    id: WidgetId,
    anchor: ContextAnchor,
    title: impl Into<String>,
    items: Vec<MenuItem>,
    tag: LayerTag,
) -> Outcome
where
    R: From<LayerResult> + 'static,
    W: ?Sized + 'static,
{
    let menu = anchor.apply(ContextMenu::new(id, items)).title(title);
    cx.open(menu, tag);
    Outcome::Changed
}

/// The key a context-menu item replays, from its character in the menu's
/// tag: `'\n'` is Enter, `'←'`/`'→'` the arrows, `'\u{1b}'` Esc, any other
/// character that plain key.
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
        mods: KeyModifiers::NONE,
    }
}

/// A row menu's chosen item `chosen` under `tag` of `kind` whose key is
/// `row:keys`: the row it was opened for, the key the item replays
/// ([`replay_key`]) and the item.
pub fn row_choice(
    tag: &LayerTag,
    kind: &str,
    chosen: Option<usize>,
) -> Option<(usize, Option<Key>, usize)> {
    let k = chosen?;
    if tag.kind != kind {
        return None;
    }
    let (row, keys) = tag.key.split_once(':')?;
    let row = row.parse::<usize>().ok()?;
    Some((row, keys.chars().nth(k).map(replay_key), k))
}

/// A tick's rebuild of a page's view from the app state: `Changed` only
/// when what `view` reads differs afterwards, so an idle page asks for no
/// redraw.
pub fn rebuild_on_tick<P, V: PartialEq>(
    page: &mut P,
    view: impl Fn(&P) -> V,
    rebuild: impl FnOnce(&mut P),
) -> Outcome {
    let before = view(page);
    rebuild(page);
    if view(page) == before {
        Outcome::Ignored
    } else {
        Outcome::Changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_choice_names_the_row_and_the_key_it_replays() {
        let tag = LayerTag::new("rowmenu").key("3:\nd");
        let (row, key, k) = row_choice(&tag, "rowmenu", Some(1)).unwrap();
        assert_eq!((row, k), (3, 1));
        assert_eq!(key.unwrap().code, KeyCode::Char('d'));
        assert_eq!(row_choice(&tag, "other", Some(1)), None);
        assert_eq!(row_choice(&tag, "rowmenu", None), None);
    }
}
