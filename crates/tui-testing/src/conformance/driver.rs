//! The 20 conformance cases (`COMPONENT_ARCHITECTURE.md` §16.2), generic
//! over [`Conformance`].

use std::collections::BTreeMap;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Style;
use tui_next::{
    Anchor, App, Axis, BindingState, Chord, CollectionCore, ColorLevel, CrossAlign, Cx, Diagnostic,
    Flow, Focusability, Id, Invalidate, ItemKey, KeyCode, KeyModifiers, LayerId, LayerKind,
    LayerSize, LayerSpec, MouseKind, Region, RegionKind, Response, Side, StateFlags, Theme, Ui,
};

use super::{Caps, Conformance, Fixture};
use crate::harness::{Harness, centre};

const SENTINEL_BEFORE: Id = Id::root("conformance.sentinel.before");
const SENTINEL_AFTER: Id = Id::root("conformance.sentinel.after");
const POPOVER: Id = Id::root("conformance.popover");
const POPOVER_CONTROL: Id = Id::root("conformance.popover.control");

/// The application wrapping one component under test.
pub struct CaseApp<C: Conformance> {
    /// The component's state.
    pub st: C::State,
    /// The fixture.
    pub fixture: Fixture,
    /// The last action.
    pub last: Option<C::Action>,
    /// The last flow/invalidate of the component's response.
    pub last_flow: (Flow, Invalidate),
    /// Draw sentinel controls before and after the component.
    pub sentinels: bool,
    /// Draw the component at all.
    pub show: bool,
    /// Draw the sentinels at all.
    pub show_sentinels: bool,
    /// Open a popover above the page on the next update.
    pub open_popover: bool,
    /// `update` calls.
    pub updates: usize,
}

impl<C: Conformance> core::fmt::Debug for CaseApp<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CaseApp")
            .field("st", &self.st)
            .field("last", &self.last)
            .finish_non_exhaustive()
    }
}

impl<C: Conformance> CaseApp<C> {
    /// A case app over `fixture`.
    pub fn new(fixture: Fixture) -> Self {
        CaseApp {
            st: C::State::default(),
            fixture,
            last: None,
            last_flow: (Flow::Ignored, Invalidate::None),
            sentinels: false,
            show: true,
            show_sentinels: true,
            open_popover: false,
            updates: 0,
        }
    }
}

impl<C: Conformance> App for CaseApp<C> {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.updates += 1;
        if self.open_popover {
            self.open_popover = false;
            let anchor = Anchor::Rect {
                rect: Rect::new(0, 0, 1, 1),
                side: Side::Below,
                align: CrossAlign::Start,
            };
            cx.open_layer(
                POPOVER,
                LayerSpec::popover(POPOVER, anchor).size(LayerSize::Fixed(6, 1)),
            );
        }
        let r = C::update(cx, &mut self.st, &self.fixture);
        self.last_flow = (r.flow(), r.invalidate());
        r.on_action(|a| self.last = Some(a))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.sentinels && self.show_sentinels {
            ui.register_control(
                SENTINEL_BEFORE,
                Rect::new(0, 0, 1, 1),
                Focusability::Focusable,
            );
        }
        if self.show {
            C::draw(ui, self.fixture.area, &self.st, &self.fixture);
        }
        if self.sentinels && self.show_sentinels {
            let a = ui.full();
            ui.register_control(
                SENTINEL_AFTER,
                Rect::new(0, a.bottom().saturating_sub(1), 1, 1),
                Focusability::Focusable,
            );
        }
        ui.layer(POPOVER, |ui, area| {
            ui.register_control(POPOVER_CONTROL, area, Focusability::Focusable);
        });
    }
}

fn harness<C: Conformance>(fixture: Fixture) -> Harness<CaseApp<C>> {
    let theme = fixture.theme.clone().downgrade(fixture.color);
    Harness::new(CaseApp::<C>::new(fixture), theme, 40, 12)
}

fn has<C: Conformance>(cap: Caps) -> bool {
    C::caps().contains(cap)
}

fn region_key(r: &Region) -> (Id, tui_next::PartRef, Rect, LayerId, RegionKind) {
    (r.owner, r.part, r.area, r.layer, r.kind)
}

fn regions_of<C: Conformance>(
    h: &Harness<CaseApp<C>>,
) -> Vec<(Id, tui_next::PartRef, Rect, LayerId, RegionKind)> {
    h.runtime()
        .registry()
        .regions()
        .iter()
        .map(region_key)
        .collect()
}

