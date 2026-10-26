use core::fmt::{self, Write as _};

use crate::theme::PaintStyle;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;

use crate::id::{Id, ItemKey, Part};
use crate::layout::{Track, distribute_into};
use crate::response::StateFlags;
use crate::text::{Span, truncate, truncate_middle, width};
use crate::theme::{
    Align, CursorStyle, Family, FgStep, GlyphRole, Role, Slot, StylePatch, Variant,
};
use crate::ui::{FrameRead, Ui};

pub trait RowFn<T> {
    fn row(&self, item: &T, row: &mut RowUi<'_>);

    fn intrinsic_label_width(&self, _item: &T) -> Option<u16> {
        None
    }
}

impl<T, F: Fn(&T, &mut RowUi<'_>)> RowFn<T> for F {
    fn row(&self, item: &T, row: &mut RowUi<'_>) {
        self(item, row);
    }
}

impl<T: core::fmt::Display> RowFn<T> for termrock_collections::DefaultRow {
    fn row(&self, item: &T, row: &mut RowUi<'_>) {
        row.label_fmt(format_args!("{item}"));
    }

    fn intrinsic_label_width(&self, item: &T) -> Option<u16> {
        Some(width(&item.to_string()))
    }
}

pub const MAX_COLUMNS: usize = 16;

const fn selected_row_patch() -> StylePatch {
    StylePatch::new()
        .set_bg(Role::HighlightBg)
        .set_fg(Role::HighlightFg)
        .add(Modifier::BOLD)
        .remove(Modifier::UNDERLINED)
}

fn selected_part_patch(part: Part) -> StylePatch {
    let patch = StylePatch::new()
        .set_bg(Role::HighlightBg)
        .set_fg(Role::HighlightFg)
        .remove(Modifier::UNDERLINED);
    if part == Part::LABEL || part == Part::ROW {
        patch.add(Modifier::BOLD)
    } else {
        patch.remove(Modifier::BOLD)
    }
}

fn merge_patch(base: Option<StylePatch>, over: Option<StylePatch>) -> Option<StylePatch> {
    match (base, over) {
        (Some(base), Some(over)) => Some(base.merge(over)),
        (Some(patch), None) | (None, Some(patch)) => Some(patch),
        (None, None) => None,
    }
}

fn fill_gutter_patch(flags: StateFlags) -> StylePatch {
    if flags.contains(StateFlags::FOCUSED) {
        StylePatch::new()
            .set_fg(Role::HighlightBg)
            .set_bg(Role::HighlightBg)
            .add(Modifier::BOLD)
    } else {
        StylePatch::new()
            .set_fg(Role::CurrentSurface)
            .set_bg(Role::CurrentSurface)
            .remove(Modifier::BOLD)
    }
}

fn resolve_row_part(
    ui: &mut Ui<'_>,
    family: Family,
    variant: Variant,
    flags: StateFlags,
    part: Part,
    patch: Option<StylePatch>,
    cursor_style: CursorStyle,
) -> crate::theme::Resolved {
    resolve_row_part_with_defaults(
        ui,
        family,
        variant,
        flags,
        part,
        patch,
        cursor_style,
        crate::theme::StyleDefaults::default(),
    )
}

fn picker_row_defaults(family: Family, part: Part, flags: StateFlags) -> StylePatch {
    if family != Family::PICKER {
        return StylePatch::new();
    }
    match part {
        Part::ICON => {
            let ink = if flags.contains(StateFlags::FOCUSED) {
                FgStep::Primary
            } else {
                FgStep::Muted
            };
            StylePatch::new()
                .set_fg(Role::Fg(ink))
                .remove(Modifier::BOLD)
        }
        Part::META => StylePatch::new()
            .set_fg(Role::Fg(FgStep::Muted))
            .remove(Modifier::BOLD),
        _ => StylePatch::new(),
    }
}

fn readable_picker_ink(
    preferred: ratatui_core::style::Color,
    bg: Option<ratatui_core::style::Color>,
) -> Option<ratatui_core::style::Color> {
    use crate::theme::contrast_ratio;
    use ratatui_core::style::Color;

    let bg = bg?;
    let ratio = |fg| contrast_ratio(fg, bg).unwrap_or(0.0);
    if ratio(preferred) >= 4.5 {
        return None;
    }
    let (black, white) = (ratio(Color::Black), ratio(Color::White));
    let (ink, contrast) = if black >= white {
        (Color::Black, black)
    } else {
        (Color::White, white)
    };
    (contrast >= 4.5).then_some(ink)
}

#[expect(
    clippy::too_many_arguments,
    reason = "keeps the existing resolver inputs and adds only component defaults"
)]
fn resolve_row_part_with_defaults(
    ui: &mut Ui<'_>,
    family: Family,
    variant: Variant,
    flags: StateFlags,
    part: Part,
    patch: Option<StylePatch>,
    cursor_style: CursorStyle,
    component_defaults: crate::theme::StyleDefaults<'_>,
) -> crate::theme::Resolved {
    if cursor_style == CursorStyle::Fill && part == Part::GUTTER {
        let explicit = ui.style_defaults_with_explicit(
            family,
            variant,
            part,
            flags,
            crate::theme::StyleDefaults::default(),
            patch.as_ref(),
        );
        let local = fill_gutter_patch(flags).merge(explicit.explicit);
        let mut resolved = ui.style_defaults(
            family,
            variant,
            part,
            flags,
            crate::theme::StyleDefaults::default(),
            Some(&local),
        );
        if explicit.explicit.glyph == Slot::Inherit {
            resolved.glyph = Slot::Clear;
        }
        return resolved;
    }
    let mut fallback = picker_row_defaults(family, part, flags).merge(component_defaults.base);
    if cursor_style == CursorStyle::Fill && flags.contains(StateFlags::FOCUSED) {
        fallback = fallback.merge(selected_part_patch(part));
    }
    let mut defaults = crate::theme::StyleDefaults::new(fallback).mono(component_defaults.mono);
    if fallback != StylePatch::new() || !component_defaults.mono.is_empty() {
        let mut resolved =
            ui.style_defaults_with_explicit(family, variant, part, flags, defaults, patch.as_ref());
        if family == Family::PICKER
            && matches!(part, Part::ICON | Part::META)
            && resolved.explicit.bg != Slot::Inherit
            && resolved.explicit.fg == Slot::Inherit
            && let Some(preferred) = resolved.resolved.style.fg
            && let Some(ink) = readable_picker_ink(preferred, resolved.resolved.style.bg)
        {
            fallback = fallback.set_fg(Role::Custom(ink));
            defaults = crate::theme::StyleDefaults::new(fallback).mono(component_defaults.mono);
            resolved = ui.style_defaults_with_explicit(
                family,
                variant,
                part,
                flags,
                defaults,
                patch.as_ref(),
            );
        }
        let resolved = resolved.resolved;
        if part == Part::GUTTER {
            let explicit = ui.style_defaults_with_explicit(
                family,
                variant,
                part,
                flags,
                crate::theme::StyleDefaults::default(),
                patch.as_ref(),
            );
            if explicit.explicit.glyph == Slot::Inherit {
                return crate::theme::Resolved {
                    glyph: Slot::Clear,
                    ..resolved
                };
            }
        }
        return resolved;
    }
    match patch {
        Some(patch) => ui.style_patched(family, variant, part, flags, &patch),
        None => ui.style(family, variant, part, flags),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MetaPaint {
    needed_width: u16,
    admitted_width: u16,
    logical_x: Option<u16>,
}

impl MetaPaint {
    pub const fn needed_width(self) -> u16 {
        self.needed_width
    }

    pub const fn admitted_width(self) -> u16 {
        self.admitted_width
    }

    pub const fn logical_x(self) -> Option<u16> {
        self.logical_x
    }
}

pub struct RowUi<'u> {
    ui: Ui<'u>,
    owner: Id,
    family: Family,
    variant: Variant,
    flags: StateFlags,
    key: ItemKey,
    row: Rect,
    container_width: u16,
    cursor_style: CursorStyle,
    author_style: crate::author::PartStyle<'u>,
    author_parts: Option<&'u [Part]>,
    container_patch: Option<StylePatch>,
    label_patch: Option<StylePatch>,
    left: u16,
    right: u16,
    keep_tail: bool,
    padded_tail: bool,
    last_meta_paint: Option<MetaPaint>,
}

impl fmt::Debug for RowUi<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RowUi")
            .field("owner", &self.owner)
            .field("key", &self.key)
            .field("row", &self.row)
            .field("container_width", &self.container_width)
            .field("left", &self.left)
            .field("right", &self.right)
            .finish_non_exhaustive()
    }
}

