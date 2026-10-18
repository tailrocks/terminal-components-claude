#![doc = r" `KeyHint` — the `key Action` pair (`COMPONENT_ARCHITECTURE.md` §18.2,"]
#![doc = r" Appendix A 4G)."]

use core::fmt::{self, Write as _};

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn, shift};
use crate::event::Chord;
use crate::id::{Id, Part};
use crate::keymap::{ChordCase, Hint, HintKey};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{Family, StylePatch, Variant};
use crate::ui::Ui;

#[doc = r" Bytes reserved for a rendered chord label. The longest chord the"]
#[doc = r" [`Chord`] `Display` can produce is `Ctrl+Alt+Shift+Backspace` (24 bytes);"]
#[doc = r" the arrow keys are three bytes each and far shorter."]
const CHORD_CAP: usize = 32;

#[doc = r" A [`Chord`]'s label rendered into a fixed stack buffer."]
#[doc = r""]
#[doc = r" `Chord` is `Display`, but a component may not allocate once per frame per"]
#[doc = r" hint (§20.9-6, R5), and there is no `Ui::paint_fmt`. Writing into a fixed"]
#[doc = r" buffer keeps the hint bar allocation-free; a chord whose label does not"]
#[doc = r" fit keeps the prefix that does, which cannot split a `char` because only"]
#[doc = r" whole `write_str` fragments are appended."]
#[derive(Clone, Copy, Debug)]
pub struct ChordText {
    buf: [u8; CHORD_CAP],
    len: usize,
}

impl ChordText {
    #[doc = r" Render `c`."]
    pub fn of(c: Chord) -> Self {
        Self::with_case(c, ChordCase::Preserve)
    }

    pub fn with_case(mut c: Chord, case: ChordCase) -> Self {
        if let (ChordCase::UppercaseAscii, crate::event::KeyCode::Char(character)) =
            (case, &mut c.code)
        {
            *character = character.to_ascii_uppercase();
        }
        let mut t = ChordText {
            buf: [0; CHORD_CAP],
            len: 0,
        };

        let _ = write!(t, "{c}");
        t
    }

    #[doc = r" The rendered label."]
    pub fn as_str(&self) -> &str {
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
            return Ok(());
        }
        if let Some(dst) = self.buf.get_mut(self.len..end) {
            dst.copy_from_slice(s.as_bytes());
            self.len = end;
        }
        Ok(())
    }
}

#[doc = r" One `key Action` pair: the chord in the key tone, the label beside it."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `KeyHint::new(id, chord, label)`; [`KeyHint::from_hint`] builds one from a"]
#[doc = r" [`Hint`] as the hint bar derives it from a component's bindings (§13.1)."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" Stateless. The caller owns nothing; the chord and label are borrowed or"]
#[doc = r" `Copy`. The runtime owns nothing for this component: it registers no"]
#[doc = r" region and no ring entry."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.variant(Variant)` (default `Recipe.default_variant`), `.patch`,"]
#[doc = r" `.patch_part`, `.slot`."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::KEYHINT`; `DEFAULT` only. A theme may define more."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" None. A key hint is chrome: it is never focused, hovered, pressed or"]
#[doc = r" disabled, and it derives nothing."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" None; `KeyHint` has no `update` phase."]
#[doc = r""]
#[doc = r" ## Focus"]
#[doc = r" Never a focus stop; `Focusability` does not apply and `autofocus` does not"]
#[doc = r" exist."]
#[doc = r""]
#[doc = r" ## Keyboard"]
#[doc = r" None. The chord it *renders* is bound by whichever component declared it;"]
#[doc = r" the hint never handles a key itself."]
#[doc = r""]
#[doc = r" ## Mouse"]
#[doc = r" None; no `PartRef` is registered, so no pointer intent is delivered."]
#[doc = r""]
#[doc = r" ## Layout"]
#[doc = r" `measure` returns `(chord + 1 + label, 1)`. `draw` uses the first row of"]
#[doc = r" `area`, clipped to that width, and returns the rect it painted; a"]
#[doc = r" degenerate rect paints nothing and returns it unchanged (R5)."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `KEY` (the chord), `ACTION` (the label)."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.patch` and `.patch_part` reach `Part::KEY` and `Part::ACTION`."]
#[doc = r" `.slot(p, …)` changes painted cells for exactly `Part::KEY` and"]
#[doc = r" `Part::ACTION` — every part `KeyHint` declares, so nothing is excluded,"]
#[doc = r" and there is no `CONTAINER` to exclude because a hint paints no fill of"]
#[doc = r" its own and sits on whatever surface its owner laid down. `ACTION` is"]
#[doc = r" painted, and therefore slot-addressable, only when the label is"]
#[doc = r" non-empty and the clipped row still has columns after the chord and its"]
#[doc = r" separating space; a hint built with an empty label paints no `ACTION`"]
#[doc = r" cell for a slot to replace."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" One `Id` per instance, used only to attribute style resolution and"]
#[doc = r" overrides; no items."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `KeyHintCase` with no capabilities;"]
#[doc = r" `render::components::key_hint::{default, disabled, empty}`;"]
#[doc = r" `keyhint::a_slot_changes_painted_cells_for_exactly_key_and_action`, which"]
#[doc = r" asserts Invariant R (§45.3) in both directions — a named part that stops"]
#[doc = r" changing cells and an unnamed part that starts changing them each fail"]
#[doc = r" it."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" Never allocates: the chord is rendered into a fixed stack buffer. Never"]
#[doc = r" writes outside `area`."]
pub struct KeyHint<'a> {
    id: Id,
    key: HintKey,
    label: &'a str,
    variant: Variant,
    ov: PartStyle<'a>,
}