fn part_area<C: Conformance>(h: &Harness<CaseApp<C>>) -> Rect {
    h.area_of_part(C::id(), C::activation_part())
        .or_else(|| h.area_of(C::id()))
        .unwrap_or_else(|| panic!("{}: no area for the activation part", C::NAME))
}

/// Case 1.
pub fn disabled_cannot_activate<C: Conformance>() {
    if !has::<C>(Caps::DISABLEABLE) {
        return;
    }
    let mut f = Fixture::default();
    f.disabled = true;
    let mut h = harness::<C>(f);
    let before = h.app().st.clone();
    assert!(
        h.ring().is_registered(C::id()),
        "{}: a disabled control is still registered",
        C::NAME
    );
    assert!(
        !h.ring().contains(C::id()),
        "{}: a disabled control is never reachable",
        C::NAME
    );
    for chord in C::activation_chords() {
        let r = h.key_mod(chord.code, chord.mods);
        assert!(
            !r.is_consumed(),
            "{}: disabled control consumed {chord}",
            C::NAME
        );
        assert!(
            h.app().last.is_none(),
            "{}: disabled control activated on {chord}",
            C::NAME
        );
    }
    let area = part_area::<C>(&h);
    let (x, y) = centre(area);
    let r = h.click(x, y);
    assert!(
        h.app().last.is_none(),
        "{}: disabled control activated on click ({r:?})",
        C::NAME
    );
    assert_eq!(
        h.app().st,
        before,
        "{}: state changed while disabled",
        C::NAME
    );
}

/// Case 2.
pub fn keyboard_and_mouse_activation_are_equivalent<C: Conformance>() {
    if !has::<C>(Caps::ACTIVATES) {
        return;
    }
    let chords = C::activation_chords();
    assert!(
        !chords.is_empty(),
        "{}: ACTIVATES without activation chords",
        C::NAME
    );
    let mut mouse = harness::<C>(Fixture::default());
    assert!(mouse.tab_to(C::id()), "{}: cannot focus", C::NAME);
    let area = part_area::<C>(&mouse);
    let (x, y) = centre(area);
    let _ = mouse.click(x, y);
    let by_mouse = mouse.app_mut().last.take();
    let mouse_flow = mouse.app().last_flow;
    assert!(
        by_mouse.is_some(),
        "{}: a click over the activation part did nothing",
        C::NAME
    );
    for chord in chords {
        let mut kb = harness::<C>(Fixture::default());
        assert!(kb.tab_to(C::id()));
        let _ = kb.key_mod(chord.code, chord.mods);
        let by_key = kb.app_mut().last.take();
        assert_eq!(
            by_key,
            by_mouse,
            "{}: {chord} and a click produce different actions",
            C::NAME
        );
        assert_eq!(
            kb.app().last_flow,
            mouse_flow,
            "{}: flow/invalidate differ for {chord}",
            C::NAME
        );
    }
}

/// Case 3.
pub fn traversal_order_is_registration_order<C: Conformance>() {
    if !has::<C>(Caps::FOCUSABLE) {
        return;
    }
    let mut app = CaseApp::<C>::new(Fixture::default());
    app.sentinels = true;
    let h = Harness::new(app, Theme::junie(), 40, 12);
    let ids: Vec<Id> = h.ring().reachable().map(|e| e.id).collect();
    assert!(ids.len() >= 3, "{}: ring {ids:?}", C::NAME);
    assert_eq!(ids.first(), Some(&SENTINEL_BEFORE));
    assert_eq!(ids.last(), Some(&SENTINEL_AFTER));
    assert!(
        ids.contains(&C::id()),
        "{}: not in the ring: {ids:?}",
        C::NAME
    );
    for id in &ids {
        assert_eq!(h.ring().prev(h.ring().next(Some(*id))), Some(*id));
        assert_eq!(h.ring().next(h.ring().prev(Some(*id))), Some(*id));
    }
}

