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

const CHORD_CAP: usize = 32;

#[derive(Clone, Copy, Debug)]
pub struct ChordText {
    buf: [u8; CHORD_CAP],
    len: usize,
}

impl ChordText {
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
    pub const PARTS: &'static [Part] = &[Part::KEY, Part::ACTION];

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

    pub const fn new(id: Id, chord: Chord, label: &'a str) -> Self {
        KeyHint {
            id,
            key: HintKey::Chord(chord),
            label,
            variant: Variant::DEFAULT,
            ov: PartStyle::new(),
        }
    }

    pub const fn descriptive(id: Id, key: &'static str, label: &'a str) -> Self {
        KeyHint {
            id,
            key: HintKey::Label(key),
            label,
            variant: Variant::DEFAULT,
            ov: PartStyle::new(),
        }
    }

    pub const fn from_hint(id: Id, h: &Hint) -> KeyHint<'static> {
        KeyHint {
            id,
            key: h.key,
            label: h.label,
            variant: Variant::DEFAULT,
            ov: PartStyle::new(),
        }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    fn key_text(&self) -> HintText {
        HintText::of(self.key)
    }

    pub fn width(&self) -> u16 {
        let key = width(self.key_text().as_str());
        if self.label.is_empty() {
            return key;
        }
        key.saturating_add(1).saturating_add(width(self.label))
    }

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

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size::exact(self.width(), 1).fit(c)
    }
}
