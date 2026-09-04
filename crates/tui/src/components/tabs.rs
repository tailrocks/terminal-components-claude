//! `Tabs` (`COMPONENT_ARCHITECTURE.md` §12.4, §17.0 A7, Appendix A 4D).

use core::fmt;
use core::marker::PhantomData;

use ratatui_core::layout::{Position, Rect};

use super::{Acc, Overrides, cell_at, first_row};
use crate::collection::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, Reconcile, Reconciliation, RowFn, RowUi, Status,
};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::theme::{Family, GlyphRole, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};

/// What a tab strip reports; every tab action carries the tab's key.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabsAction {
    /// A tab became active.
    Activated(ItemKey),
    /// A tab's close affordance fired.
    Close(ItemKey),
    /// The new-tab affordance fired.
    New,
}

/// The const-constructible commands of the tabs keymap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabsCmd {
    /// Activate the previous tab.
    Prev,
    /// Activate the next tab.
    Next,
    /// Activate the cursor tab.
    Activate,
    /// Activate tab `n` (1-based).
    Nth(u8),
    /// Close the cursor tab.
    Close,
    /// Open a new tab.
    New,
}

const fn b(chord: Chord, cmd: TabsCmd, label: &'static str, visible: bool) -> Binding<TabsCmd> {
    Binding {
        chord,
        cmd,
        label,
        priority: if visible { 60 } else { 10 },
        visible,
    }
}

const BASE: [Binding<TabsCmd>; 15] = [
    b(Chord::key(KeyCode::Left), TabsCmd::Prev, "Prev tab", true),
    b(Chord::key(KeyCode::Right), TabsCmd::Next, "Next tab", true),
    b(
        Chord::key(KeyCode::Char('h')),
        TabsCmd::Prev,
        "Prev tab",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('l')),
        TabsCmd::Next,
        "Next tab",
        false,
    ),
    b(
        Chord::key(KeyCode::Enter),
        TabsCmd::Activate,
        "Activate",
        false,
    ),
    b(
        Chord::key(KeyCode::Char(' ')),
        TabsCmd::Activate,
        "Activate",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('1')),
        TabsCmd::Nth(1),
        "Tab 1",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('2')),
        TabsCmd::Nth(2),
        "Tab 2",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('3')),
        TabsCmd::Nth(3),
        "Tab 3",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('4')),
        TabsCmd::Nth(4),
        "Tab 4",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('5')),
        TabsCmd::Nth(5),
        "Tab 5",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('6')),
        TabsCmd::Nth(6),
        "Tab 6",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('7')),
        TabsCmd::Nth(7),
        "Tab 7",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('8')),
        TabsCmd::Nth(8),
        "Tab 8",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('9')),
        TabsCmd::Nth(9),
        "Tab 9",
        false,
    ),
];
const CLOSE_X: Binding<TabsCmd> = b(
    Chord::key(KeyCode::Char('x')),
    TabsCmd::Close,
    "Close",
    true,
);
const CLOSE_DEL: Binding<TabsCmd> = b(Chord::key(KeyCode::Delete), TabsCmd::Close, "Close", false);
const NEW_N: Binding<TabsCmd> = b(Chord::key(KeyCode::Char('n')), TabsCmd::New, "New", true);

macro_rules! table {
    ($name:ident, $n:expr, [$($extra:expr),*]) => {
        const $name: [Binding<TabsCmd>; $n] = [
            BASE[0], BASE[1], BASE[2], BASE[3], BASE[4], BASE[5], BASE[6], BASE[7], BASE[8],
            BASE[9], BASE[10], BASE[11], BASE[12], BASE[13], BASE[14], $($extra),*
        ];
    };
}
table!(PLAIN, 15, []);
table!(CLOSABLE, 17, [CLOSE_X, CLOSE_DEL]);
table!(NEWABLE, 16, [NEW_N]);
table!(FULL, 18, [CLOSE_X, CLOSE_DEL, NEW_N]);

/// Durable state of a [`Tabs`]: the active key, the cursor key and the
/// strip window's logical first tab.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct TabsState {
    core: CollectionCore,
    active: Option<ItemKey>,
    first: Option<ItemKey>,
    first_index: usize,
}

impl TabsState {
    /// The active key.
    pub const fn active(&self) -> Option<ItemKey> {
        self.active
    }

    /// The cursor key.
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    /// The logical first visible tab.
    pub const fn first(&self) -> Option<ItemKey> {
        self.first
    }