/// Case 4.
pub fn hover_does_not_steal_focus<C: Conformance>() {
    let mut app = CaseApp::<C>::new(Fixture::default());
    app.sentinels = true;
    let mut h = Harness::new(app, Theme::junie(), 40, 12);
    let focus = h.focus();
    let area = h.area_of(C::id()).unwrap_or(h.app().fixture.area);
    let mut hovered_once = false;
    for pos in area.positions() {
        let _ = h.mouse(MouseKind::Move, pos.x, pos.y);
        assert_eq!(
            h.focus(),
            focus,
            "{}: hover moved focus at {pos:?}",
            C::NAME
        );
        if h.state_of(C::id()).contains(StateFlags::HOVERED) {
            hovered_once = true;
        }
    }
    if has::<C>(Caps::FOCUSABLE) {
        assert!(hovered_once, "{}: never HOVERED over its own area", C::NAME);
        let (x, y) = centre(area);
        let _ = h.mouse(MouseKind::Move, x, y);
        assert!(h.state_of(C::id()).contains(StateFlags::HOVERED));
        let _ = h.key(KeyCode::Char('\u{1}'));
        assert!(
            !h.state_of(C::id()).contains(StateFlags::HOVERED),
            "{}: a key must suppress hover",
            C::NAME
        );
        let _ = h.mouse(MouseKind::Move, x, y);
        assert!(
            h.state_of(C::id()).contains(StateFlags::HOVERED),
            "{}: motion must restore hover",
            C::NAME
        );
    }
}

/// Case 5.
pub fn draw_twice_is_byte_identical<C: Conformance>() {
    let mut h = harness::<C>(Fixture::default()).with_auto_draw(false);
    h.draw();
    let a = h.buffer().clone();
    let ra = regions_of::<C>(&h);
    h.draw();
    let b = h.buffer().clone();
    let rb = regions_of::<C>(&h);
    assert_eq!(a, b, "{}: two draws differ", C::NAME);
    assert_eq!(ra, rb, "{}: two registries differ", C::NAME);
}

/// Case 6.
pub fn draw_twice_leaves_state_equal<C: Conformance>() {
    let variants: Vec<Fixture> = {
        let mut v = vec![Fixture::default()];
        let mut d = Fixture::default();
        d.disabled = true;
        v.push(d);
        v
    };
    for f in variants {
        let mut h = harness::<C>(f).with_auto_draw(false);
        let _ = h.tab_to(C::id());
        let before = h.app().st.clone();
        h.draw();
        h.draw();
        assert_eq!(h.app().st, before, "{}: draw changed the state", C::NAME);
        let area = h.area_of(C::id()).unwrap_or(h.app().fixture.area);
        let (x, y) = centre(area);
        let _ = h.mouse(MouseKind::Move, x, y);
        let hovered = h.app().st.clone();
        h.draw();
        h.draw();
        assert_eq!(
            h.app().st,
            hovered,
            "{}: draw changed the state while hovered",
            C::NAME
        );
    }
}

/// Case 7.
pub fn draw_does_not_commit_or_cancel<C: Conformance>() {
    if !has::<C>(Caps::EDITS) {
        return;
    }
    let mut h = harness::<C>(Fixture::default());
    assert!(h.tab_to(C::id()));
    let _ = h.type_str("x");
    h.blur();
    let mid = h.app().st.clone();
    let last = h.app().last.is_some();
    h.draw();
    h.draw();
    h.draw();
    assert_eq!(h.app().st, mid, "{}: draw committed or cancelled", C::NAME);
    assert_eq!(h.app().last.is_some(), last);
}

/// Case 8.
pub fn draw_stays_inside_its_area<C: Conformance>() {
    let f = Fixture::default();
    let inner = f.area;
    let mut scene = crate::Scene::new(C::NAME, f.theme.clone(), f.color, 40, 12);
    let st = C::State::default();
    scene.draw_over(
        |buf| buf.set_style(*buf.area(), Style::new()),
        |ui, _| {
            let (buf, _) = ui.raw();
            for pos in buf.area().positions() {
                if let Some(c) = buf.cell_mut(pos) {
                    c.set_symbol("X");
                }
            }
            C::draw(ui, inner, &st, &f);
        },
    );
    for pos in scene.area().positions() {
        if inner.contains(pos) {
            continue;
        }
        let sym = scene
            .buffer()
            .cell(pos)
            .map(|c| c.symbol().to_owned())
            .unwrap_or_default();
        assert_eq!(sym, "X", "{}: wrote outside its area at {pos:?}", C::NAME);
    }
    if let Some(reg) = scene.registry() {
        for r in reg.regions() {
            assert_eq!(
                r.area,
                r.area.intersection(inner),
                "{}: region {:?} escapes the area",
                C::NAME,
                r.area
            );
        }
    }
}

fn symbol_modifier_multiset(buf: &Buffer, area: Rect) -> BTreeMap<(String, u16), usize> {
    let mut out = BTreeMap::new();
    for pos in area.positions() {
        if let Some(c) = buf.cell(pos) {
            *out.entry((c.symbol().to_owned(), c.modifier.bits()))
                .or_insert(0) += 1;
        }
    }
    out
}

