//! `KeyHint` — the `key Action` pair (`COMPONENT_ARCHITECTURE.md` §18.2,
//! Appendix A 4G).

use core::fmt::{self, Write as _};

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn, shift};
use crate::event::Chord;
use crate::id::{Id, Part};
use crate::keymap::Hint;
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{Family, StylePatch, Variant};
use crate::ui::Ui;

/// Bytes reserved for a rendered chord label. The longest chord the
/// [`Chord`] `Display` can produce is `Ctrl+Alt+Shift+Backspace` (24 bytes);
/// the arrow keys are three bytes each and far shorter.
const CHORD_CAP: usize = 32;

/// A [`Chord`]'s label rendered into a fixed stack buffer.
///
/// `Chord` is `Display`, but a component may not allocate once per frame per
/// hint (§20.9-6, R5), and there is no `Ui::paint_fmt`. Writing into a fixed
/// buffer keeps the hint bar allocation-free; a chord whose label does not
/// fit keeps the prefix that does, which cannot split a `char` because only
/// whole `write_str` fragments are appended.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ChordText {
    buf: [u8; CHORD_CAP],
    len: usize,
}

impl ChordText {
    /// Render `c`.
    pub(crate) fn of(c: Chord) -> Self {
        let mut t = ChordText {
            buf: [0; CHORD_CAP],
            len: 0,
        };
        // `write!` on a `fmt::Write` that never errors cannot fail
        let _ = write!(t, "{c}");
        t
    }

    /// The rendered label.
    pub(crate) fn as_str(&self) -> &str {
        self.buf
            .get(..self.len)
            .and_then(|b| core::str::from_utf8(b).ok())
            .unwrap_or("")
    }
}

impl fmt::Write for ChordText {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len.saturating_add(s.len());
        if end > CHORD_CAP {
            // drop the fragment whole: a partial copy could split a `char`
            return Ok(());
        }
        if let Some(dst) = self.buf.get_mut(self.len..end) {
            dst.copy_from_slice(s.as_bytes());
            self.len = end;
        }
        Ok(())
    }
}

/// One `key Action` pair: the chord in the key tone, the label beside it.
///
/// ## Construction
/// `KeyHint::new(id, chord, label)`; [`KeyHint::from_hint`] builds one from a
/// [`Hint`] as the hint bar derives it from a component's bindings (§13.1).
///
/// ## Ownership
/// Stateless. The caller owns nothing; the chord and label are borrowed or
/// `Copy`. The runtime owns nothing for this component: it registers no
/// region and no ring entry.
///
/// ## Configuration
/// `.variant(Variant)` (default `Recipe.default_variant`), `.patch`,
/// `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::KEYHINT`; `DEFAULT` only. A theme may define more.
///
/// ## States
/// None. A key hint is chrome: it is never focused, hovered, pressed or
/// disabled, and it derives nothing.
///
/// ## Actions
/// None; `KeyHint` has no `update` phase.
///
/// ## Focus
/// Never a focus stop; `Focusability` does not apply and `autofocus` does not
/// exist.
///
/// ## Keyboard
/// None. The chord it *renders* is bound by whichever component declared it;
/// the hint never handles a key itself.
///
/// ## Mouse
/// None; no `PartRef` is registered, so no pointer intent is delivered.
///
/// ## Layout
/// `measure` returns `(chord + 1 + label, 1)`. `draw` uses the first row of
/// `area`, clipped to that width, and returns the rect it painted; a
/// degenerate rect paints nothing and returns it unchanged (R5).
///
/// ## Parts
/// `KEY` (the chord), `ACTION` (the label).
///
/// ## Overrides
/// `.patch` and `.patch_part` reach `Part::KEY` and `Part::ACTION`.
/// `.slot(p, …)` changes painted cells for exactly `Part::KEY` and
/// `Part::ACTION` — every part `KeyHint` declares, so nothing is excluded,
/// and there is no `CONTAINER` to exclude because a hint paints no fill of
/// its own and sits on whatever surface its owner laid down. `ACTION` is
/// painted, and therefore slot-addressable, only when the label is
/// non-empty and the clipped row still has columns after the chord and its
/// separating space; a hint built with an empty label paints no `ACTION`
/// cell for a slot to replace.
///
/// ## Identity
/// One `Id` per instance, used only to attribute style resolution and
/// overrides; no items.
///
/// ## Testing
/// `KeyHintCase` with no capabilities;
/// `render::components::key_hint::{default, disabled, empty}`;
/// `keyhint::a_slot_changes_painted_cells_for_exactly_key_and_action`, which
/// asserts Invariant R (§45.3) in both directions — a named part that stops
/// changing cells and an unnamed part that starts changing them each fail
/// it.
///
/// ## Invariants
/// Never allocates: the chord is rendered into a fixed stack buffer. Never
/// writes outside `area`.
pub struct KeyHint<'a> {
    id: Id,
    chord: Chord,
    label: &'a str,
    variant: Variant,
    ov: PartStyle<'a>,
}

