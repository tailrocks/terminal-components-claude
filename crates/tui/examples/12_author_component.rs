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
    Intent, ItemKey, Key, KeyCode, Part, PartRef, Phase, Rect, Resolved, Response, Slot,
    StateFlags, StylePatch, Ui, Variant,
};

/// A segmented control: N labelled segments, one selected, roving cursor.
///
/// ## Parts
/// `Part::CONTAINER` — the whole strip, painted first, so the columns the
/// integer division leaves over still carry the control's own surface.
/// `SEGMENT` (`Part::custom("segment")`) — one cell-valued part per label,
/// also the click target, registered as
/// `PartRef::item(SEGMENT, ItemKey::index(i))`.
/// `Part::LABEL` — the text drawn over each segment.
///
/// ## Overrides
/// `.patch_part(&[(Part, StylePatch)])` patches any of the three declared
/// parts at precedence 6, for this instance only. The slice is **borrowed**,
/// so the patches live in the caller's `const` table and no allocation
/// happens per frame. A patched part resolves through `Ui::style_patched`
/// and an unpatched one through `Ui::style`, so an override changes what
/// this instance paints and never mutates the `Theme`. There is no `.slot`:
/// a segment is a cell-valued part with no sub-painting to replace.
/// `.state_override(StateFlags)` is showcase / fixture use only (A11) — a
/// forced instance is a *reference rendering*: it paints the state it was
/// handed and registers no control and no part, so nothing on the page is
/// focusable or clickable.
#[derive(Debug)]
pub struct Segmented<'a> {
    id: Id,
    labels: &'a [&'a str],
    variant: Variant,
    parts: &'a [(Part, StylePatch)],
    forced: Option<StateFlags>,
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