pub enum HintText {
    Chord(ChordText),
    Label(&'static str),
}
impl HintText {
    pub fn of(key: HintKey) -> Self {
        match key {
            HintKey::Chord(chord) => Self::Chord(ChordText::of(chord)),
            HintKey::ChordWithCase { chord, case } => {
                Self::Chord(ChordText::with_case(chord, case))
            }
            HintKey::Label(label) => Self::Label(label),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Chord(chord) => chord.as_str(),
            Self::Label(label) => label,
        }
    }
}

impl fmt::Debug for KeyHint<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyHint")
            .field("id", &self.id)
            .field("key", &self.key)
            .field("label", &self.label)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<'a> KeyHint<'a> {
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[Part::KEY, Part::ACTION];

    #[doc = r" Override chord display casing without changing its physical identity."]
    #[doc = r" Descriptive labels retain their literal text."]
    #[must_use]
    pub const fn chord_case(mut self, case: ChordCase) -> Self {
        self.key = match self.key {
            HintKey::Chord(chord) | HintKey::ChordWithCase { chord, .. } => {
                HintKey::ChordWithCase { chord, case }
            }
            HintKey::Label(label) => HintKey::Label(label),
        };
        self
    }

    #[doc = r" A hint showing `chord` and `label`."]
    pub const fn new(id: Id, chord: Chord, label: &'a str) -> Self {
        KeyHint {
            id,
            key: HintKey::Chord(chord),
            label,
            variant: Variant::DEFAULT,
            ov: PartStyle::new(),
        }
    }

    #[doc = r" A descriptive keycap such as `Type`, with no associated routing chord."]
    pub const fn descriptive(id: Id, key: &'static str, label: &'a str) -> Self {
        KeyHint {
            id,
            key: HintKey::Label(key),
            label,
            variant: Variant::DEFAULT,
            ov: PartStyle::new(),
        }
    }

    #[doc = r" The hint a [`Hint`] describes — the shape `HintBar` derives from a"]
    #[doc = r" component's visible bindings (§13.1)."]
    pub const fn from_hint(id: Id, h: &Hint) -> KeyHint<'static> {
        KeyHint {
            id,
            key: h.key,
            label: h.label,
            variant: Variant::DEFAULT,
            ov: PartStyle::new(),
        }
    }

    #[doc = r" The id."]
    pub const fn id(&self) -> Id {
        self.id
    }

    #[doc = r" Set the variant."]
    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[doc = r" An instance patch over every part (precedence 6)."]
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[doc = r" Per-part patches."]
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[doc = r" Replace one part's painting; layout stays."]
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    fn key_text(&self) -> HintText {
        HintText::of(self.key)
    }

    #[doc = r" The columns this hint occupies: the chord, one space, the label."]
    pub fn width(&self) -> u16 {
        let key = width(self.key_text().as_str());
        if self.label.is_empty() {
            return key;
        }
        key.saturating_add(1).saturating_add(width(self.label))
    }

    #[doc = r" The draw phase; returns the rect painted."]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = Rect {
            width: self.width().min(area.width),
            height: u16::from(area.height != 0),
            ..area
        };
        if area.is_empty() {
            return area;
        }

        let live = PartStyle::flags(StateFlags::empty(), StateFlags::empty());
        let ov = self.ov;
        let key_text = self.key_text();
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

    #[doc = r" The natural size: one row, the chord plus the label."]
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

    #[doc = r" Every part the slot sweep installs on, and whether `KeyHint` is"]
    #[doc = r" expected to honour it: the two parts it declares, then four it does"]
    #[doc = r" not, so the sweep can fail in either direction."]
    const SWEPT: &[(Part, bool)] = &[
        (Part::KEY, true),
        (Part::ACTION, true),
        (Part::CONTAINER, false),
        (Part::LABEL, false),
        (Part::ICON, false),
        (Part::MARKER, false),
    ];

    #[doc = r" One hint, optionally with one part replaced."]
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

    #[doc = r" The cells one draw paints, with `slot` installed or with none."]
    fn painted(slot: Option<Part>) -> Buffer {
        let mut rt = Runtime::new(SlotPage(slot), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        buf
    }

    #[doc = r" Invariant R (§45.3): the parts the `## Overrides` section names as"]
    #[doc = r" slot-addressable are **exactly** the parts for which installing"]
    #[doc = r" `.slot(p, …)` changes the painted cells. Asserted in both"]
    #[doc = r" directions, so a dropped `slot_for` consult and a slot honoured"]
    #[doc = r" without being named each fail."]
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
