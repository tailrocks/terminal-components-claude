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

pub trait PageTypes {
    type State: ?Sized;

    type Msg;

    type Result;

    type LayerState: ?Sized;

    type Go;
}

pub type PageCx<'a, A> =
    Cx<'a, <A as PageTypes>::Result, <A as PageTypes>::LayerState, <A as PageTypes>::Go>;

pub trait Page<A: PageTypes> {
    fn on_key(&mut self, key: &Key, s: &mut A::State, cx: &mut PageCx<A>) -> Outcome;

    fn pointer(&mut self, _ev: &PointerEvent, _s: &mut A::State, _cx: &mut PageCx<A>) -> Outcome {
        Outcome::Ignored
    }
    fn on_paste(&mut self, _text: &str, _s: &mut A::State, _cx: &mut PageCx<A>) -> Outcome {
        Outcome::Ignored
    }

    fn on_blur(&mut self, _ch: FocusChange, _s: &mut A::State, _cx: &mut PageCx<A>) -> Outcome {
        Outcome::Ignored
    }
    fn on_tick(&mut self, _s: &mut A::State, _cx: &mut PageCx<A>) -> Outcome {
        Outcome::Ignored
    }
    fn on_msg(&mut self, _msg: &A::Msg, _s: &mut A::State, _cx: &mut PageCx<A>) -> Outcome {
        Outcome::Ignored
    }

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

    fn crumb(&self, s: &A::State) -> String;

    fn primary_focus(&self) -> Option<WidgetId>;

    fn is_editing(&self) -> bool {
        false
    }

    fn animating(&self, _s: &A::State) -> bool {
        false
    }

    fn enter(&mut self, _s: &mut A::State, _cx: &mut PageCx<A>) {}

    fn on_esc_top(&mut self, _s: &mut A::State, cx: &mut PageCx<A>) -> Outcome {
        cx.back();
        Outcome::Changed
    }

    fn on_interrupt(&mut self, _s: &mut A::State, _cx: &mut PageCx<A>) -> Outcome {
        Outcome::Ignored
    }

    fn stakes(&mut self, _s: &A::State) -> Stakes {
        Stakes::Clean
    }
}

pub fn row_anchor(list: WidgetId, i: usize, area: Rect, offset: usize) -> ContextAnchor {
    let dy = i
        .saturating_sub(offset)
        .min(area.height.saturating_sub(1) as usize) as u16;
    ContextAnchor::Control {
        id: list.child(i),
        fallback: Rect::new(area.x, area.y + dy, area.width.max(1), 1),
    }
}

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