impl<'u> RowUi<'u> {
    pub fn new<'a: 'u>(
        ui: &'u mut Ui<'a>,
        owner: Id,
        family: Family,
        variant: Variant,
        flags: StateFlags,
        key: ItemKey,
        row: Rect,
    ) -> RowUi<'u> {
        Self::new_with_patches(ui, owner, family, variant, flags, key, row, None, None)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "shared widgets pass the complete row allocation alongside the local content area"
    )]
    pub fn new_with_container_width<'a: 'u>(
        ui: &'u mut Ui<'a>,
        owner: Id,
        family: Family,
        variant: Variant,
        flags: StateFlags,
        key: ItemKey,
        row: Rect,
        container_width: u16,
    ) -> RowUi<'u> {
        Self::new_with_patches_and_container_width(
            ui,
            owner,
            family,
            variant,
            flags,
            key,
            row,
            container_width,
            None,
            None,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the crate-private constructor extends RowUi's public phase context with two scoped patches"
    )]
    pub fn new_with_patches<'a: 'u>(
        ui: &'u mut Ui<'a>,
        owner: Id,
        family: Family,
        variant: Variant,
        flags: StateFlags,
        key: ItemKey,
        row: Rect,
        container_patch: Option<StylePatch>,
        label_patch: Option<StylePatch>,
    ) -> RowUi<'u> {
        Self::construct(
            ui,
            owner,
            family,
            variant,
            flags,
            key,
            row,
            row.width,
            crate::author::PartStyle::new(),
            container_patch,
            label_patch,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "shared widgets pass the complete row allocation alongside the local content area"
    )]
    pub fn new_with_patches_and_container_width<'a: 'u>(
        ui: &'u mut Ui<'a>,
        owner: Id,
        family: Family,
        variant: Variant,
        flags: StateFlags,
        key: ItemKey,
        row: Rect,
        container_width: u16,
        container_patch: Option<StylePatch>,
        label_patch: Option<StylePatch>,
    ) -> RowUi<'u> {
        Self::construct(
            ui,
            owner,
            family,
            variant,
            flags,
            key,
            row,
            container_width,
            crate::author::PartStyle::new(),
            container_patch,
            label_patch,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "component-author rows carry the existing phase context and one part-style contract"
    )]
    pub fn new_with_author_style<'a: 'u>(
        ui: &'u mut Ui<'a>,
        owner: Id,
        family: Family,
        variant: Variant,
        flags: StateFlags,
        key: ItemKey,
        row: Rect,
        author_style: crate::author::PartStyle<'u>,
    ) -> RowUi<'u> {
        Self::construct(
            ui,
            owner,
            family,
            variant,
            flags,
            key,
            row,
            row.width,
            author_style,
            None,
            None,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the component-author row contract carries identity, state, geometry, parts, and one container patch"
    )]
    pub fn new_with_author_style_and_container_patch<'a: 'u>(
        ui: &'u mut Ui<'a>,
        owner: Id,
        family: Family,
        variant: Variant,
        flags: StateFlags,
        key: ItemKey,
        row: Rect,
        author_style: crate::author::PartStyle<'u>,
        container_patch: StylePatch,
    ) -> RowUi<'u> {
        Self::construct(
            ui,
            owner,
            family,
            variant,
            flags,
            key,
            row,
            row.width,
            author_style,
            Some(container_patch),
            None,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the private constructor centralizes the public and component-author row paths"
    )]
    fn construct<'a: 'u>(
        ui: &'u mut Ui<'a>,
        owner: Id,
        family: Family,
        variant: Variant,
        flags: StateFlags,
        key: ItemKey,
        row: Rect,
        container_width: u16,
        author_style: crate::author::PartStyle<'u>,
        container_patch: Option<StylePatch>,
        label_patch: Option<StylePatch>,
    ) -> RowUi<'u> {
        let mut ui = ui.reborrow();
        let initial_container_patch =
            merge_patch(author_style.part_patch(Part::CONTAINER), container_patch);
        let container = match initial_container_patch {
            Some(patch) => {
                ui.style_patched(family, variant, Part::CONTAINER, flags, &patch)
                    .style
            }
            None => ui.style(family, variant, Part::CONTAINER, flags).style,
        };
        ui.fill(row, container);
        RowUi {
            ui,
            owner,
            family,
            variant,
            flags,
            key,
            row,
            container_width,
            cursor_style: CursorStyle::Bar,
            author_style,
            author_parts: None,
            container_patch,
            label_patch,
            left: 0,
            right: 0,
            keep_tail: false,
            padded_tail: false,
            last_meta_paint: None,
        }
    }

    fn patch_for(&self, part: Part) -> Option<StylePatch> {
        let mut patch = self
            .author_part(part)
            .then(|| self.author_style.part_patch(part))
            .flatten();
        let scoped = match part {
            Part::CONTAINER => self.container_patch,
            Part::LABEL => self.label_patch,
            _ => None,
        };
        if let Some(scoped) = scoped {
            patch = merge_patch(patch, Some(scoped));
        }
        patch
    }

    fn author_part(&self, part: Part) -> bool {
        self.author_parts.is_none_or(|parts| parts.contains(&part))
    }

    pub fn with_author_parts(mut self, parts: &'u [Part]) -> Self {
        self.author_parts = Some(parts);
        self
    }

    fn fill_patch(&self) -> Option<StylePatch> {
        merge_patch(
            self.author_style.part_patch(Part::ROW),
            self.patch_for(Part::CONTAINER),
        )
    }

    pub const fn flags(&self) -> StateFlags {
        self.flags
    }

    pub const fn key(&self) -> ItemKey {
        self.key
    }

    pub const fn owner(&self) -> Id {
        self.owner
    }

    pub const fn area(&self) -> Rect {
        self.row
    }

    pub const fn container_width(&self) -> u16 {
        self.container_width
    }

    pub fn with_cursor_style(mut self, style: CursorStyle) -> Self {
        self.cursor_style = style;
        if style == CursorStyle::Fill && self.flags.contains(StateFlags::FOCUSED) {
            let patch = self.fill_patch();
            let resolved = self.ui.style_defaults(
                self.family,
                self.variant,
                Part::ROW,
                self.flags,
                crate::theme::StyleDefaults::new(selected_row_patch()),
                patch.as_ref(),
            );
            self.ui.fill(self.row, resolved.style);
        } else if style == CursorStyle::Fill {
            let mut defaults = StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::CurrentSurface)
                .remove(Modifier::BOLD);
            if self.flags.contains(StateFlags::DISABLED) {
                defaults = defaults.set_fg(Role::DisabledFg);
            }
            let patch = self.fill_patch();
            let resolved = self.ui.style_defaults(
                self.family,
                self.variant,
                Part::ROW,
                self.flags,
                crate::theme::StyleDefaults::new(defaults),
                patch.as_ref(),
            );
            self.ui.fill(self.row, resolved.style);
        }
        self
    }

    fn remaining(&self) -> Rect {
        let used = self.left.saturating_add(self.right);
        Rect {
            x: self.row.x.saturating_add(self.left),
            y: self.row.y,
            width: self.row.width.saturating_sub(used),
            height: 1,
        }
    }

    pub fn remaining_width(&self) -> u16 {
        self.remaining().width
    }

    fn style_of(&mut self, part: Part) -> PaintStyle {
        self.style_of_with_defaults(part, crate::theme::StyleDefaults::default())
    }

    fn style_of_with_defaults(
        &mut self,
        part: Part,
        defaults: crate::theme::StyleDefaults<'_>,
    ) -> PaintStyle {
        let patch = self.patch_for(part);
        let r = resolve_row_part_with_defaults(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            part,
            patch,
            self.cursor_style,
            defaults,
        );

        r.style
    }

    pub fn marker(&mut self, g: GlyphRole) {
        let patch = self.patch_for(Part::MARKER);
        self.paint_marker(g, patch);
    }

    pub fn marker_patched(&mut self, g: GlyphRole, patch: &StylePatch) {
        let patch = merge_patch(self.patch_for(Part::MARKER), Some(*patch));
        self.paint_marker(g, patch);
    }

    fn paint_marker(&mut self, g: GlyphRole, patch: Option<StylePatch>) {
        let r = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            Part::MARKER,
            patch,
            self.cursor_style,
        );

        let area = self.remaining();
        let cell = Rect {
            width: area.width.min(1),
            ..area
        };
        match r.glyph {
            Slot::Set(glyph) => {
                let used = self.ui.glyph(area, glyph, r.style);
                self.left = self.left.saturating_add(used).saturating_add(1);
            }
            Slot::Inherit => {
                let used = self.ui.glyph(area, g, r.style);
                self.left = self.left.saturating_add(used).saturating_add(1);
            }
            Slot::Clear => {
                self.ui.fill(cell, r.style);
                self.left = self.left.saturating_add(2);
            }
        }
    }

    pub fn gutter(&mut self) {
        self.leading_part(Part::GUTTER, 1, 0, |_| {});
    }

    pub fn leading_part(
        &mut self,
        part: Part,
        width: u16,
        gap: u16,
        fallback: impl FnOnce(&mut CellUi<'_>),
    ) {
        let area = self.remaining();
        let cell = Rect {
            width: width.min(area.width),
            ..area
        };
        let patch = self.patch_for(part);
        let resolved = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            part,
            patch,
            self.cursor_style,
        );

        let author_style = self.author_style;
        let allowed = self.author_part(part);
        let slot = allowed.then(|| author_style.slot_for(part)).flatten();
        if let Some(slot) = slot {
            slot(&mut self.ui, cell);
        } else {
            match resolved.glyph {
                Slot::Set(glyph) => {
                    self.ui.glyph(cell, glyph, resolved.style);
                }
                Slot::Clear => self.ui.fill(cell, resolved.style),
                Slot::Inherit => {
                    self.ui.fill(cell, resolved.style);
                    let mut content = CellUi::new(self.ui.reborrow(), cell, resolved.style);
                    fallback(&mut content);
                }
            }
        }
        self.left = self.left.saturating_add(width).saturating_add(gap);
    }

    pub fn trailing_part(
        &mut self,
        part: Part,
        width: u16,
        gap: u16,
        fallback: impl FnOnce(&mut CellUi<'_>),
    ) {
        let area = self.remaining();
        let cell_width = width.min(area.width);
        let cell = Rect {
            x: area.right().saturating_sub(cell_width),
            width: cell_width,
            ..area
        };
        let patch = self.patch_for(part);
        let resolved = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            part,
            patch,
            self.cursor_style,
        );

        let author_style = self.author_style;
        let allowed = self.author_part(part);
        let slot = allowed.then(|| author_style.slot_for(part)).flatten();
        if let Some(slot) = slot {
            slot(&mut self.ui, cell);
        } else {
            match resolved.glyph {
                Slot::Set(glyph) => {
                    self.ui.glyph(cell, glyph, resolved.style);
                }
                Slot::Clear => self.ui.fill(cell, resolved.style),
                Slot::Inherit => {
                    self.ui.fill(cell, resolved.style);
                    let mut content = CellUi::new(self.ui.reborrow(), cell, resolved.style);
                    fallback(&mut content);
                }
            }
        }
        self.right = self.right.saturating_add(cell_width).saturating_add(gap);
    }

    pub fn has_space(&self) -> bool {
        !self.remaining().is_empty()
    }

    pub fn reserve_right(&mut self, n: u16) {
        self.right = self.right.saturating_add(n);
    }

    pub fn begin_content(&mut self) {
        self.row = self.remaining();
        self.left = 0;
        self.right = 0;
        self.last_meta_paint = None;
    }

    pub const fn meta_paint(&self) -> Option<MetaPaint> {
        self.last_meta_paint
    }

    fn record_meta_paint(&mut self, needed_width: u16, logical_x: Option<u16>) {
        self.last_meta_paint = Some(MetaPaint {
            needed_width,
            admitted_width: if logical_x.is_some() { needed_width } else { 0 },
            logical_x,
        });
    }

    pub fn indent(&mut self, depth: u16) {
        let step = self.ui.design().space.tree_indent;
        self.left = self.left.saturating_add(depth.saturating_mul(step));
    }

    pub fn label(&mut self, s: &str) {
        let st = self.style_of(Part::LABEL);
        self.label_in(s, st);
    }

    pub fn label_matched(&mut self, s: &str, matched: &[usize]) {
        let patch = self.patch_for(Part::LABEL);
        let resolved = self.ui.style_defaults_with_explicit(
            self.family,
            self.variant,
            Part::LABEL,
            self.flags,
            crate::theme::StyleDefaults::new(StylePatch::new().remove(Modifier::BOLD)),
            patch.as_ref(),
        );

        let style = resolved.resolved.style;

        let area = self.remaining();
        let used = if width(s) <= area.width {
            self.ui.paint_matched(area, s, matched, style)
        } else {
            let head = Rect {
                width: area.width.saturating_sub(1),
                ..area
            };
            let used = self.ui.paint_matched(head, s, matched, style);
            let tail = Rect {
                x: area.x.saturating_add(used),
                width: area.width.saturating_sub(used),
                ..area
            };
            used.saturating_add(self.ui.glyph(tail, GlyphRole::Ellipsis, style))
        };
        self.left = self.left.saturating_add(used);
    }

    pub fn label_patched(&mut self, s: &str, p: &StylePatch) {
        let patch = self
            .patch_for(Part::LABEL)
            .map_or(*p, |forwarded| forwarded.merge(*p));
        let st = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            Part::LABEL,
            Some(patch),
            self.cursor_style,
        )
        .style;
        self.label_in(s, st);
    }

    pub fn label_middle(&mut self, text: &str, patch: &StylePatch) {
        let remaining = self.remaining();
        let slot_width = remaining.width.saturating_sub(1);
        if slot_width == 0 {
            return;
        }

        let patch = self
            .patch_for(Part::LABEL)
            .map_or(*patch, |forwarded| forwarded.merge(*patch));
        let style = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            Part::LABEL,
            Some(patch),
            self.cursor_style,
        )
        .style;
        let slot = Rect {
            width: slot_width,
            ..remaining
        };
        let truncated = truncate_middle(text, slot_width);

        self.ui.fill(slot, style);
        let _ = self.ui.paint_str(slot, &truncated, style);
        self.left = self.left.saturating_add(slot_width);
    }

    pub fn label_end(&mut self, text: &str, patch: &StylePatch) {
        let remaining = self.remaining();
        let slot_width = remaining.width.saturating_sub(1);
        if slot_width == 0 {
            return;
        }

        let patch = self
            .patch_for(Part::LABEL)
            .map_or(*patch, |forwarded| forwarded.merge(*patch));
        let style = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            Part::LABEL,
            Some(patch),
            self.cursor_style,
        )
        .style;
        let slot = Rect {
            width: slot_width,
            ..remaining
        };

        self.ui.fill(slot, style);
        let truncated = truncate(text, slot_width);
        let _ = self.ui.paint_str(slot, &truncated, style);
        self.left = self.left.saturating_add(slot_width);

        if !self.keep_tail && width(text) <= slot_width {
            self.ui.fill(
                Rect {
                    x: remaining.x.saturating_add(slot_width),
                    width: 1,
                    ..remaining
                },
                style,
            );
            self.left = self.left.saturating_add(1);
            self.padded_tail = true;
        }
    }

    pub fn keep_row_tail(&mut self) {
        self.keep_tail = true;
    }

    pub const fn padded_tail(&self) -> bool {
        self.padded_tail
    }

    fn label_in(&mut self, s: &str, st: PaintStyle) {
        let area = self.remaining();
        let used = if width(s) <= area.width {
            self.ui.paint_str(area, s, st)
        } else {
            let head = Rect {
                width: area.width.saturating_sub(1),
                ..area
            };
            let used = self.ui.paint_str(head, s, st);
            let tail = Rect {
                x: area.x.saturating_add(used),
                width: area.width.saturating_sub(used),
                ..area
            };
            used.saturating_add(self.ui.glyph(tail, GlyphRole::Ellipsis, st))
        };
        self.left = self.left.saturating_add(used);
    }

    pub fn label_spans(&mut self, spans: &[Span<'_>]) {
        let st = self.style_of(Part::LABEL);
        let area = self.remaining();
        let used = self.ui.paint_spans(area, spans, st);
        self.left = self.left.saturating_add(used);
    }

    pub fn label_fmt(&mut self, args: fmt::Arguments<'_>) {
        let st = self.style_of(Part::LABEL);
        let area = self.remaining();
        let mut w = CellWriter {
            ui: &mut self.ui,
            area,
            x: area.x,
            style: st,
        };
        let _ = w.write_fmt(args);
        let used = w.x.saturating_sub(area.x);
        self.left = self.left.saturating_add(used);
    }

    pub fn meta(&mut self, s: &str) {
        self.meta_with_gap(s, 1);
    }

    pub fn meta_with_gap(&mut self, s: &str, gap: u16) {
        let need = width(s);
        let area = self.remaining();
        let Some(metadata_budget) = area
            .width
            .checked_sub(gap)
            .and_then(|width| width.checked_sub(1))
        else {
            self.record_meta_paint(need, None);
            return;
        };
        if need == 0 || need > metadata_budget {
            self.record_meta_paint(need, None);
            return;
        }
        let st = self.style_of(Part::META);
        let cell = Rect {
            x: area.right().saturating_sub(need),
            y: area.y,
            width: need,
            height: 1,
        };
        self.ui.paint_str(cell, s, st);
        self.right = self.right.saturating_add(need).saturating_add(gap);
        self.record_meta_paint(need, Some(cell.x));
    }

    pub fn meta_with_defaults(&mut self, s: &str, defaults: termrock_theme::StyleDefaults<'_>) {
        let need = width(s);
        let area = self.remaining();
        if need == 0 || need.saturating_add(2) > area.width {
            self.record_meta_paint(need, None);
            return;
        }
        let st = self.style_of_with_defaults(Part::META, defaults);
        let cell = Rect {
            x: area.right().saturating_sub(need),
            y: area.y,
            width: need,
            height: 1,
        };
        self.ui.paint_str(cell, s, st);
        self.right = self.right.saturating_add(need).saturating_add(1);
        self.record_meta_paint(need, Some(cell.x));
    }

    pub fn glyph_before_meta(&mut self, role: GlyphRole, patch: &StylePatch) {
        let Some(meta) = self.last_meta_paint else {
            return;
        };
        if meta.admitted_width() == 0 || self.row.height == 0 {
            return;
        }
        let Some(meta_x) = meta.logical_x() else {
            return;
        };
        let Some(x) = meta_x.checked_sub(4) else {
            return;
        };
        if x < self.row.x || x >= self.row.right() {
            return;
        }

        let patch = self
            .patch_for(Part::ICON)
            .map_or(*patch, |authored| authored.merge(*patch));
        let mut style = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            Part::ICON,
            Some(patch),
            self.cursor_style,
        )
        .style;
        if matches!(patch.bg, Slot::Inherit)
            && let Some(bg) = self
                .ui
                .buffer()
                .cell((x, self.row.y))
                .and_then(|cell| cell.style().bg)
        {
            style = style.bg(bg);
        }

        let area = Rect::new(x, self.row.y, 1, 1);
        if self.ui.glyph(area, role, style) > 0 {
            self.ui.patch_style(area, style, patch);
        }
    }

    pub fn trailing(&mut self, s: &str, p: &StylePatch) {
        let need = width(s);
        let area = self.remaining();
        if need == 0 || need > area.width {
            return;
        }
        let st = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            Part::META,
            Some(*p),
            self.cursor_style,
        )
        .style;
        let cell = Rect {
            x: area.right().saturating_sub(need),
            y: area.y,
            width: need,
            height: 1,
        };
        self.ui.paint_str(cell, s, st);
        self.right = self.right.saturating_add(need).saturating_add(1);
    }

    pub fn part(&mut self, p: Part, width: u16) -> CellUi<'_> {
        let area = self.remaining();
        let w = width.min(area.width);
        let cell = Rect {
            x: area.right().saturating_sub(w),
            y: area.y,
            width: w,
            height: 1,
        };
        self.right = self.right.saturating_add(w).saturating_add(1);
        let r = resolve_row_part(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            p,
            None,
            self.cursor_style,
        );

        CellUi::with_resolved_glyph(self.ui.reborrow(), cell, r.style, r.glyph)
    }

    pub fn columns(&mut self, widths: &[Track]) -> ColumnsUi<'_> {
        self.columns_with_gap(widths, self.ui.design().space.column_gap)
    }

    pub fn columns_with_gap(&mut self, widths: &[Track], gap: u16) -> ColumnsUi<'_> {
        let area = self.remaining();
        let mut sizes = [0u16; MAX_COLUMNS];
        let n = widths.len().min(MAX_COLUMNS);
        distribute_into(area.width, widths.get(..n).unwrap_or(&[]), gap, &mut sizes);
        let style = self.style_of(Part::CELL);
        self.left = self.left.saturating_add(area.width);
        ColumnsUi {
            family: self.family,
            variant: self.variant,
            flags: self.flags,
            label_patch: self.label_patch,
            ui: self.ui.reborrow(),
            area,
            sizes,
            n,
            gap,
            style,
            cursor_style: self.cursor_style,
        }
    }

    pub fn raw(&mut self) -> (&mut Buffer, Rect) {
        let row = self.row;
        let (buf, _) = self.ui.buffer_in(row);
        (buf, row)
    }
}