/// Case 9.
pub fn mono_states_are_distinguishable<C: Conformance>() {
    let states = C::mono_states();
    // MA-8: `mono_states()` may only NARROW the default ten, and it may not
    // drop a state the component's own capabilities imply.
    for s in states {
        assert!(
            super::DEFAULT_MONO_STATES.contains(s),
            "{}: mono_states() may only narrow DEFAULT_MONO_STATES; {s:?} is not in it",
            C::NAME
        );
    }
    for s in super::mono_states_required_by(C::caps()) {
        assert!(
            states.contains(&s),
            "{}: caps {:?} imply {s:?}, which mono_states() dropped",
            C::NAME,
            C::caps()
        );
    }
    let dropped: Vec<StateFlags> = super::DEFAULT_MONO_STATES
        .iter()
        .copied()
        .filter(|s| !states.contains(s))
        .collect();
    let why = C::mono_narrowing_reason();
    assert_eq!(
        dropped.is_empty(),
        why.is_empty(),
        "{}: mono_narrowing_reason() must be non-empty exactly when mono_states() narrows",
        C::NAME
    );
    for s in &dropped {
        for (name, _) in s.iter_names() {
            assert!(
                why.contains(name),
                "{}: mono_states() drops {name}, and mono_narrowing_reason() does not say why",
                C::NAME
            );
        }
    }
    let mut seen: Vec<(StateFlags, BTreeMap<(String, u16), usize>)> = Vec::new();
    for s in states {
        // `force` sets the props the forced state implies too, so a state
        // whose affordance is a painted symbol is actually reachable here.
        let mut f = Fixture::default().force(*s);
        f.color = ColorLevel::Mono;
        // a sentinel holds the real focus so the forced state is the only state
        let theme = f.theme.clone().downgrade(f.color);
        let mut app = CaseApp::<C>::new(f);
        app.sentinels = true;
        let h = Harness::new(app, theme, 40, 12);
        let area = h.app().fixture.area;
        let ms = symbol_modifier_multiset(h.buffer(), area);
        for (other, prev) in &seen {
            assert_ne!(
                *prev,
                ms,
                "{}: mono output of {s:?} equals {other:?}\n{}",
                C::NAME,
                h.text()
            );
        }
        seen.push((*s, ms));
    }
}

/// Case 10.
pub fn local_override_does_not_mutate_the_theme<C: Conformance>() {
    let plain = harness::<C>(Fixture::default());
    let before = plain.runtime().theme().fingerprint();
    let plain_digest = plain.snapshot().digest();
    let mut f = Fixture::default();
    f.patch = Some((
        C::PARTS
            .first()
            .copied()
            .unwrap_or(tui_next::Part::CONTAINER),
        tui_next::StylePatch::new().set_fg(tui_next::Role::Warning),
    ));
    let patched = harness::<C>(f);
    assert_eq!(
        patched.runtime().theme().fingerprint(),
        before,
        "{}: the theme changed",
        C::NAME
    );
    assert_eq!(plain.runtime().theme().fingerprint(), before);
    if patched.app().fixture.patch.is_some() && C::caps().bits() != 0 {
        // a component that honours `patch_part` renders differently; one that
        // declares no parts cannot, and is exempt
        if !C::PARTS.is_empty() {
            assert_ne!(
                patched.snapshot().digest(),
                plain_digest,
                "{}: the instance patch had no effect",
                C::NAME
            );
        }
    }
}

/// Case 11.
pub fn id_separator_collision_free<C: Conformance>() {
    let h = harness::<C>(Fixture::default());
    let dups = h
        .diagnostics()
        .iter()
        .filter(|d| matches!(d, Diagnostic::DuplicateId { .. }))
        .count();
    assert_eq!(dups, 0, "{}: duplicate ids: {:?}", C::NAME, h.diagnostics());
    let regions = regions_of::<C>(&h);
    let controls: Vec<Id> = regions
        .iter()
        .filter(|r| r.4 == RegionKind::Control)
        .map(|r| r.0)
        .collect();
    let mut sorted = controls.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        controls.len(),
        "{}: a control id registered twice",
        C::NAME
    );
}

