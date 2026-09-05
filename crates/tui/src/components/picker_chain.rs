//! Staged asynchronous picker controller (`COMPONENT_ARCHITECTURE.md` §14.2 J8).

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn};
use crate::collection::empty::{CenteredText, draw_centered};
use crate::collection::{EmptyState, Status};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// One stage in a picker chain.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PickerStage<'a> {
    /// Stable stage identity.
    pub key: ItemKey,
    /// Breadcrumb label.
    pub label: &'a str,
    /// Current readiness.
    pub status: Status,
}

impl<'a> PickerStage<'a> {
    /// Declare a ready stage.
    pub const fn new(key: ItemKey, label: &'a str) -> Self {
        Self {
            key,
            label,
            status: Status::Ready,
        }
    }

    /// Set readiness.
    #[must_use]
    pub const fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }
}

/// Semantic chain output.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PickerChainAction {
    /// Return to the previous stage.
    Back(ItemKey),
    /// Retry the current failed stage.
    Retry(ItemKey),
}

/// Chain commands.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PickerChainCmd {
    /// Go back one stage.
    Back,
    /// Retry the failed current stage.
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

/// Durable active stage and breadcrumb history.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PickerChainState {
    current: Option<ItemKey>,
    history: Vec<ItemKey>,
}

impl PickerChainState {
    /// Active stage.
    pub const fn current(&self) -> Option<ItemKey> {
        self.current
    }

    /// Enter a stage while retaining the earlier breadcrumb.
    pub fn enter(&mut self, key: ItemKey) {
        if self.current != Some(key) {
            if let Some(old) = self.current {
                self.history.push(old);
            }
            self.current = Some(key);
        }
    }

    /// Return one stage.
    pub fn back(&mut self) -> Option<ItemKey> {
        let key = self.history.pop()?;
        self.current = Some(key);
        Some(key)
    }
}

/// Breadcrumb and lifecycle controller for a staged picker.
///
/// ## Construction
/// `PickerChain::new(id, stages)`; actual domain items remain with the app.
///
/// ## Ownership
/// Caller owns declarations, current item collections and state.
///
/// ## Configuration
/// `.empty`, `.patch`, `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::PICKER`, default variant.
///
/// ## States
/// Current stage readiness maps to `BUSY`, `LOADING`, or `ERROR`.
///
/// ## Actions
/// Keyed back and retry actions. The caller-owned nested picker reports its
/// own choices directly; the chain does not forward or duplicate them.
///
/// ## Focus
/// One chain focus stop. Nested picker content keeps its own focus.
///
/// ## Keyboard
/// Backspace goes back; `r` retries only an errored stage.
///
/// ## Mouse
/// Breadcrumb clicks go back to that stage; retry is a keyed control.
///
/// ## Layout
/// Breadcrumb row followed by caller-owned picker content.
///
/// ## Parts
/// `CONTAINER`, `HEADER`, `LABEL`, `BODY`, `HELP`, `ICON`.
///
/// ## Overrides
/// All parts accept patches; `HEADER`, `LABEL`, `BODY`, `HELP`, `ICON` accept slots.
///
/// ## Identity
/// Breadcrumb and retry hit targets are root-owned keyed part references.
///
/// ## Testing
/// Conformance covers focus, activation, tiny rectangles and status.
#[derive(Debug)]
pub struct PickerChain<'a> {
    id: Id,
    stages: &'a [PickerStage<'a>],
    empty: Option<EmptyState<'a>>,
    ov: PartStyle<'a>,
}