struct CellWriter<'a, 'u> {
    ui: &'a mut Ui<'u>,
    area: Rect,
    x: u16,
    style: PaintStyle,
}

impl fmt::Write for CellWriter<'_, '_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let right = self.area.right();
        if self.x >= right {
            return Ok(());
        }
        let cell = Rect {
            x: self.x,
            y: self.area.y,
            width: right.saturating_sub(self.x),
            height: 1,
        };
        let used = self.ui.paint_str(cell, s, self.style);
        self.x = self.x.saturating_add(used);
        Ok(())
    }
}

pub struct CellUi<'u> {
    ui: Ui<'u>,
    area: Rect,
    style: PaintStyle,
    reset_style: PaintStyle,
    used: u16,
    align: Align,
    tone: Option<Role>,
    add: Modifier,
    patch: Option<StylePatch>,
    resolved_glyph: Slot<GlyphRole>,
}

impl fmt::Debug for CellUi<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CellUi")
            .field("area", &self.area)
            .field("used", &self.used)
            .field("align", &self.align)
            .finish_non_exhaustive()
    }
}

impl<'u> CellUi<'u> {
    pub fn new(ui: Ui<'u>, area: Rect, style: PaintStyle) -> Self {
        Self::with_resolved_glyph(ui, area, style, Slot::Inherit)
    }