/// Case 12.
#[expect(
    clippy::too_many_lines,
    reason = "one case: click identity across a reorder plus the retention half (MA-9)"
)]
pub fn item_identity_survives_reorder<C: Conformance>() {
    if !has::<C>(Caps::COLLECTION) {
        return;
    }
    let mut h = harness::<C>(Fixture::default());
    let keys = C::item_keys(&h.app().fixture);
    assert!(keys.len() >= 2, "{}: need at least two rows", C::NAME);
    let k1 = keys[0];
    let _ = h.click_part(C::id(), C::row_part(k1));
    let a = h.app_mut().last.take();
    assert_eq!(
        a.as_ref().and_then(C::action_key_of),
        Some(k1),
        "{}: click did not name k1",
        C::NAME
    );
    // reverse permutation
    let n = keys.len();
    let perm: Vec<usize> = (0..n).rev().collect();
    C::reorder(&mut h.app_mut().fixture, &perm);
    h.draw();
    let _ = h.click_part(C::id(), C::row_part(k1));
    let b = h.app_mut().last.take();
    assert_eq!(
        b.as_ref().and_then(C::action_key_of),
        Some(k1),
        "{}: k1 lost after reverse",
        C::NAME
    );
    // insert + remove
    let mut rows = h.app().fixture.rows.clone();
    rows.remove(0);
    rows.insert(
        0,
        super::FixtureRow {
            key: ItemKey::num(9_999),
            label: "new".to_owned(),
            meta: String::new(),
            disabled: false,
        },
    );
    h.app_mut().fixture.rows = rows;
    h.draw();
    if h.area_of_part(C::id(), C::row_part(k1)).is_some() {
        let _ = h.click_part(C::id(), C::row_part(k1));
        let c = h.app_mut().last.take();
        assert_eq!(
            c.as_ref().and_then(C::action_key_of),
            Some(k1),
            "{}: k1 lost after insert+remove",
            C::NAME
        );
    }
    // MA-9: click identity alone is not the case. Set cursor and checked on
    // k1 and k2 and assert they *survive* the same reorder — the retention
    // half §16.2 case 12 specifies, exercised against the `CollectionCore`
    // every keyed collection embeds.
    let k2 = keys[1];
    let mut core = CollectionCore::new();
    let ordered = |ks: &[ItemKey]| {
        let owned: Vec<ItemKey> = ks.to_vec();
        move |i: usize| owned.get(i).copied().unwrap_or(ItemKey::index(0))
    };
    core.reconcile_with(keys.len(), ordered(&keys), |_| true);
    core.set_cursor(0, k1);
    core.checked_mut().insert(k1);
    core.checked_mut().insert(k2);

    let reversed: Vec<ItemKey> = keys.iter().rev().copied().collect();
    core.reconcile_with(reversed.len(), ordered(&reversed), |_| true);
    assert_eq!(
        core.cursor(),
        Some(k1),
        "{}: the cursor must still name k1 after a reverse permutation",
        C::NAME
    );
    assert_eq!(
        core.cursor_index(),
        reversed.len().saturating_sub(1),
        "{}: the cursor index must follow k1 to its new position",
        C::NAME
    );
    assert!(
        core.checked().contains(k1) && core.checked().contains(k2),
        "{}: the checked set must still name k1 and k2 after a reorder",
        C::NAME
    );

    // insert + remove: k1 disappears, k2 survives, and the cursor moves to a
    // surviving neighbour rather than to a stale index
    let mut after: Vec<ItemKey> = reversed.clone();
    after.retain(|k| *k != k1);
    after.insert(0, ItemKey::num(9_998));
    let dropped = core.reconcile_with(after.len(), ordered(&after), |_| true);
    assert!(
        core.cursor().is_some_and(|c| after.contains(&c)),
        "{}: the cursor must land on a surviving key",
        C::NAME
    );
    assert!(
        core.checked().contains(k2),
        "{}: k2 was still present and must stay checked",
        C::NAME
    );
    assert!(
        !core.checked().contains(k1),
        "{}: a vanished key must leave the checked set",
        C::NAME
    );
    assert!(
        matches!(
            dropped,
            tui_next::Reconciliation::SelectionDropped(1)
                | tui_next::Reconciliation::CursorMoved(_)
        ),
        "{}: the reconcile must report the dropped checked key or the moved cursor, got {dropped:?}",
        C::NAME
    );
}