impl<'a> PickerChain<'a> {
    /// Styled parts.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::HEADER,
        Part::LABEL,
        Part::BODY,
        Part::HELP,
        Part::ICON,
    ];

    /// Build a chain over borrowed stage declarations.
    pub const fn new(id: Id, stages: &'a [PickerStage<'a>]) -> Self {
        Self {
            id,
            stages,
            empty: None,
            ov: PartStyle::new(),
        }
    }

    /// Set the empty/loading/error content below the breadcrumb.
    #[must_use]
    pub const fn empty(mut self, empty: EmptyState<'a>) -> Self {
        self.empty = Some(empty);
        self
    }
    /// Patch every part.
    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.ov = self.ov.global(patch);
        self
    }
    /// Patch selected parts.
    #[must_use]
    pub const fn patch_part(mut self, parts: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(parts);
        self
    }
    /// Replace a supported part painter.
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

    /// Update back/retry and breadcrumb activation.
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

    /// Draw breadcrumb and optional readiness surface.
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

    /// Natural breadcrumb size.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;
    use crate::ui::cx::LastFrame;
    use crate::ui::{FrameState, Ui, UiCore};
    use ratatui_core::buffer::{Buffer, Cell};

    const AREA: Rect = Rect::new(0, 0, 40, 8);

    fn slot_icon(ui: &mut Ui<'_>, area: Rect) {
        ui.paint_str(area, "#", ui.surface_style());
    }

    fn render(chain: &PickerChain<'_>, state: &PickerChainState) -> (Buffer, FrameState) {
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, AREA);
        let mut page = Buffer::empty(AREA);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            chain.draw(&mut ui, AREA, state);
        }
        (page, frame)
    }

    fn text(buffer: &Buffer) -> String {
        AREA.positions()
            .filter_map(|position| buffer.cell(position))
            .map(Cell::symbol)
            .collect()
    }

    #[test]
    fn back_one_step_preserves_the_breadcrumb() {
        let mut st = PickerChainState::default();
        let a = ItemKey::text("account");
        let b = ItemKey::text("vault");
        st.enter(a);
        st.enter(b);
        assert_eq!(st.back(), Some(a));
        assert_eq!(st.current(), Some(a));
    }

    #[test]
    fn readiness_icon_is_declared_and_chain_owned() {
        assert!(PickerChain::PARTS.contains(&Part::ICON));
        let failed_stage = PickerStage::new(ItemKey::num(1), "Account").status(Status::Error);
        let mut chain_state = PickerChainState::default();
        chain_state.enter(failed_stage.key);
        let declared_stages = [failed_stage];
        let chain = PickerChain::new(Id::root("chain.icon"), &declared_stages);
        let (buffer, _) = render(&chain, &chain_state);
        assert!(text(&buffer).contains("Unable to load"));
        assert!(text(&buffer).contains("Press r to retry"));
        assert!(text(&buffer).contains(Theme::junie().design.glyphs.get(GlyphRole::Error)));
    }

    #[test]
    fn icon_patch_and_slot_reach_the_computed_prefix_cell() {
        let failed_stage = PickerStage::new(ItemKey::num(1), "Account").status(Status::Error);
        let mut chain_state = PickerChainState::default();
        chain_state.enter(failed_stage.key);
        let declared_stages = [failed_stage];
        let icon_patches = [(
            Part::ICON,
            StylePatch::new().set_glyph(GlyphRole::WarningMark),
        )];
        let patched_chain =
            PickerChain::new(Id::root("chain.patch"), &declared_stages).patch_part(&icon_patches);
        let (buffer, _) = render(&patched_chain, &chain_state);
        assert!(text(&buffer).contains(Theme::junie().design.glyphs.get(GlyphRole::WarningMark)));

        let slotted =
            PickerChain::new(Id::root("chain.slot"), &declared_stages).slot(Part::ICON, &slot_icon);
        let (buffer, _) = render(&slotted, &chain_state);
        assert!(text(&buffer).contains('#'));
    }

    #[test]
    fn earlier_crumb_and_error_retry_are_root_owned_keyed_parts() {
        let a = PickerStage::new(ItemKey::num(1), "Account");
        let b = PickerStage::new(ItemKey::num(2), "Vault").status(Status::Error);
        let stages = [a, b];
        let mut state = PickerChainState::default();
        state.enter(a.key);
        state.enter(b.key);
        let id = Id::root("chain.parts");
        let chain = PickerChain::new(id, &stages);
        let (_, frame) = render(&chain, &state);
        assert!(
            frame
                .registry
                .area_of_part(id, PartRef::item(Part::LABEL, a.key))
                .is_some()
        );
        assert!(
            frame
                .registry
                .area_of_part(id, PartRef::item(Part::HELP, b.key))
                .is_some()
        );
    }

    #[test]
    fn action_surface_is_only_back_and_retry() {
        fn key(action: PickerChainAction) -> ItemKey {
            match action {
                PickerChainAction::Back(key) | PickerChainAction::Retry(key) => key,
            }
        }
        let key_value = ItemKey::num(7);
        assert_eq!(key(PickerChainAction::Back(key_value)), key_value);
        assert_eq!(key(PickerChainAction::Retry(key_value)), key_value);
    }
}