    fn with_resolved_glyph(
        ui: Ui<'u>,
        area: Rect,
        style: PaintStyle,
        resolved_glyph: Slot<GlyphRole>,
    ) -> Self {
        CellUi {
            ui,
            area,
            style,
            reset_style: style,
            used: 0,
            align: Align::Left,
            tone: None,
            add: Modifier::empty(),
            patch: None,
            resolved_glyph,
        }
    }

    fn resolved_glyph_width(&self) -> u16 {
        match self.resolved_glyph {
            Slot::Set(g) => width(self.ui.design().glyphs.get(g)),
            Slot::Clear => 1,
            Slot::Inherit => 0,
        }
    }

    fn free(&self) -> Rect {
        let reserved = self.resolved_glyph_width();
        Rect {
            x: self.area.x.saturating_add(self.used),
            y: self.area.y,
            width: self
                .area
                .width
                .saturating_sub(self.used)
                .saturating_sub(reserved),
            height: 1,
        }
    }

    pub fn available_width(&self) -> u16 {
        self.free().width
    }

    fn paint_resolved_glyph(&mut self) {
        let reserved = self.resolved_glyph_width();
        let free = self.area.width.saturating_sub(self.used);
        if reserved == 0 || free == 0 {
            return;
        }
        let cell = Rect {
            x: self.area.x.saturating_add(self.used),
            y: self.area.y,
            width: reserved.min(free),
            height: 1,
        };
        let used = match self.resolved_glyph {
            Slot::Set(g) => self.ui.glyph(cell, g, self.style),

            Slot::Clear => {
                self.ui.fill(cell, self.style);
                cell.width
            }
            Slot::Inherit => 0,
        };
        self.used = self.used.saturating_add(used);
    }