/// Case 13.
pub fn focus_reconcile_follows_the_rule<C: Conformance>() {
    if !has::<C>(Caps::FOCUSABLE) {
        return;
    }
    let mut app = CaseApp::<C>::new(Fixture::default());
    app.sentinels = true;
    let mut h = Harness::new(app, Theme::junie(), 40, 12);
    assert!(h.tab_to(C::id()));
    // (a) nearest surviving entry by previous index: the sentinel after it
    h.app_mut().show = false;
    h.draw();
    assert_eq!(
        h.focus(),
        Some(SENTINEL_AFTER),
        "{}: (a) nearest survivor",
        C::NAME
    );
    // (b)/(c) the scope's first enabled entry when the neighbours vanish too
    h.app_mut().show = true;
    h.draw();
    assert!(h.tab_to(C::id()));
    h.app_mut().show = false;
    h.app_mut().show_sentinels = false;
    h.draw();
    assert_eq!(h.focus(), None, "{}: (d) nothing reachable", C::NAME);
    h.app_mut().show_sentinels = true;
    h.draw();
    assert_eq!(
        h.focus(),
        Some(SENTINEL_BEFORE),
        "{}: (c) innermost scope's first entry",
        C::NAME
    );
}

/// Case 14.
pub fn focus_trap_and_restore<C: Conformance>() {
    let traps_focus = has::<C>(Caps::TRAPS_FOCUS);
    assert!(
        !traps_focus || has::<C>(Caps::OVERLAY),
        "{}: TRAPS_FOCUS requires OVERLAY",
        C::NAME
    );
    if !has::<C>(Caps::OVERLAY) {
        return;
    }
    let mut app = CaseApp::<C>::new(Fixture::default());
    app.sentinels = true;
    let mut h = Harness::new(app, Theme::junie(), 40, 12);
    assert!(h.tab_to(C::id()));
    let prior_focus = h.focus();
    let chord =
        C::open_chord().unwrap_or_else(|| panic!("{}: OVERLAY without an open chord", C::NAME));
    let _ = h.key_mod(chord.code, chord.mods);
    let layer = C::layer_id().unwrap_or_else(|| panic!("{}: OVERLAY without a layer id", C::NAME));
    assert!(
        h.is_open(layer),
        "{}: {chord} did not open the layer",
        C::NAME
    );
    // §29.8 modification 1: the OVERLAY/TRAPS_FOCUS split polices itself in
    // both directions, so the trap half can never again be skipped by omission
    // and the capability can never be a lie about the runtime.
    let kind = h
        .runtime()
        .open_spec(layer)
        .unwrap_or_else(|| {
            panic!(
                "{}: the layer it just opened has no live spec in the runtime",
                C::NAME
            )
        })
        .kind;
    assert!(
        traps_focus || kind != LayerKind::Modal,
        "{}: opens a `LayerKind::Modal` without declaring `Caps::TRAPS_FOCUS`. A modal owns a \
         focus scope (`LayerKind::Modal => ScopeMode::Trap`), so add `Caps::TRAPS_FOCUS` to \
         `{}::caps()`; until then case 14's trap-leak, non-empty-trap, Tab-wrap and \
         zero-size-still-traps assertions never execute for this component",
        C::NAME,
        C::NAME
    );
    assert!(
        !traps_focus || h.ring().active_trap().is_some(),
        "{}: declares `Caps::TRAPS_FOCUS` but no `ScopeMode::Trap` is armed once {chord} has \
         opened the layer (its kind is {kind:?}). Either open a `LayerKind::Modal` or push a \
         trap scope of the component's own, or drop `Caps::TRAPS_FOCUS` from `{}::caps()`",
        C::NAME,
        C::NAME
    );
    if traps_focus {
        let inside: Vec<Id> = h.ring().reachable().map(|e| e.id).collect();
        assert!(
            !inside.contains(&SENTINEL_BEFORE) && !inside.contains(&SENTINEL_AFTER),
            "{}: the trap leaks — a control outside the open layer is still reachable. \
             Reachable inside the trap: {inside:?}, which must contain neither \
             {SENTINEL_BEFORE:?} nor {SENTINEL_AFTER:?}",
            C::NAME
        );
        // §29.8 modification 2: an empty trap would satisfy the leak assertion
        // vacuously while focus reconciles to `None` and the opener loses
        // `FOCUSED` under its own layer. Reject it, and check the wrap
        // unconditionally rather than only when there is more than one stop.
        assert!(
            !inside.is_empty(),
            "{}: the armed trap has no reachable focus stop. Nothing drawn inside the open \
             layer registered a focusable control, so focus reconciles to `None` and the \
             opener loses `FOCUSED` while its own layer is open; register at least one \
             focusable control inside the layer",
            C::NAME
        );
        let first = h.focus();
        assert!(
            first.is_some(),
            "{}: focus is `None` inside a non-empty trap over {inside:?}",
            C::NAME
        );
        for _ in 0..inside.len() {
            let _ = h.key(KeyCode::Tab);
        }
        assert_eq!(
            h.focus(),
            first,
            "{}: Tab does not wrap inside the trap — {} Tab press(es) over the {} reachable \
             stop(s) {inside:?} must return focus to where it started",
            C::NAME,
            inside.len(),
            inside.len()
        );
        for _ in 0..inside.len() {
            let _ = h.key(KeyCode::BackTab);
        }
        assert_eq!(
            h.focus(),
            first,
            "{}: reverse-Tab does not wrap inside the trap — {} reverse-Tab press(es) over the \
             {} reachable stop(s) {inside:?} must return focus to where it started",
            C::NAME,
            inside.len(),
            inside.len()
        );
    }
    let _ = h.key(KeyCode::Esc);
    assert!(
        !h.is_open(layer),
        "{}: Esc did not close the layer",
        C::NAME
    );
    assert_eq!(
        h.focus(),
        prior_focus,
        "{}: focus not restored to the pre-open focus",
        C::NAME
    );
    if traps_focus {
        // a layer that cannot draw still traps
        let _ = h.key_mod(chord.code, chord.mods);
        let _ = h.resize(1, 1);
        assert!(h.is_open(layer));
        assert!(h.top_layer() > LayerId::PAGE);
    }
}