    /// Make `key` active (and the cursor).
    pub fn set_active(&mut self, index: usize, key: ItemKey) {
        self.active = Some(key);
        self.core.set_cursor(index, key);
    }
}

impl Reconcile for TabsState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        let r = self.core.reconcile(len, &key);
        if let Some(a) = self.active
            && !(0..len).any(|i| key(i) == a)
        {
            self.active = self.core.cursor();
        }
        match self.first {
            Some(f) => {
                if let Some(i) = (0..len).find(|&i| key(i) == f) {
                    self.first_index = i;
                } else {
                    let i = self.first_index.min(len.saturating_sub(1));
                    self.first_index = i;
                    self.first = (len > 0).then(|| key(i));
                }
            }
            None => self.first_index = 0,
        }
        r
    }

    fn invalidate(&mut self) {
        self.core.invalidate();
    }
}

/// A horizontal tab strip with stable keys: one focus stop, an active tab,
/// a keyboard cursor and a window that follows the logical first tab.
///
/// ## Construction
/// `Tabs::new(id)`; items are passed to each phase.
///
/// ## Ownership
/// The caller owns the items and a [`TabsState`]; the runtime owns focus,
/// hover and press.
///
/// ## Configuration
/// `.key`, `.row` (`Part::TAB` pre-styled), `.allow_new(bool)` (`false`),
/// `.closable(bool)` (`false`), `.status`, `.patch`, `.patch_part`,
/// `.state_override`.
///
/// ## Variants
/// `Family::TABS`, `DEFAULT` only.
///
/// ## States
/// The strip wears `FOCUSED`/`FOCUS_VISIBLE`/`HOVERED`/`PRESSED` from the
/// runtime; the active tab derives `ACTIVE`, the cursor tab `FOCUSED`. Under
/// `.state_override` the first tab in the window stands in for the cursor,
/// and it derives `ACTIVE` only when the forced state contains `SELECTED` —
/// a strip's selection *is* its active tab.
///
/// ## Actions
/// `Activated(k)`, `Close(k)`, `New`.
///
/// ## Focus
/// One `Focusable` stop; does not swallow typing.
///
/// ## Keyboard
/// `←`/`h` previous, `→`/`l` next (both activate), `Enter`/`Space`
/// activate the cursor, `1`–`9` activate by position; with `.closable`:
/// `x`/`Del` close; with `.allow_new`: `n` new.
///
/// ## Mouse
/// `PartRef::item(Part::TAB, k)` click activates; `PartRef::item(Part::CLOSE,
/// k)` click closes; `PartRef::of(Part::NEW)` click opens; the `OVERFLOW`
/// parts (`ItemKey::index(0)` left, `index(1)` right) shift the window.
///
/// ## Layout
/// Two rows: labels, then the rule. Tabs are content-sized from what the
/// renderer painted; the window is `fit` tabs from the logical first, with
/// `‹N` / `N›` counters when hidden. `measure` is `(…, tabs_height)`; `draw`
/// returns the two rows. `viewport_len = fit` is reported and the next
/// `update` keeps the active tab inside the window.
///
/// ## Parts
/// `CONTAINER`, `TAB` (one tab's plane), `LABEL` and `META` (what the
/// `.row` painter writes through [`RowUi`]), `CLOSE`, `NEW`, `RULE`,
/// `OVERFLOW`, `BADGE`.
///
/// ## Overrides
/// `.patch`, `.patch_part`; no slots.
///
/// ## Identity
/// `.key` supplies stable keys; `ByIndex` is unstable under reorder.
///
/// ## Testing
/// `TabsCase` with `ACTIVATES | FOCUSABLE | COLLECTION`;
/// `render::components::tabs::*`.
///
/// ## Invariants
/// After an insert or reorder the active key, the cursor key and the window
/// name the same tabs; `Close(k)` targets the logical tab.
pub struct Tabs<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    allow_new: bool,
    closable: bool,
    status: Status,
    ov: Overrides<'a>,
    _t: PhantomData<fn(&T)>,
}

impl<T, K, R> fmt::Debug for Tabs<'_, T, K, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tabs")
            .field("id", &self.id)
            .field("allow_new", &self.allow_new)
            .field("closable", &self.closable)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl<T> Tabs<'_, T, ByIndex, DefaultRow> {
    /// A strip keyed by index and painted through `Display`.
    pub const fn new(id: Id) -> Self {
        Tabs {
            id,
            key: ByIndex,
            row: DefaultRow,
            allow_new: false,
            closable: false,
            status: Status::Ready,
            ov: Overrides::new(),
            _t: PhantomData,
        }
    }
}