    pub fn text(&mut self, s: &str) -> &mut Self {
        let area = self.free();
        let used = self.ui.paint_str(area, s, self.style);
        self.used = self.used.saturating_add(used);
        self
    }

    pub fn remove_modifier(&mut self, modifiers: Modifier) -> &mut Self {
        self.style = self.style.remove_modifier(modifiers);
        self
    }

    pub fn authored_modifiers(&self) -> Modifier {
        self.style.add_modifier
    }

    pub fn text_matched(&mut self, s: &str, matched: &[usize]) -> &mut Self {
        let area = self.free();
        let used = if width(s) <= area.width {
            self.ui.paint_matched(area, s, matched, self.style)
        } else {
            let head = Rect {
                width: area.width.saturating_sub(1),
                ..area
            };
            let used = self.ui.paint_matched(head, s, matched, self.style);
            let tail = Rect {
                x: area.x.saturating_add(used),
                width: area.width.saturating_sub(used),
                ..area
            };
            used.saturating_add(self.ui.glyph(tail, GlyphRole::Ellipsis, self.style))
        };
        self.used = self.used.saturating_add(used);
        self
    }

    pub fn glyphs(&mut self, g: GlyphRole, n: usize) -> &mut Self {
        let sym = self.ui.design().glyphs.get(g);
        for _ in 0..n {
            let area = self.free();
            if area.is_empty() {
                break;
            }
            let used = self.ui.paint_str(area, sym, self.style);
            if used == 0 {
                break;
            }
            self.used = self.used.saturating_add(used);
        }
        self
    }

