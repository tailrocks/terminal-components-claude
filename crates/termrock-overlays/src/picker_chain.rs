use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use termrock_runtime::empty::{CenteredText, draw_centered};
use termrock_runtime::empty::{EmptyState, Status};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PickerStage<'a> {
    pub key: ItemKey,
    pub label: &'a str,
    pub status: Status,
}

impl<'a> PickerStage<'a> {
    pub const fn new(key: ItemKey, label: &'a str) -> Self {
        Self {
            key,
            label,
            status: Status::Ready,
        }
    }

    #[must_use]
    pub const fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PickerChainAction {
    Back(ItemKey),
    Retry(ItemKey),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PickerChainCmd {
    Back,
    Retry,
}

const BINDINGS: &[Binding<PickerChainCmd>] = &[
    Binding {
        action: crate::ActionKey::custom("picker-chain.back"),
        chord: Some(Chord::key(KeyCode::Backspace)),
        cmd: PickerChainCmd::Back,
        label: "Back",
        priority: 70,
        visible: true,
    },
    Binding {
        action: crate::ActionKey::custom("picker-chain.retry"),
        chord: Some(Chord::key(KeyCode::Char('r'))),
        cmd: PickerChainCmd::Retry,
        label: "Retry",
        priority: 50,
        visible: true,
    },
];

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PickerChainState {
    current: Option<ItemKey>,
    history: Vec<ItemKey>,
}

impl PickerChainState {
    pub const fn current(&self) -> Option<ItemKey> {
        self.current
    }

    pub fn enter(&mut self, key: ItemKey) {
        if self.current != Some(key) {
            if let Some(old) = self.current {
                self.history.push(old);
            }
            self.current = Some(key);
        }
    }

    pub fn back(&mut self) -> Option<ItemKey> {
        let key = self.history.pop()?;
        self.current = Some(key);
        Some(key)
    }
}

#[derive(Debug)]
pub struct PickerChain<'a> {
    id: Id,
    stages: &'a [PickerStage<'a>],
    empty: Option<EmptyState<'a>>,
    ov: PartStyle<'a>,
}

