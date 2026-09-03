//! A downstream component using only `tui_next::author`
//! (`COMPONENT_ARCHITECTURE.md` §17 example 12, Scenario G).
//!
//! Theme resolution, focus, hover, press, dispatch, hit testing, capture,
//! cursor, layers, digest testing and the conformance suite are all
//! reachable from `author::` with no private access. The arithmetic is
//! written as in the document; the example is an external consumer and is
//! not held to the library's saturating-arithmetic rule.
#![expect(
    clippy::arithmetic_side_effects,
    reason = "verbatim from COMPONENT_ARCHITECTURE.md §17 example 12"
)]

use tui_next::author::{
    Binding, BindingState, Bindings, Chord, Cx, Family, Focusability, FrameRead, GlyphRole, Id,
    Intent, ItemKey, KeyCode, Part, PartRef, Phase, Rect, Response, StateFlags, Ui, Variant,
};

/// A segmented control: N labelled segments, one selected, roving cursor.
#[derive(Debug)]
pub struct Segmented<'a> {
    id: Id,
    labels: &'a [&'a str],
    variant: Variant,
}

/// Durable interaction state: the roving cursor and the chosen segment.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SegmentedState {
    /// The segment under the cursor.
    pub cursor: usize,
    /// The chosen segment.
    pub selected: usize,
}

/// What the control reports.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SegmentedAction {
    /// The cursor moved.
    Moved,
    /// A segment was chosen.
    Selected(ItemKey),
}

/// The const-constructible command a chord maps to. `update` turns it into a
/// `SegmentedAction` carrying the live key (§21 item 10, M11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegCmd {
    /// Move the cursor left.
    Prev,
    /// Move the cursor right.
    Next,
    /// Choose the segment under the cursor.
    Select,
}

const SEGMENT: Part = Part::custom("segment");
const F_SEGMENTED: Family = Family::custom("segmented");

const BINDINGS: &[Binding<SegCmd>] = &[
    Binding {
        chord: Chord::key(KeyCode::Left),
        cmd: SegCmd::Prev,
        label: "Prev",
        priority: 40,
        visible: true,
    },
    Binding {
        chord: Chord::key(KeyCode::Right),
        cmd: SegCmd::Next,
        label: "Next",
        priority: 40,
        visible: true,
    },
    Binding {
        chord: Chord::key(KeyCode::Enter),
        cmd: SegCmd::Select,
        label: "Select",
        priority: 80,
        visible: true,
    },
    Binding {
        chord: Chord::key(KeyCode::Char(' ')),
        cmd: SegCmd::Select,
        label: "Select",
        priority: 80,
        visible: false,
    },
];

impl Bindings for Segmented<'_> {
    type Cmd = SegCmd;
    fn bindings(&self, _s: BindingState) -> &'static [Binding<SegCmd>] {
        BINDINGS
    }
}

impl<'a> Segmented<'a> {
    /// The parts this control styles.
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, SEGMENT, Part::LABEL];

    /// A control over `labels`.
    pub fn new(id: Id, labels: &'a [&'a str]) -> Self {
        Self {
            id,
            labels,
            variant: Variant::DEFAULT,
        }
    }

    /// Set the variant.
    #[must_use]
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    /// The update phase.
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut SegmentedState) -> Response<SegmentedAction> {
        let mut r = Response::ignored();
        let n = self.labels.len();
        if n == 0 {
            return r.for_id(self.id);
        }
        // `cx.intents` borrows only the frozen queue (§21 item 6), so `cx`'s services stay
        // usable inside the loop; keys are matched through the SAME table the hint bar shows,
        // which is what `bindings_match_handled_keys` checks.
        for it in cx.intents(self.id) {
            match it {
                Intent::Key(k) => match BINDINGS
                    .iter()
                    .find(|b| b.chord == k.chord())
                    .map(|b| b.cmd)
                {
                    Some(SegCmd::Prev) => {
                        st.cursor = (st.cursor + n - 1) % n;
                        r = Response::action(SegmentedAction::Moved);
                    }
                    Some(SegCmd::Next) => {
                        st.cursor = (st.cursor + 1) % n;
                        r = Response::action(SegmentedAction::Moved);
                    }
                    Some(SegCmd::Select) => {
                        st.selected = st.cursor;
                        r = Response::action(SegmentedAction::Selected(ItemKey::index(
                            st.selected,
                        )));
                        cx.request_repaint();
                    }
                    None => {}
                },
                Intent::Pointer {
                    phase: Phase::Click,
                    part:
                        PartRef {
                            part,
                            item: Some(k),
                        },
                    ..
                } if part == SEGMENT => {
                    if let ItemKey::Index(i) = k {
                        st.cursor = i;
                        st.selected = i;
                    }
                    r = Response::action(SegmentedAction::Selected(k));
                }
                _ => {}
            }
        }
        r.for_id(self.id)
    }

    /// The draw phase.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &SegmentedState) -> Rect {
        if area.is_empty() {
            return area; // registers nothing (R5)
        }
        ui.register_control(self.id, area, Focusability::Focusable);
        let live = ui.state(self.id);
        let w = area.width / self.labels.len().max(1) as u16;
        for (i, label) in self.labels.iter().enumerate() {
            let cell = Rect {
                x: area.x + w * i as u16,
                y: area.y,
                width: w,
                height: area.height,
            };
            let mut s =
                live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE | StateFlags::HOVERED);
            if i == st.selected {
                s |= StateFlags::SELECTED;
            }
            if i == st.cursor && live.contains(StateFlags::FOCUSED) {
                s |= StateFlags::ACTIVE;
            }
            let r = ui.style(F_SEGMENTED, self.variant, SEGMENT, s);
            ui.fill(cell, r.style);
            if let Some(g) = r.glyph {
                ui.glyph(cell, g, r.style); // a declared part paints Resolved.glyph (A4)
            } else if s.contains(StateFlags::SELECTED) {
                ui.glyph(cell, GlyphRole::Chosen, r.style);
            }
            let ls = ui.style(F_SEGMENTED, self.variant, Part::LABEL, s).style;
            ui.paint_str(cell, label, ls);
            ui.register_part(self.id, PartRef::item(SEGMENT, ItemKey::index(i)), cell); // RegionKind::Part
        }
        area
    }
}

fn main() {}