impl<'a, T, K, R> Tabs<'a, T, K, R> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::TAB,
        Part::LABEL,
        Part::META,
        Part::CLOSE,
        Part::NEW,
        Part::RULE,
        Part::OVERFLOW,
        Part::BADGE,
    ];

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// A stable key accessor.
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> Tabs<'a, T, K2, R> {
        Tabs {
            id: self.id,
            key: k,
            row: self.row,
            allow_new: self.allow_new,
            closable: self.closable,
            status: self.status,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    /// A tab painter (`Part::TAB` pre-styled).
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> Tabs<'a, T, K, R2> {
        Tabs {
            id: self.id,
            key: self.key,
            row: r,
            allow_new: self.allow_new,
            closable: self.closable,
            status: self.status,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    /// Show a trailing new-tab affordance.
    #[must_use]
    pub fn allow_new(mut self, yes: bool) -> Self {
        self.allow_new = yes;
        self
    }

    /// Show a close affordance on every tab.
    #[must_use]
    pub fn closable(mut self, yes: bool) -> Self {
        self.closable = yes;
        self
    }

    /// Data readiness.
    #[must_use]
    pub fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    /// Showcase / fixture use only (A11).
    #[must_use]
    pub fn state_override(mut self, s: StateFlags) -> Self {
        self.ov = self.ov.state_override(s);
        self
    }

    /// An instance patch over every part.
    #[must_use]
    pub fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.patch(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(ps);
        self
    }

    fn table(&self) -> &'static [Binding<TabsCmd>] {
        table_for(self.closable, self.allow_new)
    }
}

/// The binding table for a strip configuration.
pub(crate) const fn table_for(closable: bool, allow_new: bool) -> &'static [Binding<TabsCmd>] {
    match (closable, allow_new) {
        (false, false) => &PLAIN,
        (true, false) => &CLOSABLE,
        (false, true) => &NEWABLE,
        (true, true) => &FULL,
    }
}