impl fmt::Debug for KeyHint<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyHint")
            .field("id", &self.id)
            .field("chord", &self.chord)
            .field("label", &self.label)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<'a> KeyHint<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[Part::KEY, Part::ACTION];

    /// A hint showing `chord` and `label`.
    pub const fn new(id: Id, chord: Chord, label: &'a str) -> Self {
        KeyHint {
            id,
            chord,
            label,
            variant: Variant::DEFAULT,
            ov: PartStyle::new(),
        }
    }

    /// The hint a [`Hint`] describes — the shape `HintBar` derives from a
    /// component's visible bindings (§13.1).
    pub const fn from_hint(id: Id, h: &Hint) -> KeyHint<'static> {
        KeyHint {
            id,
            chord: h.chord,
            label: h.label,
            variant: Variant::DEFAULT,
            ov: PartStyle::new(),
        }
    }

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// Set the variant.
    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    /// An instance patch over every part (precedence 6).
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    /// Per-part patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    /// Replace one part's painting; layout stays.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// The columns this hint occupies: the chord, one space, the label.
    pub fn width(&self) -> u16 {
        let key = width(ChordText::of(self.chord).as_str());
        if self.label.is_empty() {
            return key;
        }
        key.saturating_add(1).saturating_add(width(self.label))
    }

    /// The draw phase; returns the rect painted.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = Rect {
            width: self.width().min(area.width),
            height: u16::from(area.height != 0),
            ..area
        };
        if area.is_empty() {
            return area;
        }
        // neither half: a key hint is static chrome
        let live = PartStyle::flags(StateFlags::empty(), StateFlags::empty());
        let ov = self.ov;
        let key_text = ChordText::of(self.chord);
        let key_cell = Rect {
            width: width(key_text.as_str()).min(area.width),
            ..area
        };
        if let Some(f) = ov.slot_for(Part::KEY) {
            f(ui, key_cell);
        } else {
            let s = ov.style(ui, self.id, Family::KEYHINT, self.variant, Part::KEY, live);
            ui.paint_str(key_cell, key_text.as_str(), s.style);
        }
        let rest = shift(area, key_cell.width.saturating_add(1));
        if !rest.is_empty() {
            if let Some(f) = ov.slot_for(Part::ACTION) {
                f(ui, rest);
            } else {
                let s = ov.style(
                    ui,
                    self.id,
                    Family::KEYHINT,
                    self.variant,
                    Part::ACTION,
                    live,
                );
                ui.paint_str(rest, self.label, s.style);
            }
        }
        area
    }

    /// The natural size: one row, the chord plus the label.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size::exact(self.width(), 1).fit(c)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::event::{KeyCode, KeyModifiers};
    use crate::response::Response;
    use crate::runtime::stub::SCREEN;
    use crate::runtime::{App, Runtime};
    use crate::theme::Theme;
    use crate::ui::Cx;

    const HINT: Id = Id::root("keyhint.tests");
    const LABEL: &str = "Open";

    /// Every part the slot sweep installs on, and whether `KeyHint` is
    /// expected to honour it: the two parts it declares, then four it does
    /// not, so the sweep can fail in either direction.
    const SWEPT: &[(Part, bool)] = &[
        (Part::KEY, true),
        (Part::ACTION, true),
        (Part::CONTAINER, false),
        (Part::LABEL, false),
        (Part::ICON, false),
        (Part::MARKER, false),
    ];

    /// One hint, optionally with one part replaced.
    struct SlotPage(Option<Part>);

    impl App for SlotPage {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let replaced = |ui: &mut Ui<'_>, r: Rect| {
                let s = ui.surface_style();
                ui.paint_str(r, "########", s);
            };
            let mut h = KeyHint::new(HINT, Chord::key(KeyCode::Enter), LABEL);
            if let Some(p) = self.0 {
                h = h.slot(p, &replaced);
            }
            h.draw(ui, SCREEN);
        }
    }

    /// The cells one draw paints, with `slot` installed or with none.
    fn painted(slot: Option<Part>) -> Buffer {
        let mut rt = Runtime::new(SlotPage(slot), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf);
        buf
    }

    /// Invariant R (§45.3): the parts the `## Overrides` section names as
    /// slot-addressable are **exactly** the parts for which installing
    /// `.slot(p, …)` changes the painted cells. Asserted in both
    /// directions, so a dropped `slot_for` consult and a slot honoured
    /// without being named each fail.
    #[test]
    fn a_slot_changes_painted_cells_for_exactly_key_and_action() {
        let plain = painted(None);
        for (p, honoured) in SWEPT {
            let with = painted(Some(*p));
            assert_eq!(
                with != plain,
                *honoured,
                "`.slot({p:?}, …)` is documented as {}, but the painted cells {} the plain hint's",
                if *honoured { "honoured" } else { "inert" },
                if with == plain {
                    "match"
                } else {
                    "differ from"
                }
            );
        }
        assert_eq!(
            KeyHint::PARTS,
            &[Part::KEY, Part::ACTION],
            "the honoured set above is the whole of `PARTS`, so no declared part is excluded"
        );
    }

    #[test]
    fn chord_text_renders_without_allocating_and_clips_whole_fragments() {
        assert_eq!(ChordText::of(Chord::key(KeyCode::Enter)).as_str(), "Enter");
        assert_eq!(
            ChordText::of(Chord::with(KeyCode::Char('s'), KeyModifiers::CONTROL)).as_str(),
            "Ctrl+s"
        );
        assert_eq!(ChordText::of(Chord::key(KeyCode::Left)).as_str(), "←");
        // the buffer never holds a partial `char`
        let long = ChordText::of(Chord::with(
            KeyCode::Backspace,
            KeyModifiers::CONTROL
                .union(KeyModifiers::ALT)
                .union(KeyModifiers::SHIFT),
        ));
        assert!(core::str::from_utf8(&long.buf[..long.len]).is_ok());
    }

    #[test]
    fn width_is_the_chord_the_gap_and_the_label() {
        let h = KeyHint::new(Id::root("t"), Chord::key(KeyCode::Enter), "Open");
        assert_eq!(h.width(), 5 + 1 + 4);
        let bare = KeyHint::new(Id::root("t"), Chord::key(KeyCode::Enter), "");
        assert_eq!(bare.width(), 5);
    }
}