/// Case 15.
pub fn pointer_capture_delivers_drag_and_release<C: Conformance>() {
    if !has::<C>(Caps::CAPTURES) {
        return;
    }
    let mut h = harness::<C>(Fixture::default());
    let area = part_area::<C>(&h);
    let (x, y) = centre(area);
    let _ = h.mouse(MouseKind::Down, x, y);
    assert_eq!(
        h.runtime().capture_owner(),
        Some(C::id()),
        "{}: press did not claim capture",
        C::NAME
    );
    let _ = h.mouse(MouseKind::Drag, 39, 11);
    assert_eq!(
        h.runtime().capture_owner(),
        Some(C::id()),
        "{}: capture lost during a drag outside",
        C::NAME
    );
    assert!(h.state_of(C::id()).contains(StateFlags::PRESSED));
    h.app_mut().last = None;
    let _ = h.mouse(MouseKind::Up, 39, 11);
    assert_eq!(
        h.runtime().capture_owner(),
        None,
        "{}: capture not released",
        C::NAME
    );
    assert!(
        h.app().last.is_none(),
        "{}: release outside the captured area activated",
        C::NAME
    );
}

/// Case 16.
pub fn wheel_at_boundary_is_consumed_without_repaint<C: Conformance>() {
    if !has::<C>(Caps::SCROLLS) {
        return;
    }
    let mut h = harness::<C>(Fixture::default());
    let focus = h.focus();
    let area = h.area_of(C::id()).unwrap_or(h.app().fixture.area);
    let (x, y) = centre(area);
    let r = h.wheel(Axis::V, -1, x, y);
    assert!(
        r.is_consumed() && !r.is_changed(),
        "{}: wheel at the top must be consumed without repaint",
        C::NAME
    );
    let r = h.wheel(Axis::V, 1, x, y);
    assert!(r.is_consumed(), "{}: wheel down not consumed", C::NAME);
    assert_eq!(h.focus(), focus, "{}: wheel moved focus", C::NAME);
}

/// Case 17.
pub fn cursor_write_is_rejected_off_top_layer<C: Conformance>() {
    if !has::<C>(Caps::CURSOR) {
        return;
    }
    let mut h = harness::<C>(Fixture::default());
    assert!(h.tab_to(C::id()));
    assert!(
        h.cursor().is_some(),
        "{}: no cursor while focused on the top layer",
        C::NAME
    );
    h.app_mut().open_popover = true;
    let _ = h.tick();
    assert!(h.is_open(POPOVER));
    let _ = h.tick();
    assert!(
        h.diagnostics()
            .iter()
            .any(|d| matches!(d, Diagnostic::CursorRejected { owner, .. } if *owner == C::id())),
        "{}: no CursorRejected under a popover: {:?}",
        C::NAME,
        h.diagnostics()
    );
}