impl<'a> PickerChain<'a> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::HEADER,
        Part::LABEL,
        Part::BODY,
        Part::HELP,
        Part::ICON,
    ];

    pub const fn new(id: Id, stages: &'a [PickerStage<'a>]) -> Self {
        Self {
            id,
            stages,
            empty: None,
            ov: PartStyle::new(),
        }
    }

    #[must_use]
    pub const fn empty(mut self, empty: EmptyState<'a>) -> Self {
        self.empty = Some(empty);
        self
    }
    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.ov = self.ov.global(patch);
        self
    }
    #[must_use]
    pub const fn patch_part(mut self, parts: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(parts);
        self
    }
    #[must_use]
    pub const fn slot(mut self, part: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(part, f);
        self
    }

    fn active<'s>(&self, st: &'s PickerChainState) -> Option<&'a PickerStage<'a>> {
        st.current
            .and_then(|key| self.stages.iter().find(|s| s.key == key))
            .or_else(|| self.stages.first())
    }

    fn can_rewind_to(st: &PickerChainState, key: ItemKey) -> bool {
        st.history.contains(&key)
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut PickerChainState,
    ) -> Response<PickerChainAction> {
        if st.current.is_none() {
            st.current = self.stages.first().map(|s| s.key);
        }
        let mut action = None;
        let mut consumed = false;
        for intent in cx.intents(self.id) {
            match intent {
                Intent::Binding(binding) => {
                    if let Some(cmd) = Binding::command(BINDINGS, binding) {
                        consumed = true;
                        match cmd {
                            PickerChainCmd::Back => {
                                action = st.back().map(PickerChainAction::Back);
                            }
                            PickerChainCmd::Retry => {
                                if let Some(stage) =
                                    self.active(st).filter(|s| s.status == Status::Error)
                                {
                                    action = Some(PickerChainAction::Retry(stage.key));
                                }
                            }
                        }
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    part:
                        PartRef {
                            part: Part::LABEL,
                            item: Some(key),
                        },
                    ..
                } if Self::can_rewind_to(st, key) => {
                    while st.current != Some(key) && st.back().is_some() {}
                    if st.current == Some(key) {
                        action.get_or_insert(PickerChainAction::Back(key));
                    }
                    consumed = true;
                }
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    part:
                        PartRef {
                            part: Part::HELP,
                            item: Some(key),
                        },
                    ..
                } if self
                    .active(st)
                    .is_some_and(|stage| stage.key == key && stage.status == Status::Error) =>
                {
                    action.get_or_insert(PickerChainAction::Retry(key));
                    consumed = true;
                }
                Intent::Pointer {
                    part:
                        PartRef {
                            part: Part::LABEL | Part::HELP,
                            ..
                        },
                    ..
                } => {
                    consumed = true;
                }
                _ => {}
            }
        }
        match action {
            Some(a) => Response::action(a),
            None if consumed => Response::consumed(),
            None => Response::ignored(),
        }
        .for_id(self.id)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one pass owns breadcrumb, body and readiness geometry"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &PickerChainState) -> Rect {
        if area.is_empty() {
            return area;
        }
        ui.register_control(self.id, area, Focusability::Focusable);
        let derived = self
            .active(st)
            .map_or(StateFlags::empty(), |s| s.status.flags());
        let live = PartStyle::flags(ui.state(self.id), derived);
        let base_live = live.difference(
            StateFlags::FOCUSED
                | StateFlags::FOCUS_VISIBLE
                | StateFlags::HOVERED
                | StateFlags::PRESSED,
        );
        ui.publish_bindings(self.id, live, BINDINGS);
        let base = self.ov.style(
            ui,
            self.id,
            Family::PICKER,
            Variant::DEFAULT,
            Part::CONTAINER,
            base_live,
        );
        ui.fill(area, base.style);
        let header = Rect { height: 1, ..area };
        if let Some(slot) = self.ov.slot_for(Part::HEADER) {
            slot(ui, header);
        } else {
            let hs = self.ov.style(
                ui,
                self.id,
                Family::PICKER,
                Variant::DEFAULT,
                Part::HEADER,
                base_live,
            );
            ui.fill(header, hs.style);
            let mut x = header.x;
            for stage in self.stages {
                let w = crate::text::width(stage.label)
                    .saturating_add(3)
                    .min(header.right().saturating_sub(x));
                let cell = Rect {
                    x,
                    width: w,
                    ..header
                };
                let actionable = Self::can_rewind_to(st, stage.key);
                if actionable {
                    ui.register_part(self.id, PartRef::item(Part::LABEL, stage.key), cell);
                }
                let runtime_pressed = FrameRead::pressed_part(ui, self.id)
                    == Some(PartRef::item(Part::LABEL, stage.key));
                let runtime_hovered = FrameRead::hovered_part(ui, self.id)
                    == Some(PartRef::item(Part::LABEL, stage.key));
                let current = st.current == Some(stage.key);
                let ls = self.ov.style(
                    ui,
                    self.id,
                    Family::PICKER,
                    Variant::DEFAULT,
                    Part::LABEL,
                    base_live
                        | if current {
                            StateFlags::ACTIVE
                                | live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE)
                        } else {
                            StateFlags::empty()
                        }
                        | if runtime_hovered {
                            StateFlags::HOVERED
                        } else {
                            StateFlags::empty()
                        }
                        | if runtime_pressed {
                            StateFlags::PRESSED
                        } else {
                            StateFlags::empty()
                        },
                );
                ui.paint_str(cell, stage.label, ls.style);
                x = x.saturating_add(w);
            }
        }
        let body = Rect {
            y: area.y.saturating_add(1),
            height: area.height.saturating_sub(1),
            ..area
        };
        let readiness = self.active(st).and_then(|stage| match stage.status {
            Status::Busy | Status::Loading => Some(EmptyState::Loading { label: "Loading" }),
            Status::Error => Some(EmptyState::Error {
                message: "Unable to load",
                detail: Some("Press r to retry"),
            }),
            Status::Ready => self.empty,

            _ => Some(EmptyState::Loading { label: "Loading" }),
        });
        if let Some(slot) = self.ov.slot_for(Part::BODY) {
            slot(ui, body);
        } else if let Some(empty) = readiness {
            self.draw_readiness(ui, body, empty, base_live, st);
        }
        ui.register_decor(self.id, PartRef::of(Part::BODY), body);
        area
    }

    fn draw_readiness(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        empty: EmptyState<'_>,
        live: StateFlags,
        st: &PickerChainState,
    ) {
        let title = self.ov.style(
            ui,
            self.id,
            Family::PICKER,
            Variant::DEFAULT,
            Part::LABEL,
            live,
        );
        let active_key = self.active(st).map(|stage| stage.key);
        let active_help = active_key.map(|key| PartRef::item(Part::HELP, key));
        let mut help_live = live;
        if active_help.is_some_and(|part| FrameRead::hovered_part(ui, self.id) == Some(part)) {
            help_live |= StateFlags::HOVERED;
        }
        if active_help.is_some_and(|part| FrameRead::pressed_part(ui, self.id) == Some(part)) {
            help_live |= StateFlags::PRESSED;
        }
        let help = self.ov.style(
            ui,
            self.id,
            Family::PICKER,
            Variant::DEFAULT,
            Part::HELP,
            help_live,
        );
        let icon = self.ov.style(
            ui,
            self.id,
            Family::PICKER,
            Variant::DEFAULT,
            Part::ICON,
            live,
        );
        let glyph = match empty.status() {
            Status::Busy | Status::Loading => ui.design().motion.spinner_frames.first().copied(),
            Status::Error => match icon.glyph {
                Slot::Set(glyph) => Some(ui.design().glyphs.get(glyph)),
                Slot::Inherit => Some(ui.design().glyphs.get(GlyphRole::Error)),
                Slot::Clear => None,
            },
            Status::Ready => None,

            _ => None,
        };
        let icon_width = glyph.map_or(0, crate::text::width);
        let detail = empty.detail();
        draw_centered(
            ui,
            area,
            CenteredText {
                title: empty.title(),
                detail,
                icon_width,
            },
            |ui, icon_area| {
                if let Some(slot) = self.ov.slot_for(Part::ICON) {
                    slot(ui, icon_area);
                } else if let Some(glyph) = glyph {
                    ui.paint_str(icon_area, glyph, icon.style);
                }
            },
            |ui, title_area| {
                if let Some(slot) = self.ov.slot_for(Part::LABEL) {
                    slot(ui, title_area);
                } else {
                    ui.paint_str(title_area, empty.title(), title.style);
                }
            },
            |ui, detail_area| {
                if let Some(slot) = self.ov.slot_for(Part::HELP) {
                    slot(ui, detail_area);
                } else if let Some(detail) = detail {
                    ui.paint_str(detail_area, detail, help.style);
                }
                if empty.status() == Status::Error
                    && let Some(key) = active_key
                {
                    ui.register_part(self.id, PartRef::item(Part::HELP, key), detail_area);
                }
            },
        );
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        let w = self.stages.iter().fold(0u16, |n, s| {
            n.saturating_add(crate::text::width(s.label))
                .saturating_add(3)
        });
        Size {
            min: (1, 1),
            preferred: (w, 8),
        }
        .fit(c)
    }
}

impl Bindings for PickerChain<'_> {
    type Cmd = PickerChainCmd;
    fn bindings(&self, _state: BindingState) -> &'static [Binding<Self::Cmd>] {
        BINDINGS
    }
}