/// Write `n` (clamped to 99) into `buf`; returns the digits.
fn digits(n: usize, buf: &mut [u8; 2]) -> &str {
    let n = n.min(99);
    if n >= 10 {
        buf[0] = b'0'.saturating_add((n / 10) as u8);
        buf[1] = b'0'.saturating_add((n % 10) as u8);
        core::str::from_utf8(&buf[..]).unwrap_or("")
    } else {
        buf[0] = b'0'.saturating_add(n as u8);
        core::str::from_utf8(&buf[..1]).unwrap_or("")
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> Tabs<'_, T, K, R> {
    fn key_at(&self, items: &[T], i: usize) -> ItemKey {
        items
            .get(i)
            .map_or(ItemKey::index(i), |it| self.key.key(it, i))
    }

    fn index_of(&self, items: &[T], key: ItemKey, hint: Option<usize>) -> Option<usize> {
        if let Some(h) = hint
            && h < items.len()
            && self.key_at(items, h) == key
        {
            return Some(h);
        }
        (0..items.len()).find(|&i| self.key_at(items, i) == key)
    }

    fn activate(&self, st: &mut TabsState, items: &[T], i: usize, acc: &mut Acc<TabsAction>) {
        if items.is_empty() {
            acc.consumed();
            return;
        }
        let i = i.min(items.len().saturating_sub(1));
        let key = self.key_at(items, i);
        st.set_active(i, key);
        acc.action(TabsAction::Activated(key));
    }

    /// Keep the active tab inside the window, using last frame's `fit`.
    fn follow(&self, st: &mut TabsState, items: &[T], fit: usize) {
        let Some(active) = st.active else { return };
        let Some(ai) = self.index_of(items, active, Some(st.core.cursor_index())) else {
            return;
        };
        if ai < st.first_index {
            st.first_index = ai;
            st.first = Some(active);
        } else if fit > 0 && ai >= st.first_index.saturating_add(fit) {
            let i = ai.saturating_add(1).saturating_sub(fit);
            st.first_index = i;
            st.first = Some(self.key_at(items, i));
        }
    }

    /// The update phase: reconcile, then drain keys and pointer intents.
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut TabsState, items: &[T]) -> Response<TabsAction> {
        let len = items.len();
        let _ = st.reconcile(len, |i| self.key_at(items, i));
        if st.core.cursor().is_none() && len > 0 {
            let key = self.key_at(items, 0);
            st.core.set_cursor(0, key);
        }
        if st.active.is_none() {
            st.active = st.core.cursor();
        }
        if st.first.is_none() && len > 0 {
            st.first = Some(self.key_at(items, 0));
            st.first_index = 0;
        }
        let mut acc = Acc::<TabsAction>::new();
        let table = self.table();
        for it in cx.intents(self.id) {
            match it {
                Intent::Key(k) => {
                    let cur = st.core.cursor_index();
                    match Binding::lookup(table, &k) {
                        Some(TabsCmd::Prev) => {
                            self.activate(st, items, cur.saturating_sub(1), &mut acc);
                        }
                        Some(TabsCmd::Next) => {
                            self.activate(st, items, cur.saturating_add(1), &mut acc);
                        }
                        Some(TabsCmd::Activate) => self.activate(st, items, cur, &mut acc),
                        Some(TabsCmd::Nth(n)) => {
                            let i = usize::from(n).saturating_sub(1);
                            if i < len {
                                self.activate(st, items, i, &mut acc);
                            } else {
                                acc.consumed();
                            }
                        }
                        Some(TabsCmd::Close) => {
                            if len > 0 {
                                acc.action(TabsAction::Close(self.key_at(items, cur)));
                            } else {
                                acc.consumed();
                            }
                        }
                        Some(TabsCmd::New) => acc.action(TabsAction::New),
                        None => {}
                    }
                }
                Intent::Pointer { phase, part, .. } => match (phase, part.part, part.item) {
                    (Phase::Press, Part::TAB, Some(k)) => {
                        if let Some(i) = self.index_of(items, k, Some(st.core.cursor_index())) {
                            st.core.set_cursor(i, k);
                        }
                        acc.changed();
                    }
                    (Phase::Click | Phase::DoubleClick, Part::TAB, Some(k)) => {
                        match self.index_of(items, k, Some(st.core.cursor_index())) {
                            Some(i) => self.activate(st, items, i, &mut acc),
                            None => acc.consumed(),
                        }
                    }
                    (Phase::Click, Part::CLOSE, Some(k)) => acc.action(TabsAction::Close(k)),
                    (Phase::Click, Part::NEW, _) => acc.action(TabsAction::New),
                    (Phase::Click, Part::OVERFLOW, Some(ItemKey::Index(0))) => {
                        let i = st.first_index.saturating_sub(1);
                        st.first_index = i;
                        st.first = (len > 0).then(|| self.key_at(items, i));
                        acc.changed();
                    }
                    (Phase::Click, Part::OVERFLOW, Some(ItemKey::Index(_))) => {
                        let i = st.first_index.saturating_add(1).min(len.saturating_sub(1));
                        st.first_index = i;
                        st.first = (len > 0).then(|| self.key_at(items, i));
                        acc.changed();
                    }
                    _ => acc.consumed(),
                },
                _ => {}
            }
        }
        if let Some(l) = cx.layout(self.id) {
            self.follow(st, items, l.viewport_len);
        }
        acc.finish(self.id)
    }

    /// The draw phase: the label row and the rule row.
    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the window, the overflow counters and the new affordance"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &TabsState, items: &[T]) -> Rect {
        if area.is_empty() {
            return area;
        }
        let len = items.len();
        let row0 = first_row(area);
        let used = Rect {
            height: area.height.min(2),
            ..area
        };
        if !self.ov.is_forced() {
            ui.register_control(self.id, used, Focusability::Focusable);
        }
        let live = self.ov.flags(ui.state(self.id)) | self.status.flags();
        let ov = self.ov;
        let id = self.id;
        let strip = ov.style(
            ui,
            id,
            Family::TABS,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        );
        ui.fill(used, strip.style);
        let rule_row = if area.height >= 2 {
            Some(Rect {
                y: area.y.saturating_add(1),
                height: 1,
                ..row0
            })
        } else {
            None
        };
        if let Some(rr) = rule_row {
            let quiet = ov.style(
                ui,
                id,
                Family::TABS,
                Variant::DEFAULT,
                Part::RULE,
                StateFlags::empty(),
            );
            for col in rr.columns() {
                ui.glyph(
                    col,
                    quiet.glyph.unwrap_or(GlyphRole::RuleQuiet),
                    quiet.style,
                );
            }
        }
        let last = ui.layout(self.id);
        let overflow_last = last.is_some_and(|l| l.viewport_len < l.content_len);
        let first_index = st
            .first
            .and_then(|f| self.index_of(items, f, Some(st.first_index)))
            .unwrap_or(0)
            .min(len);
        let new_w: u16 = if self.allow_new { 4 } else { 0 };
        let left_w: u16 = if first_index > 0 { 4 } else { 0 };
        let right_w: u16 = if overflow_last || first_index > 0 {
            4
        } else {
            0
        };
        let right_limit = row0.right().saturating_sub(right_w).saturating_sub(new_w);
        let mut x = row0.x.saturating_add(left_w);
        let mut fit = 0usize;
        let cursor = st.core.cursor();
        let forced = self.ov.is_forced();
        for i in first_index..len {
            let Some(item) = items.get(i) else { break };
            let key = self.key.key(item, i);
            let avail = right_limit.saturating_sub(x);
            if avail < 3 {
                break;
            }
            // A11 reference rendering: with no real state the first tab in the
            // window stands in for the cursor, and it becomes the *active* tab
            // only when the forced state names the strip's selection. A tab
            // strip's `SELECTED` is its active tab, so keeping `ACTIVE` off the
            // merely-focused tab is what makes `focused` and `selected`
            // distinguishable without colour (§11.4, §16.2 case 9).
            let forced_first = forced && i == first_index;
            let is_cursor = cursor == Some(key) || (forced_first && cursor.is_none());
            let is_active = st.active == Some(key)
                || (forced_first && st.active.is_none() && live.contains(StateFlags::SELECTED));
            let mut flags = StateFlags::empty();
            if is_active {
                flags |= StateFlags::ACTIVE;
            }
            if is_cursor {
                flags |=
                    live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE | StateFlags::PRESSED);
                if forced {
                    flags |=
                        live & (StateFlags::SELECTED | StateFlags::DISABLED | StateFlags::HOVERED);
                }
            }
            // paint the tab's content into the remaining strip, then measure it
            let content = Rect {
                x: x.saturating_add(1),
                y: row0.y,
                width: avail.saturating_sub(1),
                height: 1,
            };
            {
                let mut r = RowUi::new(ui, id, Family::TABS, Variant::DEFAULT, flags, key, content);
                self.row.row(item, &mut r);
            }
            let label_w = painted_width(ui, content).max(1);
            let close_w: u16 = if self.closable { 2 } else { 0 };
            let tab_w = 1u16
                .saturating_add(label_w)
                .saturating_add(1)
                .saturating_add(close_w);
            let tab = Rect {
                x,
                y: row0.y,
                width: tab_w.min(avail),
                height: 1,
            };
            let tail = Rect {
                x: tab.right(),
                y: row0.y,
                width: right_limit.saturating_sub(tab.right()),
                height: 1,
            };
            ui.fill(tail, strip.style);
            let ts = ov.style(ui, id, Family::TABS, Variant::DEFAULT, Part::TAB, flags);
            ui.paint_style(tab, ts.style);
            if self.closable {
                let close_cell = cell_at(tab, tab.right().saturating_sub(2));
                let cs = ov.style(ui, id, Family::TABS, Variant::DEFAULT, Part::CLOSE, flags);
                ui.glyph(close_cell, cs.glyph.unwrap_or(GlyphRole::Close), cs.style);
            }
            if is_active && let Some(rr) = rule_row {
                let rs = ov.style(ui, id, Family::TABS, Variant::DEFAULT, Part::RULE, flags);
                let span = Rect {
                    x: tab.x,
                    width: tab.width,
                    ..rr
                };
                for col in span.columns() {
                    ui.glyph(col, rs.glyph.unwrap_or(GlyphRole::RuleActive), rs.style);
                }
            }
            ui.register_part(self.id, PartRef::item(Part::TAB, key), tab);
            if self.closable {
                let close_cell = cell_at(tab, tab.right().saturating_sub(2));
                ui.register_part(self.id, PartRef::item(Part::CLOSE, key), close_cell);
            }
            x = tab.right().saturating_add(1);
            fit = fit.saturating_add(1);
        }
        let mut buf = [0u8; 2];
        if first_index > 0 {
            let cell = Rect {
                x: row0.x,
                y: row0.y,
                width: left_w.min(row0.width),
                height: 1,
            };
            let os = ov.style(
                ui,
                id,
                Family::TABS,
                Variant::DEFAULT,
                Part::OVERFLOW,
                StateFlags::empty(),
            );
            let used = ui.glyph(cell, GlyphRole::OverflowLeft, os.style);
            ui.paint_str(
                super::shift(cell, used),
                digits(first_index, &mut buf),
                os.style,
            );
            ui.register_part(
                self.id,
                PartRef::item(Part::OVERFLOW, ItemKey::index(0)),
                cell,
            );
        }
        let hidden_right = len.saturating_sub(first_index.saturating_add(fit));
        if hidden_right > 0 {
            let cell = Rect {
                x: row0
                    .right()
                    .saturating_sub(new_w)
                    .saturating_sub(4)
                    .max(row0.x),
                y: row0.y,
                width: 4.min(row0.width),
                height: 1,
            };
            let os = ov.style(
                ui,
                id,
                Family::TABS,
                Variant::DEFAULT,
                Part::OVERFLOW,
                StateFlags::empty(),
            );
            ui.fill(cell, strip.style);
            let d = digits(hidden_right, &mut buf);
            let dw = crate::text::width(d);
            let text = Rect {
                x: cell.right().saturating_sub(1).saturating_sub(dw),
                width: dw,
                ..cell
            }
            .intersection(cell);
            ui.paint_str(text, d, os.style);
            ui.glyph(
                cell_at(cell, cell.right().saturating_sub(1)),
                GlyphRole::OverflowRight,
                os.style,
            );
            ui.register_part(
                self.id,
                PartRef::item(Part::OVERFLOW, ItemKey::index(1)),
                cell,
            );
        }
        if self.allow_new {
            let cell = Rect {
                x: row0.right().saturating_sub(new_w).max(row0.x),
                y: row0.y,
                width: new_w.min(row0.width),
                height: 1,
            };
            let ns = ov.style(
                ui,
                id,
                Family::TABS,
                Variant::DEFAULT,
                Part::NEW,
                StateFlags::empty(),
            );
            let inner = super::shift(cell, 1);
            ui.glyph(inner, ns.glyph.unwrap_or(GlyphRole::NewTab), ns.style);
            ui.register_part(self.id, PartRef::of(Part::NEW), cell);
        }
        ui.report_layout(self.id, LayoutFacts::new(fit, len, used.height, used.width));
        used
    }

    /// The natural size: the design's `tabs_height`.
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let h = ui.design().size.tabs_height;
        Size {
            min: (8, h),
            preferred: (c.max.0, h),
        }
        .fit(c)
    }
}