/// Case 18.
pub fn secret_never_appears_in_debug<C: Conformance>() {
    if !has::<C>(Caps::SECRET) {
        return;
    }
    let secret = C::secret_bytes();
    assert!(
        !secret.is_empty(),
        "{}: SECRET without secret bytes",
        C::NAME
    );
    let mut f = Fixture::default();
    f.secret = Some(secret);
    let mut h = harness::<C>(f);
    assert!(h.tab_to(C::id()));
    let _ = h.type_str(secret);
    let dbg = format!("{:?}", h.app().st);
    assert!(!dbg.contains(secret), "{}: secret in Debug: {dbg}", C::NAME);
    assert!(!h.text().contains(secret), "{}: secret on screen", C::NAME);
    let d1 = h.snapshot().digest();
    let other: String = secret.chars().map(|_| 'z').collect();
    let mut f2 = Fixture::default();
    f2.secret = Some(secret);
    let mut h2 = harness::<C>(f2);
    assert!(h2.tab_to(C::id()));
    let _ = h2.type_str(&other);
    assert_eq!(
        h2.snapshot().digest(),
        d1,
        "{}: the digest depends on the secret",
        C::NAME
    );
}

/// Case 19.
pub fn survives_tiny_rects_0x0_to_3x3<C: Conformance>() {
    for w in 0..=3u16 {
        for hgt in 0..=3u16 {
            let mut f = Fixture::default();
            f.area = Rect::new(5, 5, w, hgt);
            let inner = f.area;
            let mut scene = crate::Scene::new(C::NAME, f.theme.clone(), f.color, 12, 12);
            let st = C::State::default();
            scene.draw_over(
                |_| {},
                |ui, _| {
                    let (buf, _) = ui.raw();
                    for pos in buf.area().positions() {
                        if let Some(c) = buf.cell_mut(pos) {
                            c.set_symbol("X");
                        }
                    }
                    C::draw(ui, inner, &st, &f);
                },
            );
            for pos in scene.area().positions() {
                if inner.contains(pos) {
                    continue;
                }
                let sym = scene
                    .buffer()
                    .cell(pos)
                    .map(|c| c.symbol().to_owned())
                    .unwrap_or_default();
                assert_eq!(sym, "X", "{}: {w}x{hgt} wrote outside at {pos:?}", C::NAME);
            }
            if let Some(reg) = scene.registry() {
                for r in reg.regions() {
                    assert_eq!(
                        r.area,
                        r.area.intersection(inner),
                        "{}: {w}x{hgt} region escapes",
                        C::NAME
                    );
                }
                if inner.is_empty() {
                    assert!(
                        reg.hit(Position::new(5, 5)).is_none(),
                        "{}: stale geometry after a 0x0 frame",
                        C::NAME
                    );
                }
            }
        }
    }
}

fn chord_universe() -> Vec<Chord> {
    let mut out = Vec::new();
    let mut codes: Vec<KeyCode> = ('a'..='z').map(KeyCode::Char).collect();
    codes.extend(('0'..='9').map(KeyCode::Char));
    codes.extend([
        KeyCode::Char(' '),
        KeyCode::Enter,
        KeyCode::Esc,
        KeyCode::Backspace,
        KeyCode::Delete,
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Home,
        KeyCode::End,
        KeyCode::PageUp,
        KeyCode::PageDown,
        KeyCode::Insert,
    ]);
    codes.extend((1..=12).map(KeyCode::F));
    for code in codes {
        for mods in [
            KeyModifiers::NONE,
            KeyModifiers::CONTROL,
            KeyModifiers::ALT,
            KeyModifiers::SHIFT,
        ] {
            out.push(Chord::with(code, mods));
        }
    }
    out
}

/// Case 20.
pub fn bindings_match_handled_keys<C: Conformance>() {
    if !has::<C>(Caps::FOCUSABLE) {
        return;
    }
    let states = [
        BindingState::default(),
        BindingState {
            flags: StateFlags::FOCUSED,
        },
    ];
    for st in states {
        let table = C::bindings(st);
        // every declared chord is consumed
        for b in table {
            let mut h = harness::<C>(Fixture::default());
            assert!(h.tab_to(C::id()));
            let r = h.key_mod(b.chord.code, b.chord.mods);
            assert!(
                r.is_consumed(),
                "{}: declared chord {} not consumed",
                C::NAME,
                b.chord
            );
        }
        // every consumed chord is declared (bare Char exempt for TYPES)
        for chord in chord_universe() {
            if chord.code == KeyCode::Esc {
                continue;
            }
            if has::<C>(Caps::TYPES) && chord.is_bare_char() {
                continue;
            }
            let mut h = harness::<C>(Fixture::default());
            assert!(h.tab_to(C::id()));
            let r = h.key_mod(chord.code, chord.mods);
            if r.is_consumed() {
                let key = tui_next::Key {
                    code: chord.code,
                    mods: chord.mods,
                };
                assert!(
                    table.iter().any(|b| b.chord.matches(&key)),
                    "{}: consumed {chord} which is not in the binding table",
                    C::NAME
                );
            }
        }
    }
}