    pub fn num(&mut self, n: i64) -> &mut Self {
        let mut buf = [0u8; 24];
        let s = format_i64(n, &mut buf);
        self.text(s)
    }

    pub fn money(&mut self, cents: i64) -> &mut Self {
        let mut buf = [0u8; 32];
        let s = format_money(cents, &mut buf);
        self.text(s)
    }

    pub const fn align(&mut self, a: Align) -> &mut Self {
        self.align = a;
        self
    }

    pub const fn padding_style(&mut self, style: PaintStyle) -> &mut Self {
        self.reset_style = style;
        self
    }

    pub const fn tone(&mut self, r: Role) -> &mut Self {
        self.tone = Some(r);
        self
    }

    pub fn italic(&mut self, yes: bool) -> &mut Self {
        self.add = if yes {
            self.add.union(Modifier::ITALIC)
        } else {
            self.add.difference(Modifier::ITALIC)
        };
        self
    }

    pub fn suffix(&mut self, g: GlyphRole) -> &mut Self {
        let area = self.free();
        let used = self.ui.glyph(area, g, self.style);
        self.used = self.used.saturating_add(used);
        self
    }

    pub fn suffix_style(&mut self, g: GlyphRole, style: PaintStyle) -> &mut Self {
        let area = self.free();
        let used = self.ui.glyph(area, g, style);
        self.used = self.used.saturating_add(used);
        self
    }