/// The command a key press means, resolved against the SAME table the hint
/// bar shows. [`Binding::lookup`] is the library idiom: it matches through
/// `Chord::matches`, which folds `SHIFT` out for `KeyCode::Char`, so
/// `Shift+Space` still selects. A derived `b.chord == k.chord()` comparison
/// is strictly narrower and silently drops every shifted character chord.
fn command_for(k: &Key) -> Option<SegCmd> {
    Binding::lookup(BINDINGS, k)
}

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
            parts: &[],
            forced: None,
        }
    }

    /// Set the variant.
    #[must_use]
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    /// Per-part instance patches (precedence 6), borrowed from the caller.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.parts = ps;
        self
    }

    /// Showcase / fixture use only (A11): render a state without producing
    /// it. A forced instance registers no control and no part.
    #[must_use]
    pub const fn state_override(mut self, s: StateFlags) -> Self {
        self.forced = Some(s);
        self
    }

    /// The instance patch for `part`: every matching `.patch_part` entry
    /// merged in declaration order, or `None` when the caller patched
    /// nothing — in which case the plain resolution path is used.
    fn part_patch(&self, part: Part) -> Option<StylePatch> {
        let mut acc: Option<StylePatch> = None;
        for (p, patch) in self.parts {
            if *p == part {
                acc = Some(match acc {
                    Some(a) => a.merge(*patch),
                    None => *patch,
                });
            }
        }
        acc
    }

    /// Resolve one declared part through the whole precedence chain,
    /// including this instance's patches (precedence 6). Every part goes
    /// through here, which is what makes `.patch_part` reach the rendering
    /// without any component ever touching the `Theme`.
    fn style(&self, ui: &mut Ui<'_>, part: Part, flags: StateFlags) -> Resolved {
        let r = match self.part_patch(part) {
            Some(p) => ui.style_patched(F_SEGMENTED, self.variant, part, flags, &p),
            None => ui.style(F_SEGMENTED, self.variant, part, flags),
        };
        // Record what this component resolved, so the declared-parts check
        // can see it. `testing` is this package's feature; a component in a
        // crate of its own forwards a feature of its own to it.
        #[cfg(feature = "testing")]
        ui.note_styled(self.id, F_SEGMENTED, self.variant, part, r);
        r
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
                Intent::Key(k) => match command_for(&k) {
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
        // A forced instance is a reference rendering: it paints the state it
        // was handed and registers neither a control nor a part, so nothing
        // it draws is focusable, hoverable or clickable (A11).
        let live = if let Some(forced) = self.forced {
            forced
        } else {
            ui.register_control(self.id, area, Focusability::Focusable);
            ui.state(self.id)
        };
        // The container is the first declared part and is painted first: the
        // segments and their labels sit on top of it, and the columns the
        // integer division leaves over still carry the control's own surface.
        let container = self.style(ui, Part::CONTAINER, live);
        ui.fill(area, container.style);
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
            let r = self.style(ui, SEGMENT, s);
            ui.fill(cell, r.style);
            match r.glyph {
                Slot::Set(g) => {
                    ui.glyph(cell, g, r.style); // a declared part paints Resolved.glyph (A4)
                }
                Slot::Inherit if s.contains(StateFlags::SELECTED) => {
                    ui.glyph(cell, GlyphRole::Chosen, r.style);
                }
                Slot::Inherit | Slot::Clear => {}
            }
            let ls = self.style(ui, Part::LABEL, s).style;
            ui.paint_str(cell, label, ls);
            if self.forced.is_none() {
                // RegionKind::Part — never registered by a reference rendering
                ui.register_part(self.id, PartRef::item(SEGMENT, ItemKey::index(i)), cell);
            }
        }
        area
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use tui_next::author::{ColorLevel, KeyModifiers, Role, Theme};
    use tui_next_testing::Scene;

    const SEG: Id = Id::root("example.segmented");
    const LABELS: &[&str] = &["One", "Two", "Three"];
    const AREA: Rect = Rect {
        x: 0,
        y: 0,
        width: 20,
        height: 1,
    };

    /// A caller's per-instance override table: borrowed, `const`, allocated
    /// never.
    const LOUD_LABEL: &[(Part, StylePatch)] =
        &[(Part::LABEL, StylePatch::new().set_fg(Role::Danger))];

    fn scene(name: &'static str) -> Scene {
        Scene::new(name, Theme::junie(), ColorLevel::TrueColor, 24, 3)
    }

    fn styled_parts_of(scene: &mut Scene, seg: &Segmented<'_>) -> Vec<Part> {
        let st = SegmentedState::default();
        let mut out = Vec::new();
        scene.draw(|ui, _| {
            seg.draw(ui, AREA, &st);
            out = ui
                .styled_parts()
                .iter()
                .filter(|(owner, _)| *owner == SEG)
                .map(|(_, p)| *p)
                .collect();
        });
        out
    }

    #[test]
    fn the_first_declared_part_is_a_part_the_component_paints() {
        let seg = Segmented::new(SEG, LABELS);
        let mut sc = scene("segmented_container");
        let parts = styled_parts_of(&mut sc, &seg);
        assert!(
            parts.contains(&Part::CONTAINER),
            "PARTS declares {:?} first, but one draw styled only {parts:?}",
            Part::CONTAINER
        );
        for p in &parts {
            assert!(
                Segmented::PARTS.contains(p),
                "styled {p:?}, which is not in PARTS {:?}",
                Segmented::PARTS
            );
        }
    }

    #[test]
    fn an_instance_patch_changes_the_rendering_and_leaves_the_theme_alone() {
        let st = SegmentedState::default();
        let mut sc = scene("segmented_patch");
        let plain = |ui: &mut Ui<'_>, _: Rect| {
            Segmented::new(SEG, LABELS).draw(ui, AREA, &st);
        };

        // One warm-up frame: focus lands on the control the frame after it
        // first registers, so frame 1 and frame 2 legitimately differ. Every
        // digest compared below is taken from a settled frame.
        sc.draw(&plain);
        sc.draw(&plain);
        let before = sc.digest();

        sc.draw(|ui, _| {
            Segmented::new(SEG, LABELS)
                .patch_part(LOUD_LABEL)
                .draw(ui, AREA, &st);
        });
        let patched = sc.digest();

        sc.draw(&plain);
        let after = sc.digest();

        assert_ne!(
            before, patched,
            "`.patch_part` did not reach the rendering: both digests are {before:016x}"
        );
        assert_eq!(
            before, after,
            "the patch outlived its instance: an unpatched draw on the same theme now \
             digests {after:016x}, was {before:016x}"
        );
    }

    #[test]
    fn a_forced_instance_registers_no_control() {
        let st = SegmentedState::default();
        let mut sc = scene("segmented_forced");
        sc.draw(|ui, _| {
            Segmented::new(SEG, LABELS)
                .state_override(StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE)
                .draw(ui, AREA, &st);
        });
        assert!(
            sc.runtime().is_some_and(|rt| rt.area_of(SEG).is_none()),
            "a reference rendering registered a control (A11)"
        );
    }

    #[test]
    fn a_shifted_char_chord_still_resolves_to_its_command() {
        let shifted = Key {
            code: KeyCode::Char(' '),
            mods: KeyModifiers::SHIFT,
        };
        assert_eq!(
            command_for(&shifted),
            Some(SegCmd::Select),
            "Shift+Space was dropped: a derived `chord == k.chord()` is narrower than \
             `Chord::matches`, which folds SHIFT out for `KeyCode::Char`"
        );
        let plain = Key {
            code: KeyCode::Char(' '),
            mods: KeyModifiers::NONE,
        };
        assert_eq!(command_for(&plain), Some(SegCmd::Select));
        assert_eq!(
            command_for(&Key {
                code: KeyCode::Left,
                mods: KeyModifiers::SHIFT,
            }),
            None,
            "SHIFT is folded out for `Char` only; Shift+Left is a different chord"
        );
    }
}