/// Columns of `row` painted with a non-blank symbol, measured from its left.
fn painted_width(ui: &mut Ui<'_>, row: Rect) -> u16 {
    ui.with_area(row, |ui| {
        let (buf, clip) = ui.raw();
        let mut last = 0u16;
        for x in clip.columns().map(|c| c.x) {
            if let Some(c) = buf.cell(Position::new(x, clip.y))
                && c.symbol() != " "
            {
                last = x.saturating_sub(clip.x).saturating_add(1);
            }
        }
        last
    })
}

impl<T, K, R> Bindings for Tabs<'_, T, K, R> {
    type Cmd = TabsCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<TabsCmd>] {
        self.table()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_targets_the_logical_tab_after_a_reorder() {
        let mut st = TabsState::default();
        let keys = |v: &[u64]| {
            let v: Vec<ItemKey> = v.iter().map(|n| ItemKey::num(*n)).collect();
            v
        };
        let a = keys(&[1, 2, 3]);
        let _ = st.reconcile(3, |i| a[i]);
        st.set_active(1, ItemKey::num(2));
        st.first = Some(ItemKey::num(1));
        // insert at 0 and reverse: the active and first keys survive
        let b = keys(&[9, 3, 2, 1]);
        let r = st.reconcile(4, |i| b[i]);
        assert_eq!(r, Reconciliation::Unchanged);
        assert_eq!(st.active(), Some(ItemKey::num(2)));
        assert_eq!(st.cursor(), Some(ItemKey::num(2)));
        assert_eq!(st.first(), Some(ItemKey::num(1)));
        assert_eq!(st.first_index, 3);
        // the active tab vanishes: the cursor's neighbour becomes active
        let c = keys(&[9, 3, 1]);
        let _ = st.reconcile(3, |i| c[i]);
        assert_eq!(st.active(), st.cursor());
        assert!(st.active().is_some());
        let mut buf = [0u8; 2];
        assert_eq!(digits(7, &mut buf), "7");
        assert_eq!(digits(42, &mut buf), "42");
        assert_eq!(digits(500, &mut buf), "99");
    }
}