    pub fn patch(&mut self, p: &StylePatch) -> &mut Self {
        self.patch = Some(match self.patch {
            Some(cur) => cur.merge(*p),
            None => *p,
        });
        self
    }

    pub fn color_level(&self) -> crate::theme::ColorLevel {
        self.ui.theme_ref().capability.color
    }
}

impl Drop for CellUi<'_> {
    fn drop(&mut self) {
        self.paint_resolved_glyph();
        let used = self.used.min(self.area.width);
        if used == 0 {
            return;
        }

        let free = self.area.width.saturating_sub(used);
        let shift = match self.align {
            Align::Left => 0,
            Align::Center => free / 2,
            Align::Right => free,
        };
        let y = self.area.y;
        if shift > 0 {
            let mut x = self.area.x.saturating_add(used);
            while x > self.area.x {
                x = x.saturating_sub(1);
                let src = Position::new(x, y);
                let dst = Position::new(x.saturating_add(shift), y);
                self.ui.move_cell(src, dst, self.reset_style);
            }
        }

        let painted = Rect {
            x: self.area.x.saturating_add(shift),
            y,
            width: used,
            height: 1,
        };
        let theme = self.ui.theme_ref();
        let surface = self.ui.surface();
        let mut delta = StylePatch::new().add(self.add);
        if let Some(r) = self.tone {
            delta = delta.set_fg(r);
        }
        let st = crate::theme::resolve::bind(theme, delta, self.patch.as_ref(), surface).style;
        if st != PaintStyle::new() {
            self.ui.paint_style(painted, st);
        }
    }
}

pub struct ColumnsUi<'u> {
    family: Family,
    variant: Variant,
    flags: StateFlags,
    label_patch: Option<StylePatch>,
    ui: Ui<'u>,
    area: Rect,
    sizes: [u16; MAX_COLUMNS],
    n: usize,
    gap: u16,
    style: PaintStyle,
    cursor_style: CursorStyle,
}

impl fmt::Debug for ColumnsUi<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ColumnsUi")
            .field("area", &self.area)
            .field("n", &self.n)
            .finish_non_exhaustive()
    }
}

impl ColumnsUi<'_> {
    pub const fn len(&self) -> usize {
        self.n
    }

    pub const fn is_empty(&self) -> bool {
        self.n == 0
    }

    pub fn rect(&self, i: usize) -> Rect {
        if i >= self.n {
            return Rect::ZERO;
        }
        let mut x = self.area.x;
        for (k, w) in self.sizes.iter().enumerate().take(i) {
            let _ = k;
            x = x.saturating_add(*w).saturating_add(self.gap);
        }
        let w = self.sizes.get(i).copied().unwrap_or(0);
        Rect {
            x,
            y: self.area.y,
            width: w.min(self.area.right().saturating_sub(x)),
            height: 1,
        }
        .intersection(self.area)
    }

    pub fn cell_part(&mut self, i: usize, part: Part) -> CellUi<'_> {
        self.cell_part_with_defaults(i, part, termrock_theme::StyleDefaults::default())
    }

    pub fn cell_part_with_defaults(
        &mut self,
        i: usize,
        part: Part,
        defaults: termrock_theme::StyleDefaults<'_>,
    ) -> CellUi<'_> {
        let rect = self.rect(i);
        let patch = (part == Part::LABEL).then_some(self.label_patch).flatten();
        let resolved = resolve_row_part_with_defaults(
            &mut self.ui,
            self.family,
            self.variant,
            self.flags,
            part,
            patch,
            self.cursor_style,
            defaults,
        );

        CellUi::with_resolved_glyph(self.ui.reborrow(), rect, resolved.style, resolved.glyph)
    }

    pub fn cell(&mut self, i: usize) -> CellUi<'_> {
        let rect = self.rect(i);
        let style = self.style;
        CellUi::new(self.ui.reborrow(), rect, style)
    }
}

fn format_i64(n: i64, buf: &mut [u8; 24]) -> &str {
    let mut i = buf.len();
    let neg = n < 0;
    let mut v = n.unsigned_abs();
    if v == 0 {
        i = i.saturating_sub(1);
        if let Some(b) = buf.get_mut(i) {
            *b = b'0';
        }
    }
    while v > 0 {
        i = i.saturating_sub(1);
        if let Some(b) = buf.get_mut(i) {
            *b = b'0'.saturating_add((v % 10) as u8);
        }
        v /= 10;
    }
    if neg {
        i = i.saturating_sub(1);
        if let Some(b) = buf.get_mut(i) {
            *b = b'-';
        }
    }
    core::str::from_utf8(buf.get(i..).unwrap_or(&[])).unwrap_or("")
}

fn format_money(cents: i64, buf: &mut [u8; 32]) -> &str {
    let mut i = buf.len();
    let neg = cents < 0;
    let v = cents.unsigned_abs();
    let (whole, frac) = (v / 100, v % 100);
    let mut put = |b: u8| {
        i = i.saturating_sub(1);
        if let Some(slot) = buf.get_mut(i) {
            *slot = b;
        }
    };
    put(b'0'.saturating_add((frac % 10) as u8));
    put(b'0'.saturating_add((frac / 10) as u8));
    put(b'.');
    let mut w = whole;
    let mut digits = 0u32;
    loop {
        put(b'0'.saturating_add((w % 10) as u8));
        w /= 10;
        digits = digits.saturating_add(1);
        if w == 0 {
            break;
        }
        if digits.is_multiple_of(3) {
            put(b',');
        }
    }
    if neg {
        put(b'-');
    }
    core::str::from_utf8(buf.get(i..).unwrap_or(&[])).unwrap_or("")
}

impl termrock_text::secret::CellWriter for CellUi<'_> {
    fn glyphs(&mut self, mask: crate::theme::GlyphRole, n: usize) {
        self.glyphs(mask, n);
    }

    fn text(&mut self, s: &str) {
        self.text(s);
    }
}
