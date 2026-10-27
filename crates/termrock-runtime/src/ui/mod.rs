mod command_scope;
pub mod composition;
pub mod cx;
pub mod derived;
pub mod layer_buf;
pub mod measure;
pub mod paint;
mod slot;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Color;

use core::ops::{BitOr, BitOrAssign};
use std::collections::HashMap;

pub use composition::{CompositionError, CompositionSlots, ContentSlot, MAX_COMPOSITION_DEPTH};
use cx::LastFrame;
pub use cx::{Cx, FrameRead, LayoutFacts};
use derived::{DerivedCache, ReferenceCacheKey};
use layer_buf::LayerPool;
pub use slot::LayerSlot;

use crate::action::ActionKey;
use crate::cursor::CursorRequest;
use crate::diagnostics::{Diagnostic, Diagnostics};
use crate::focus::{FocusEntry, FocusRing, Focusability, ScopeId, ScopeMode};
use crate::hit::{Axes, Headroom, Registry, ScrollRouteKey, TextDragRegistration};
use crate::id::{Id, Part, PartRef};
use crate::keymap::{
    Binding, BindingDescriptor, BindingExtensionError, BindingRegistry, DynamicBindingRegistry,
    FocusedHintKey, FocusedHints, HintLayer, KeyMap, ScopedBubbleBinding,
};
use crate::layer::{LayerId, LayerKind, LayerOpening};
use crate::response::StateFlags;
use crate::theme::resolve::StyleCache;
use crate::theme::{DesignTokens, Family, Overlay, Resolved, Role, Surface, Theme, Variant};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct CellRoles {
    pub(crate) fg: Option<Role>,
    pub(crate) bg: Option<Role>,
}

impl CellRoles {
    fn patch(self, style: Option<crate::theme::PaintStyle>) -> Self {
        match style {
            Some(style) => Self {
                fg: if style.fg.is_none() {
                    self.fg
                } else {
                    style.fg_role.map(|(role, surface)| role.painted(surface))
                },
                bg: if style.bg.is_none() {
                    self.bg
                } else {
                    style.bg_role.map(|(role, surface)| role.painted(surface))
                },
            },
            None => Self::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct FrameState {
    pub registry: Registry,
    pub(crate) ring: FocusRing,
    pub(crate) layers: LayerPool,
    pub(crate) layer_openings: Vec<LayerOpening>,
    pub(crate) managed_tooltip_opening: Option<LayerOpening>,
    pub(crate) managed_tooltip_drawn: bool,
    pub(crate) cursors: Vec<CursorRequest>,
    pub(crate) retained_cursor_request: Option<RetainedCursorRequest>,
    pub(crate) typing: Vec<crate::runtime::typing::TypingDeclaration>,
    pub(crate) focused_paste_targets: Vec<crate::runtime::typing::FocusedPasteDeclaration>,
    pub(crate) typing_bindings: BindingRegistry,
    pub(crate) typing_resolved: crate::runtime::typing::TypingResolved,
    pub(crate) layout: Vec<(Id, LayoutFacts)>,
    pub(crate) declared: Vec<(Id, StateFlags)>,
    pub(crate) bindings: BindingRegistry,
    pub(crate) scoped_bubble_bindings: Vec<ScopedBubbleBinding>,
    pub(crate) diagnostics: Diagnostics,
    pub(crate) tooltips: crate::Tooltips,
    pub(crate) roles: Vec<CellRoles>,
    pub(crate) screen: Rect,
    pub(crate) inert_floor: LayerId,
    pub(crate) top: LayerId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RetainedCursorRequest {
    pub(crate) owner: Id,
    pub(crate) parent_layer: LayerId,
    pub(crate) overlay_layer: LayerId,
    pub(crate) cursor_index: usize,
    pub(crate) mode: CursorRetentionMode,
    pub(crate) ambiguous: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CursorRetentionMode {
    FocusedOwner,
    VisibleOwner,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct ReferenceState(u8);

impl ReferenceState {
    pub const FOCUSED: Self = Self(1 << 0);
    pub const FOCUS_VISIBLE: Self = Self(1 << 1);
    pub const HOVERED: Self = Self(1 << 2);
    pub const PRESSED: Self = Self(1 << 3);

    #[must_use]
    pub const fn union(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }

    const fn state_flags(self) -> StateFlags {
        let mut flags = StateFlags::empty();
        if self.0 & Self::FOCUSED.0 != 0 {
            flags = flags.union(StateFlags::FOCUSED);
        }
        if self.0 & Self::FOCUS_VISIBLE.0 != 0 {
            flags = flags.union(StateFlags::FOCUSED);
            flags = flags.union(StateFlags::FOCUS_VISIBLE);
        }
        if self.0 & Self::HOVERED.0 != 0 {
            flags = flags.union(StateFlags::HOVERED);
        }
        if self.0 & Self::PRESSED.0 != 0 {
            flags = flags.union(StateFlags::PRESSED);
        }
        flags
    }
}

impl BitOr for ReferenceState {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl BitOrAssign for ReferenceState {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ReferenceTarget {
    id: Id,
    part: Option<PartRef>,
    state: ReferenceState,
}

impl ReferenceTarget {
    pub const fn new(id: Id, state: ReferenceState) -> Self {
        Self {
            id,
            part: None,
            state,
        }
    }

    #[must_use]
    pub const fn part(mut self, part: PartRef) -> Self {
        self.part = Some(part);
        self
    }
}

#[derive(Clone, Copy, Debug)]
struct ReferenceScope {
    target: Option<ReferenceTarget>,
    cache_key: ReferenceCacheKey,
}

impl FrameState {
    pub fn reset(&mut self, generation: u32, screen: Rect) {
        self.registry.reset(generation);
        self.ring.reset();
        self.layers.begin();
        self.layer_openings.clear();
        self.managed_tooltip_opening = None;
        self.managed_tooltip_drawn = false;
        self.tooltips = crate::Tooltips::DEFAULT;
        self.cursors.clear();
        self.retained_cursor_request = None;
        self.typing.clear();
        self.focused_paste_targets.clear();
        self.typing_bindings.reset();
        self.typing_resolved = crate::runtime::typing::TypingResolved::default();
        self.layout.clear();
        self.declared.clear();
        self.bindings.reset();
        self.scoped_bubble_bindings.clear();
        self.diagnostics.clear();
        self.roles.clear();
        self.roles
            .resize(screen.area() as usize, CellRoles::default());
        self.screen = screen;
        self.inert_floor = LayerId::PAGE;
        self.top = LayerId::PAGE;
    }

    fn role_index(&self, pos: Position) -> Option<usize> {
        if !self.screen.contains(pos) {
            return None;
        }
        let row = usize::from(pos.y.saturating_sub(self.screen.y));
        let col = usize::from(pos.x.saturating_sub(self.screen.x));
        Some(
            row.saturating_mul(usize::from(self.screen.width))
                .saturating_add(col),
        )
    }
}

#[derive(Default)]
pub struct UiCore {
    cache: DerivedCache,
    reference_cache: DerivedCache,
    pub(crate) style_cache: StyleCache,
    overlays: Vec<Overlay>,
    stack_hash: u64,
    pub(crate) focused_hints: FocusedHints,
    pub(crate) keymap: KeyMap,
    pub(crate) keymap_revision: u64,
    dynamic_bindings: DynamicBindingRegistry,
    next_targetless_reference: u64,
    pub(crate) cluster_scratch: crate::text::clusters::ClusterScratch,
    pub(crate) majority_bg_counts: HashMap<Color, (usize, usize)>,
}

impl core::fmt::Debug for UiCore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UiCore")
            .field("cache", &self.cache)
            .field("overlays", &self.overlays.len())
            .field("stack_hash", &self.stack_hash)
            .finish_non_exhaustive()
    }
}

impl UiCore {
    pub(crate) fn begin_cache_frame(&mut self, generation: u32) {
        self.cache.begin_frame(generation);
        self.reference_cache.begin_frame(generation);
    }

    pub(crate) fn clear_caches(&mut self) {
        self.cache.clear();
        self.reference_cache.clear();
        self.style_cache.clear();
    }

    fn targetless_reference_key(&mut self) -> ReferenceCacheKey {
        let key = self.next_targetless_reference;
        self.next_targetless_reference = self.next_targetless_reference.wrapping_add(1);
        if self.next_targetless_reference == 0 {
            self.reference_cache.clear();
        }
        ReferenceCacheKey::Targetless(key)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Target {
    Page,
    Layer(usize),
}

struct OverlayGuard<'a, 'f> {
    ui: &'a mut Ui<'f>,
    saved_hash: u64,
}

impl Drop for OverlayGuard<'_, '_> {
    fn drop(&mut self) {
        self.ui.core.overlays.pop();
        self.ui.core.stack_hash = self.saved_hash;
    }
}

struct FocusScopeGuard<'a, 'f> {
    ui: &'a mut Ui<'f>,
}

impl Drop for FocusScopeGuard<'_, '_> {
    fn drop(&mut self) {
        self.ui.frame.ring.pop_scope();
    }
}

pub struct Ui<'f> {
    frame: &'f mut FrameState,
    page: &'f mut Buffer,
    core: &'f mut UiCore,
    theme: &'f Theme,
    last: &'f LastFrame,
    clip: Rect,
    surface: Surface,
    target: Target,
    layer: LayerId,
    inert: bool,
    reference: Option<ReferenceScope>,
}

impl core::fmt::Debug for Ui<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Ui")
            .field("clip", &self.clip)
            .field("surface", &self.surface)
            .field("layer", &self.layer)
            .field("inert", &self.inert)
            .field("reference", &self.reference)
            .finish_non_exhaustive()
    }
}

impl<'f> Ui<'f> {
    pub const fn tooltips(&self) -> crate::Tooltips {
        self.frame.tooltips
    }

    pub fn new(
        frame: &'f mut FrameState,
        page: &'f mut Buffer,
        core: &'f mut UiCore,
        theme: &'f Theme,
        last: &'f LastFrame,
    ) -> Self {
        let clip = frame.screen;
        let inert = frame.inert_floor > LayerId::PAGE;
        Ui {
            frame,
            page,
            core,
            theme,
            last,
            clip,
            surface: Surface::Canvas,
            target: Target::Page,
            layer: LayerId::PAGE,
            inert,
            reference: None,
        }
    }

    pub fn reborrow(&mut self) -> Ui<'_> {
        Ui {
            frame: &mut *self.frame,
            page: &mut *self.page,
            core: &mut *self.core,
            theme: self.theme,
            last: self.last,
            clip: self.clip,
            surface: self.surface,
            target: self.target,
            layer: self.layer,
            inert: self.inert,
            reference: self.reference,
        }
    }

    pub fn with_theme<R>(&mut self, theme: &Theme, f: impl FnOnce(&mut Ui<'_>) -> R) -> R {
        let mut nested = Ui {
            frame: &mut *self.frame,
            page: &mut *self.page,
            core: &mut *self.core,
            theme,
            last: self.last,
            clip: self.clip,
            surface: self.surface,
            target: self.target,
            layer: self.layer,
            inert: self.inert,
            reference: self.reference,
        };
        f(&mut nested)
    }

    pub const fn full(&self) -> Rect {
        self.clip
    }

    pub const fn surface(&self) -> Surface {
        self.surface
    }

    pub fn bg(&self) -> Color {
        self.theme.bg(self.surface)
    }

    pub const fn layer_id(&self) -> LayerId {
        self.layer
    }

    pub fn is_covered_by_native_layer(&self) -> bool {
        self.reference.is_none() && self.layer != self.frame.top
    }

    pub fn owner_has_focus_presentation(&self, owner: Id) -> bool {
        match self.reference_state(owner) {
            Some(state) => state.contains(StateFlags::FOCUSED),
            None => self.last.snapshot.focus == Some(owner),
        }
    }

    pub const fn is_inert(&self) -> bool {
        self.inert || self.reference.is_some()
    }

    pub const fn context_state(&self) -> StateFlags {
        if self.is_inert() {
            StateFlags::INACTIVE_LAYER
        } else {
            StateFlags::empty()
        }
    }

    pub const fn is_inactive_layer(&self) -> bool {
        self.is_inert()
    }

    pub fn diagnostic(&mut self, diagnostic: Diagnostic) {
        self.frame.diagnostics.push(diagnostic);
    }

    pub fn reference<R>(
        &mut self,
        target: Option<ReferenceTarget>,
        f: impl FnOnce(&mut Ui<'_>) -> R,
    ) -> R {
        let cache_key = target.map_or_else(
            || self.core.targetless_reference_key(),
            ReferenceCacheKey::Target,
        );
        let mut nested = self.reborrow();
        nested.reference = Some(ReferenceScope { target, cache_key });
        f(&mut nested)
    }

    fn reference_state(&self, id: Id) -> Option<StateFlags> {
        self.reference.map(|scope| {
            scope.target.map_or(StateFlags::empty(), |target| {
                if target.id == id {
                    target.state.state_flags()
                } else {
                    StateFlags::empty()
                }
            })
        })
    }

    fn reference_part(&self, owner: Id, state: ReferenceState) -> Option<PartRef> {
        self.reference
            .and_then(|scope| scope.target)
            .filter(|target| target.id == owner && target.state.0 & state.0 != 0)
            .map(|target| target.part.unwrap_or_else(|| PartRef::of(Part::CONTAINER)))
    }

    fn registrations_suppressed(&self) -> bool {
        self.inert || self.reference.is_some()
    }

    pub fn style(
        &mut self,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
    ) -> Resolved {
        self.core.style_cache.resolve(
            self.theme,
            family,
            variant,
            part,
            flags,
            &self.core.overlays,
            self.core.stack_hash,
            self.surface,
        )
    }

    pub(crate) fn style_inherited(
        &self,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
        inherited: crate::theme::PaintStyle,
    ) -> Resolved {
        crate::theme::resolve::bind_inherited(
            self.theme,
            (family, variant, part),
            flags,
            &self.core.overlays,
            self.surface,
            inherited,
        )
    }

    pub fn style_defaults(
        &self,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
        defaults: crate::theme::StyleDefaults<'_>,
        local: Option<&crate::theme::StylePatch>,
    ) -> Resolved {
        crate::theme::resolve::bind_defaults(
            self.theme,
            (family, variant, part),
            flags,
            &self.core.overlays,
            self.surface,
            defaults,
            local,
        )
    }

    pub fn style_defaults_with_explicit(
        &self,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
        defaults: crate::theme::StyleDefaults<'_>,
        local: Option<&crate::theme::StylePatch>,
    ) -> crate::theme::ResolvedStyle {
        crate::theme::resolve::bind_defaults_with_explicit(
            self.theme,
            (family, variant, part),
            flags,
            &self.core.overlays,
            self.surface,
            defaults,
            local,
        )
    }

    pub fn style_defaults_with_fallback_ownership(
        &self,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
        defaults: crate::theme::StyleDefaults<'_>,
        local: Option<&crate::theme::StylePatch>,
    ) -> (Resolved, crate::theme::StylePatch) {
        crate::theme::resolve::bind_defaults_with_fallback_ownership(
            self.theme,
            (family, variant, part),
            flags,
            &self.core.overlays,
            self.surface,
            defaults,
            local,
        )
    }

    pub fn paint_patch(&self, patch: &crate::theme::StylePatch) -> crate::theme::PaintStyle {
        crate::theme::resolve::bind(self.theme, *patch, None, self.surface).style
    }

    pub fn style_patched(
        &mut self,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
        patch: &crate::theme::StylePatch,
    ) -> Resolved {
        self.core.style_cache.resolve_patched(
            self.theme,
            family,
            variant,
            part,
            flags,
            &self.core.overlays,
            self.core.stack_hash,
            self.surface,
            patch,
        )
    }

    pub fn resolve(
        &self,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
    ) -> Resolved {
        crate::theme::resolve::resolve_uncached(
            self.theme,
            family,
            variant,
            part,
            flags,
            self.surface,
            &self.core.overlays,
            None,
        )
    }

    pub fn glyph_str(&self, g: crate::theme::GlyphRole) -> &'static str {
        self.theme.design.glyphs.get(g)
    }

    pub fn surface_style(&self) -> crate::theme::PaintStyle {
        let mut st = ratatui_core::style::Style::new();
        st.bg = Some(self.theme.bg(self.surface));
        st.fg = crate::theme::resolve::bind_role(
            self.theme,
            Role::Fg(crate::theme::FgStep::Primary),
            self.surface,
        );
        crate::theme::PaintStyle::bound(
            st,
            Some(Role::Fg(crate::theme::FgStep::Primary)),
            Some(Role::Surface(self.surface)),
            self.surface,
        )
    }

    pub fn with_part<R>(
        &mut self,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
        f: impl FnOnce(&mut Ui<'_>, Resolved) -> R,
    ) -> R {
        let r = self.style(family, variant, part, flags);
        f(&mut self.reborrow(), r)
    }

    pub(crate) fn with_part_hit_precedence<R>(
        &mut self,
        part: PartRef,
        f: impl FnOnce(&mut Ui<'_>) -> R,
    ) -> R {
        let start = self.frame.registry.len();
        let result = {
            let mut nested = self.reborrow();
            f(&mut nested)
        };
        self.frame.registry.promote_part_hit_suffix(start, part);
        result
    }

    pub fn with_area<R>(&mut self, area: Rect, f: impl FnOnce(&mut Ui<'_>) -> R) -> R {
        let clip = self.clip.intersection(area);
        let mut nested = self.reborrow();
        nested.clip = clip;
        f(&mut nested)
    }

    pub fn with_surface<R>(&mut self, s: Surface, f: impl FnOnce(&mut Ui<'_>) -> R) -> R {
        let mut nested = self.reborrow();
        nested.surface = s;
        f(&mut nested)
    }

    pub fn with_overlay<R>(&mut self, ov: &Overlay, f: impl FnOnce(&mut Ui<'_>) -> R) -> R {
        let saved_hash = self.core.stack_hash;
        self.core.overlays.push(*ov);
        self.core.stack_hash = saved_hash.rotate_left(7) ^ ov.hash();
        let guard = OverlayGuard {
            ui: self,
            saved_hash,
        };
        let mut nested = guard.ui.reborrow();
        f(&mut nested)
    }

    pub fn focus_scope<R>(
        &mut self,
        id: Id,
        mode: ScopeMode,
        f: impl FnOnce(&mut Ui<'_>) -> R,
    ) -> R {
        if self.reference.is_some() {
            return f(&mut self.reborrow());
        }
        self.frame
            .ring
            .push_scope(ScopeId::new(id), mode, self.layer);
        let guard = FocusScopeGuard { ui: self };
        let mut nested = guard.ui.reborrow();
        f(&mut nested)
    }

    fn register_entry(
        &mut self,
        id: Id,
        area: Rect,
        f: Focusability,
        swallows_typing: bool,
        pointer_enabled: bool,
    ) {
        if self.registrations_suppressed() || area.is_empty() {
            return;
        }
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return;
        }
        let diagnostic = if pointer_enabled {
            self.frame.registry.register_control(id, area, self.layer)
        } else {
            self.frame
                .registry
                .register_keyboard_control(id, area, self.layer)
        };
        if let Some(d) = diagnostic {
            self.frame.diagnostics.push(d);
        }
        self.register_focus_entry(id, area, f, swallows_typing);
    }

    fn register_focus_entry(&mut self, id: Id, area: Rect, f: Focusability, swallows_typing: bool) {
        let disabled = match f {
            Focusability::Focusable | Focusability::FocusableReadOnly => false,
            Focusability::Disabled => true,
            Focusability::ClickOnly => return,
        };
        let scope = self.frame.ring.current_scope();
        self.frame.ring.register(FocusEntry {
            id,
            scope,
            disabled,
            area,
            layer: self.layer,
            swallows_typing,
        });
        if f == Focusability::FocusableReadOnly {
            self.declare_state(id, StateFlags::READ_ONLY);
        }
    }

    pub fn register_control(&mut self, id: Id, area: Rect, f: Focusability) {
        self.register_entry(id, area, f, false, true);
    }

    pub fn register_focus_only(&mut self, id: Id, f: Focusability) {
        if self.registrations_suppressed() {
            return;
        }
        self.register_focus_entry(id, Rect::ZERO, f, false);
    }

    pub fn register_editor(&mut self, id: Id, area: Rect, f: Focusability, flags: StateFlags) {
        self.register_entry(id, area, f, true, true);
        self.declare_state(id, flags);
    }

    pub fn register_keyboard_editor(
        &mut self,
        id: Id,
        area: Rect,
        f: Focusability,
        flags: StateFlags,
    ) {
        self.register_entry(id, area, f, true, false);
        self.declare_state(id, flags);
    }

    pub fn declare_state(&mut self, id: Id, flags: StateFlags) {
        if self.reference.is_some() {
            return;
        }

        let flags = flags | self.context_state();
        if flags.is_empty() {
            return;
        }
        if let Some((_, f)) = self.frame.declared.iter_mut().find(|(i, _)| *i == id) {
            *f |= flags;
        } else {
            self.frame.declared.push((id, flags));
        }
    }

    pub fn publish_bindings<C: Copy + 'static>(
        &mut self,
        owner: Id,
        flags: StateFlags,
        table: &'static [Binding<C>],
    ) {
        self.publish_bindings_filtered(owner, flags, table, |_| true);
    }

    pub(crate) fn publish_scope_bindings<C: Copy + 'static>(
        &mut self,
        scope: ScopeId,
        owner: Id,
        flags: StateFlags,
        table: &'static [Binding<C>],
    ) {
        if self.registrations_suppressed() || self.frame.ring.scope_layer(scope) != Some(self.layer)
        {
            return;
        }
        if let Some(action) =
            self.frame
                .bindings
                .publish_filtered(owner, flags, self.layer, table, |_| true)
        {
            self.frame
                .diagnostics
                .push(Diagnostic::DuplicateBindingAction { owner, action });
            return;
        }
        self.frame.scoped_bubble_bindings.push(ScopedBubbleBinding {
            scope,
            owner,
            layer: self.layer,
        });
    }

    pub(crate) fn publish_bindings_filtered<C: Copy + 'static>(
        &mut self,
        owner: Id,
        flags: StateFlags,
        table: &'static [Binding<C>],
        include: impl Fn(C) -> bool,
    ) {
        let initial_focus = self.last.snapshot.focus.is_none()
            && self.last.ring.entries().is_empty()
            && self.frame.ring.reachable().next().map(|entry| entry.id) == Some(owner);
        if self.registrations_suppressed()
            || (self.last.snapshot.focus != Some(owner) && !initial_focus)
            || (!flags.contains(StateFlags::FOCUSED) && !initial_focus)
            || !self.frame.ring.contains(owner)
        {
            return;
        }
        let flags = if initial_focus {
            flags | StateFlags::FOCUSED
        } else {
            flags
        };
        if let Some(action) = self
            .frame
            .bindings
            .publish_filtered(owner, flags, self.layer, table, include)
        {
            self.frame
                .diagnostics
                .push(Diagnostic::DuplicateBindingAction { owner, action });
        }
    }

    pub(crate) fn extend_bindings<C: Copy + 'static>(
        &mut self,
        owner: Id,
        flags: StateFlags,
        table: &'static [Binding<C>],
        include: impl Fn(C) -> bool,
    ) -> Result<(), BindingExtensionError> {
        let initial_focus = self.last.snapshot.focus.is_none()
            && self.last.ring.entries().is_empty()
            && self.frame.ring.reachable().next().map(|entry| entry.id) == Some(owner);
        if self.registrations_suppressed()
            || (self.last.snapshot.focus != Some(owner) && !initial_focus)
            || (!flags.contains(StateFlags::FOCUSED) && !initial_focus)
            || !self.frame.ring.contains(owner)
        {
            return Ok(());
        }
        let flags = if initial_focus {
            flags | StateFlags::FOCUSED
        } else {
            flags
        };
        match self
            .frame
            .bindings
            .extend(owner, flags, self.layer, table, include)
        {
            Ok(()) => Ok(()),
            Err(error @ BindingExtensionError::MissingBase { .. }) => Err(error),
            Err(error @ BindingExtensionError::DuplicateAction { action, .. }) => {
                self.frame
                    .diagnostics
                    .push(Diagnostic::DuplicateBindingAction { owner, action });
                Err(error)
            }
            Err(error @ BindingExtensionError::ConflictingChord { chord, .. }) => {
                self.frame.diagnostics.push(Diagnostic::BindingConflict {
                    chord,
                    phase: crate::keymap::KeyPhase::Bubble,
                    a: owner,
                    b: owner,
                });
                Err(error)
            }
        }
    }

    pub fn publish_typing_target<C: Copy + 'static>(
        &mut self,
        owner: Id,
        table: &'static [Binding<C>],
        include: impl Fn(C) -> bool,
        cursor: bool,
    ) {
        if self.registrations_suppressed() {
            return;
        }
        self.frame
            .typing
            .push(crate::runtime::typing::TypingDeclaration {
                owner,
                layer: self.layer,
                cursor,
            });
        if let Some(action) = self.frame.typing_bindings.publish_filtered(
            owner,
            StateFlags::EDITING,
            self.layer,
            table,
            include,
        ) {
            self.frame
                .diagnostics
                .push(Diagnostic::DuplicateBindingAction { owner, action });
        }
    }

    pub fn publish_focused_paste_target(&mut self, owner: Id) {
        let registered_here = self
            .frame
            .ring
            .entry(owner)
            .is_some_and(|entry| entry.layer == self.layer && !entry.area.is_empty())
            && self.frame.registry.layer_of(owner) == Some(self.layer)
            && self
                .frame
                .registry
                .area_of(owner)
                .is_some_and(|area| !area.is_empty());
        if self.registrations_suppressed() || !registered_here {
            return;
        }
        if !self
            .frame
            .focused_paste_targets
            .iter()
            .any(|declaration| declaration.owner == owner && declaration.layer == self.layer)
        {
            self.frame.focused_paste_targets.push(
                crate::runtime::typing::FocusedPasteDeclaration {
                    owner,
                    layer: self.layer,
                },
            );
        }
    }

    pub fn publish_dynamic_bindings<I>(&mut self, owner: Id, flags: StateFlags, bindings: I)
    where
        I: Iterator<Item = (ActionKey, Option<crate::event::Chord>)> + Clone,
    {
        self.publish_additional_binding_descriptors(
            owner,
            flags,
            bindings.map(|(action, chord)| BindingDescriptor {
                action,
                chord,
                label: "",
                priority: 0,
                visible: false,
            }),
        );
    }

    pub fn publish_additional_bindings<C: Copy + 'static>(
        &mut self,
        owner: Id,
        flags: StateFlags,
        table: &'static [Binding<C>],
    ) {
        self.publish_additional_binding_descriptors(
            owner,
            flags,
            table.iter().map(|binding| BindingDescriptor {
                action: binding.action,
                chord: binding.chord,
                label: binding.label,
                priority: binding.priority,
                visible: binding.visible,
            }),
        );
    }

    fn publish_additional_binding_descriptors<I>(
        &mut self,
        owner: Id,
        flags: StateFlags,
        bindings: I,
    ) where
        I: Iterator<Item = BindingDescriptor> + Clone,
    {
        let initial_focus = self.last.snapshot.focus.is_none()
            && self.last.ring.entries().is_empty()
            && self.frame.ring.reachable().next().map(|entry| entry.id) == Some(owner);
        if self.registrations_suppressed()
            || (self.last.snapshot.focus != Some(owner) && !initial_focus)
            || (!flags.contains(StateFlags::FOCUSED) && !initial_focus)
            || !self.frame.ring.contains(owner)
        {
            return;
        }
        let flags = if initial_focus {
            flags | StateFlags::FOCUSED
        } else {
            flags
        };
        let (base, base_descriptors) = self
            .frame
            .bindings
            .get(owner)
            .map_or((None, &[][..]), |(published, descriptors)| {
                (Some(published.table), descriptors)
            });
        let (descriptors, revision) =
            self.core
                .dynamic_bindings
                .update_descriptors(owner, base, base_descriptors, bindings);
        if let Some(action) =
            self.frame
                .bindings
                .publish_dynamic(owner, flags, self.layer, descriptors, revision)
        {
            self.frame
                .diagnostics
                .push(Diagnostic::DuplicateBindingAction { owner, action });
        }
    }

    pub fn effective_chord(
        &self,
        owner: Id,
        action: ActionKey,
        default: Option<crate::event::Chord>,
    ) -> Option<crate::event::Chord> {
        self.core.keymap.component_chord(owner, action, default)
    }

    pub fn with_focused_hints<R>(
        &mut self,
        f: impl FnOnce(&mut Ui<'_>, &HintLayer) -> R,
    ) -> Option<R> {
        if self.reference.is_some() {
            return None;
        }
        let focus = self.last.snapshot.focus?;
        let (published, table) = self.frame.bindings.get(focus)?;
        let key = FocusedHintKey {
            focus,
            flags: published.flags,
            layer: self.frame.top,
            table: published.table,
            keymap_revision: self.core.keymap_revision,
        };
        let mut cache = core::mem::take(&mut self.core.focused_hints);
        cache.derive(key, table, &self.core.keymap);
        let result = f(self, &cache.layer);
        self.core.focused_hints = cache;
        Some(result)
    }

    pub fn register_part(&mut self, owner: Id, part: PartRef, area: Rect) {
        if self.registrations_suppressed() {
            return;
        }
        self.frame
            .registry
            .register_part(owner, part, area.intersection(self.clip), self.layer);
    }

    pub fn register_text_drag_autoscroll(
        &mut self,
        owner: Id,
        part: PartRef,
        area: Rect,
        axes: Axes,
        headroom: Headroom,
        source_revision: u32,
    ) {
        if self.registrations_suppressed() {
            return;
        }
        self.frame
            .registry
            .register_text_drag_autoscroll(TextDragRegistration {
                owner,
                part,
                area: area.intersection(self.clip),
                layer: self.layer,
                axes,
                headroom,
                source_revision,
            });
    }

    pub fn register_hover_part(&mut self, owner: Id, part: PartRef, area: Rect) {
        if self.registrations_suppressed() {
            return;
        }
        self.frame.registry.register_hover_part(
            owner,
            part,
            area.intersection(self.clip),
            self.layer,
        );
    }

    pub fn register_decor(&mut self, owner: Id, part: PartRef, area: Rect) {
        if self.registrations_suppressed() {
            return;
        }
        self.frame
            .registry
            .register_decor(owner, part, area.intersection(self.clip), self.layer);
    }

    pub fn register_scroll(&mut self, id: Id, area: Rect, axes: Axes, head: Headroom) {
        if self.registrations_suppressed() {
            return;
        }
        self.frame.registry.register_scroll(
            id,
            area.intersection(self.clip),
            self.layer,
            axes,
            head,
        );
    }

    pub fn register_routed_scroll(
        &mut self,
        id: Id,
        area: Rect,
        axes: Axes,
        head: Headroom,
        source_revision: u64,
    ) -> Option<ScrollRouteKey> {
        if self.registrations_suppressed() {
            return None;
        }
        self.frame.registry.register_routed_scroll(
            id,
            area.intersection(self.clip),
            self.layer,
            axes,
            head,
            source_revision,
        )
    }

    pub fn register_hit_stack_wheel(
        &mut self,
        id: Id,
        part: PartRef,
        area: Rect,
        axes: Axes,
        source_revision: u64,
    ) -> Option<ScrollRouteKey> {
        if self.registrations_suppressed() {
            return None;
        }
        self.frame.registry.register_hit_stack_wheel(
            id,
            part,
            area.intersection(self.clip),
            self.layer,
            axes,
            source_revision,
        )
    }

    pub fn register_event_local_hit_stack_wheel(
        &mut self,
        id: Id,
        part: PartRef,
        area: Rect,
        axes: Axes,
        source_revision: u64,
    ) -> Option<ScrollRouteKey> {
        if self.registrations_suppressed() {
            return None;
        }
        self.frame.registry.register_event_local_hit_stack_wheel(
            id,
            part,
            area.intersection(self.clip),
            self.layer,
            axes,
            source_revision,
        )
    }

    pub fn link_scroll_parent(&mut self, child: ScrollRouteKey, parent: ScrollRouteKey) -> bool {
        !self.registrations_suppressed() && self.frame.registry.link_scroll_parent(child, parent)
    }

    pub fn register_passive_reveal_scroll(&mut self, id: Id, area: Rect, head: Headroom) {
        if self.registrations_suppressed() {
            return;
        }
        self.frame.registry.register_passive_reveal_scroll(
            id,
            area.intersection(self.clip),
            self.layer,
            head,
        );
    }

    pub fn report_layout(&mut self, id: Id, l: LayoutFacts) {
        if self.reference.is_some() {
            return;
        }
        self.frame.layout.push((id, l));
    }

    pub fn report_horizontal_source_offset(&mut self, owner: Id, source_column: usize) -> bool {
        if self.reference.is_some() {
            return false;
        }
        let Some((_, facts)) = self
            .frame
            .layout
            .iter_mut()
            .rev()
            .find(|(id, _)| *id == owner)
        else {
            return false;
        };
        facts.horizontal_source_offset = Some(source_column);
        true
    }

    pub fn set_cursor(&mut self, owner: Id, pos: Position) {
        self.record_cursor(owner, pos, false, crate::cursor::CursorShape::Block);
    }

    pub fn set_bar_cursor(&mut self, owner: Id, pos: Position) {
        self.record_cursor(owner, pos, false, crate::cursor::CursorShape::Bar);
    }

    pub fn retain_owner_cursor(&mut self, owner: Id) -> bool {
        self.retain_cursor_request(owner, CursorRetentionMode::FocusedOwner)
    }

    pub fn retain_visible_owner_cursor(&mut self, owner: Id) -> bool {
        self.retain_cursor_request(owner, CursorRetentionMode::VisibleOwner)
    }

    fn retain_cursor_request(&mut self, owner: Id, mode: CursorRetentionMode) -> bool {
        if self.reference.is_some() || self.inert || self.layer != self.frame.top {
            return false;
        }
        if mode == CursorRetentionMode::FocusedOwner && self.last.snapshot.focus != Some(owner) {
            return false;
        }
        let Some((top_index, top)) = self.frame.layers.active().iter().enumerate().next_back()
        else {
            return false;
        };
        if top.layer != self.layer
            || !top.drawn
            || top.spec.kind != LayerKind::Popover
            || top.spec.inert_below
        {
            return false;
        }
        if let Some(existing) = self.frame.retained_cursor_request.as_mut() {
            if existing.owner == owner && existing.mode == mode && !existing.ambiguous {
                return true;
            }
            if !existing.ambiguous {
                self.frame
                    .diagnostics
                    .push(crate::diagnostics::Diagnostic::CursorRejected {
                        owner,
                        layer: self.layer,
                    });
                existing.ambiguous = true;
            }
            return false;
        }
        let parent_layer = top_index
            .checked_sub(1)
            .and_then(|parent_index| self.frame.layers.active().get(parent_index))
            .map_or(LayerId::PAGE, |parent| parent.layer);
        if parent_layer < self.frame.inert_floor
            || self.frame.registry.layer_of(owner) != Some(parent_layer)
        {
            return false;
        }
        let Some(area) = self.frame.registry.area_of(owner) else {
            return false;
        };
        let Some((cursor_index, cursor)) =
            self.frame
                .cursors
                .iter_mut()
                .enumerate()
                .rev()
                .find(|(_, cursor)| {
                    cursor.owner == owner
                        && cursor.layer == parent_layer
                        && !cursor.inert
                        && !cursor.typing_offer
                        && self.frame.screen.contains(cursor.pos)
                        && area.contains(cursor.pos)
                })
        else {
            return false;
        };
        cursor.layer = self.layer;
        self.frame.retained_cursor_request = Some(RetainedCursorRequest {
            owner,
            parent_layer,
            overlay_layer: self.layer,
            cursor_index,
            mode,
            ambiguous: false,
        });
        true
    }

    pub fn set_undrawn_top_bar_cursor(&mut self, owner: Id, area: Rect, pos: Position) {
        if self.reference.is_some() || area.is_empty() {
            return;
        }
        let saved_layer = self.layer;
        let saved_inert = self.inert;
        self.layer = self.frame.top;
        self.inert = false;
        self.register_keyboard_editor(owner, area, Focusability::Focusable, StateFlags::EDITING);
        self.set_bar_cursor(owner, pos);
        self.layer = saved_layer;
        self.inert = saved_inert;
    }

    pub fn offer_typing_cursor(&mut self, owner: Id, pos: Position) {
        self.record_cursor(owner, pos, true, crate::cursor::CursorShape::Block);
    }

    fn record_cursor(
        &mut self,
        owner: Id,
        pos: Position,
        typing_offer: bool,
        shape: crate::cursor::CursorShape,
    ) {
        let in_clip = self.clip.contains(pos)
            || (self.layer > LayerId::PAGE
                && self.layer == self.frame.top
                && self.frame.screen.contains(pos));
        if self.reference.is_some() || !in_clip {
            return;
        }
        self.frame.cursors.push(CursorRequest {
            layer: self.layer,
            owner,
            pos,
            inert: self.inert,
            typing_offer,
            shape,
        });
    }

    pub fn layer_for<R>(
        &mut self,
        opening: LayerOpening,
        f: impl FnOnce(&mut Ui<'_>, Rect) -> R,
    ) -> Option<R> {
        if self.reference.is_some() {
            return None;
        }
        let index = self.frame.layers.find(opening.id())?;
        let layer_matches = self
            .frame
            .layers
            .active()
            .get(index)
            .is_some_and(|draw| draw.layer == opening.layer_id());
        if !layer_matches || !self.frame.layer_openings.contains(&opening) {
            return None;
        }
        self.layer_with_opening(opening.id(), Some(opening), f)
    }

    pub fn layer<R>(&mut self, id: Id, f: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> Option<R> {
        self.layer_with_opening(id, None, f)
    }

    fn layer_with_opening<R>(
        &mut self,
        id: Id,
        opening: Option<LayerOpening>,
        f: impl FnOnce(&mut Ui<'_>, Rect) -> R,
    ) -> Option<R> {
        if self.reference.is_some() {
            return None;
        }
        let managed = self.frame.managed_tooltip_opening;
        if managed.is_some_and(|managed| managed.id() == id) && opening != managed {
            return None;
        }
        let idx = self.frame.layers.find(id)?;
        let (layer, area, kind) = {
            let d = self.frame.layers.active_mut().get_mut(idx)?;
            if d.drawn {
                self.frame
                    .diagnostics
                    .push(Diagnostic::DuplicateLayerDraw { id });
                return None;
            }
            d.drawn = true;
            (d.layer, d.area, d.spec.kind)
        };
        if managed.is_some() && opening == managed {
            self.frame.managed_tooltip_drawn = true;
        }
        let inert = layer < self.frame.inert_floor;
        let surface = match kind {
            LayerKind::Modal => Surface::Overlay,
            LayerKind::Popover | LayerKind::Tooltip => Surface::Popover,
        };
        let mode = match kind {
            LayerKind::Modal => ScopeMode::Trap,
            LayerKind::Popover | LayerKind::Tooltip => ScopeMode::Normal,
        };

        let parent = idx
            .checked_sub(1)
            .and_then(|parent| self.frame.layers.active().get(parent))
            .map(|candidate| ScopeId::new(candidate.id));
        self.frame
            .ring
            .ensure_scope(ScopeId::new(id), mode, layer, parent);
        self.frame.ring.push_scope(ScopeId::new(id), mode, layer);

        let guard = FocusScopeGuard { ui: self };
        let mut nested = guard.ui.reborrow();
        nested.target = Target::Layer(idx);
        nested.clip = area;
        nested.layer = layer;
        nested.inert = inert;
        nested.surface = surface;
        Some(f(&mut nested, area))
    }

    pub fn cache<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        if let Some(scope) = self.reference {
            self.core
                .reference_cache
                .get_mut_reference::<T>(id, scope.cache_key)
        } else {
            self.core.cache.get_mut::<T>(id)
        }
    }

    pub(crate) fn buffer_in(&mut self, area: Rect) -> (&mut Buffer, Rect) {
        let a = area.intersection(self.clip);
        self.mark_area(a, None);
        (self.buffer(), a)
    }

    pub(crate) fn buffer(&mut self) -> &mut Buffer {
        match self.target {
            Target::Page => self.page,
            Target::Layer(i) => match self.frame.layers.active_mut().get_mut(i) {
                Some(d) => &mut d.buf,
                None => self.page,
            },
        }
    }

    pub(crate) fn move_cell(
        &mut self,
        src: Position,
        dst: Position,
        reset: crate::theme::PaintStyle,
    ) {
        if !self.clip.contains(src) || !self.clip.contains(dst) {
            return;
        }
        let (roles, underlay) = match self.target {
            Target::Page => (self.roles_at(src), None),
            Target::Layer(i) => self.frame.layers.active().get(i).map_or_else(
                || (CellRoles::default(), None),
                |d| (d.roles_at(src), d.underlay_at(src).cloned()),
            ),
        };
        if let Some(cell) = self.buffer().cell(src).cloned() {
            if let Some(target) = self.buffer().cell_mut(dst) {
                *target = cell;
            }
            self.mark(dst, None);
            match self.target {
                Target::Page => {
                    if let Some(i) = self.frame.role_index(dst)
                        && let Some(target) = self.frame.roles.get_mut(i)
                    {
                        *target = roles;
                    }
                }
                Target::Layer(i) => {
                    if let Some(d) = self.frame.layers.active_mut().get_mut(i) {
                        d.set_roles(dst, roles);
                        d.set_underlay(dst, underlay);
                    }
                }
            }
            if let Some(cell) = self.buffer().cell_mut(src) {
                let kept_fg = cell.fg;
                let kept_bg = cell.bg;
                cell.reset();
                let style = reset.into_style();
                cell.set_style(style);
                if style.fg.is_none() {
                    cell.fg = kept_fg;
                }
                if style.bg.is_none() {
                    cell.bg = kept_bg;
                }
            }

            self.mark(src, Some(reset));
            if let Target::Layer(i) = self.target
                && let Some(d) = self.frame.layers.active_mut().get_mut(i)
            {
                d.reset_source_underlay(src, reset);
            }
        }
    }

    pub(crate) fn mark(&mut self, pos: Position, style: Option<crate::theme::PaintStyle>) {
        match self.target {
            Target::Page => {
                if let Some(i) = self.frame.role_index(pos)
                    && let Some(r) = self.frame.roles.get_mut(i)
                {
                    *r = r.patch(style);
                }
            }
            Target::Layer(i) => {
                if style.is_none() {
                    let page_cell = self.page.cell(pos).cloned();
                    let page_roles = self.roles_at(pos);
                    if let Some(page_cell) = page_cell
                        && let Some(d) = self.frame.layers.active_mut().get_mut(i)
                    {
                        d.materialize_underlay(pos, &page_cell, page_roles);
                    }
                }
                if let Some(d) = self.frame.layers.active_mut().get_mut(i) {
                    d.mark(pos, style);
                }
            }
        }
    }

    pub(crate) fn cell_written(&self, pos: Position) -> bool {
        match self.target {
            Target::Page => true,
            Target::Layer(i) => self
                .frame
                .layers
                .active()
                .get(i)
                .is_some_and(|layer| layer.is_written(pos)),
        }
    }

    pub(crate) fn mark_area(&mut self, area: Rect, style: Option<crate::theme::PaintStyle>) {
        let area = area.intersection(self.clip);
        match self.target {
            Target::Page => {
                for pos in area.positions() {
                    self.mark(pos, style);
                }
            }
            Target::Layer(i) => {
                if let Some(d) = self.frame.layers.active_mut().get_mut(i) {
                    d.mark_area(area, style);
                }
            }
        }
    }

    pub const fn theme_ref(&self) -> &'f Theme {
        self.theme
    }

    pub(crate) fn roles_at(&self, pos: Position) -> CellRoles {
        self.frame
            .role_index(pos)
            .and_then(|i| self.frame.roles.get(i))
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn page_mut(&mut self) -> &mut Buffer {
        self.page
    }

    pub fn fill_card_from_page(&mut self, area: Rect, card: ratatui_core::style::Color) {
        let theme = self.theme.clone();
        let samples: Vec<_> = area
            .positions()
            .map(|pos| {
                let (fg, bg) = self
                    .page
                    .cell(pos)
                    .map(|cell| (cell.fg, cell.bg))
                    .unwrap_or((
                        ratatui_core::style::Color::Reset,
                        ratatui_core::style::Color::Reset,
                    ));
                let (fg, _) = theme.backdrop_colors(fg, bg);
                (pos, fg)
            })
            .collect();
        {
            let buf = self.buffer();
            for (pos, fg) in samples {
                if let Some(cell) = buf.cell_mut(pos) {
                    cell.set_symbol(" ");
                    let mut style = ratatui_core::style::Style::new();
                    style.fg = Some(fg);
                    style.bg = Some(card);
                    cell.set_style(style);
                    cell.modifier = ratatui_core::style::Modifier::empty();
                }
            }
        }
        self.mark_area(area, None);
    }

    pub fn fill_keep_fg(&mut self, area: Rect, bg: ratatui_core::style::Color) {
        self.fill(area, crate::theme::PaintStyle::new().bg(bg));
    }

    pub fn recolor_rgb(
        &mut self,
        area: Rect,
        mut map: impl FnMut(ratatui_core::style::Color) -> ratatui_core::style::Color,
    ) {
        self.recolor_cells(area, |fg, bg| {
            let fg = if matches!(fg, ratatui_core::style::Color::Rgb(..)) {
                map(fg)
            } else {
                fg
            };
            let bg = if matches!(bg, ratatui_core::style::Color::Rgb(..)) {
                map(bg)
            } else {
                bg
            };
            (fg, bg)
        });
    }

    pub fn recolor_pair(
        &mut self,
        area: Rect,
        map: impl FnMut(
            ratatui_core::style::Color,
            ratatui_core::style::Color,
        ) -> (ratatui_core::style::Color, ratatui_core::style::Color),
    ) {
        self.recolor_cells(area, map);
    }

    pub fn recolor_fg(
        &mut self,
        area: Rect,
        mut map: impl FnMut(
            ratatui_core::style::Color,
            ratatui_core::style::Color,
        ) -> Option<ratatui_core::style::Color>,
    ) {
        self.recolor_cells(area, |fg, bg| (map(fg, bg).unwrap_or(fg), bg));
    }

    fn recolor_cells(
        &mut self,
        area: Rect,
        mut map: impl FnMut(
            ratatui_core::style::Color,
            ratatui_core::style::Color,
        ) -> (ratatui_core::style::Color, ratatui_core::style::Color),
    ) {
        let area = area.intersection(self.clip);
        for pos in area.positions() {
            let Some((old_fg, old_bg)) = self.buffer().cell(pos).map(|cell| (cell.fg, cell.bg))
            else {
                continue;
            };
            let (fg, bg) = map(old_fg, old_bg);
            let fg_changed = fg != old_fg;
            let bg_changed = bg != old_bg;
            if !fg_changed && !bg_changed {
                continue;
            }

            {
                let Some(cell) = self.buffer().cell_mut(pos) else {
                    continue;
                };
                if fg_changed {
                    cell.set_fg(fg);
                }
                if bg_changed {
                    cell.set_bg(bg);
                }
            }

            let mut patch = crate::theme::PaintStyle::new();
            if fg_changed {
                patch = patch.fg(fg);
            }
            if bg_changed {
                patch = patch.bg(bg);
            }
            self.mark(pos, Some(patch));
        }
    }

    pub(crate) fn layer_count(&self) -> usize {
        self.frame.layers.active().len()
    }

    pub(crate) fn layer_meta(&self, i: usize) -> Option<(crate::layer::Backdrop, bool)> {
        self.frame
            .layers
            .active()
            .get(i)
            .map(|d| (d.spec.backdrop, d.drawn))
    }

    pub(crate) fn composite(&mut self, i: usize) {
        if let Some(d) = self.frame.layers.active().get(i) {
            let content_dim = matches!(d.spec.backdrop, crate::layer::Backdrop::Dim { .. })
                && matches!(
                    self.theme_ref().capability.color,
                    crate::theme::ColorLevel::Ansi16 | crate::theme::ColorLevel::Mono
                );
            d.composite_onto(self.page, &mut self.frame.roles, content_dim);
        }
    }
}

impl FrameRead for Ui<'_> {
    fn state(&self, id: Id) -> StateFlags {
        self.reference_state(id)
            .unwrap_or_else(|| self.last.state(id))
    }

    fn hovered_part(&self, owner: Id) -> Option<PartRef> {
        if self.reference.is_some() {
            self.reference_part(owner, ReferenceState::HOVERED)
        } else {
            self.last.hovered_part(owner)
        }
    }

    fn pressed_part(&self, owner: Id) -> Option<PartRef> {
        if self.reference.is_some() {
            self.reference_part(owner, ReferenceState::PRESSED)
        } else {
            self.last.pressed_part(owner)
        }
    }

    fn theme(&self) -> &Theme {
        self.theme
    }

    fn design(&self) -> &DesignTokens {
        &self.theme.design
    }

    fn area(&self, id: Id) -> Option<Rect> {
        if self.reference.is_some() {
            None
        } else {
            self.last.registry.area_of(id)
        }
    }

    fn area_of_part(&self, owner: Id, part: PartRef) -> Option<Rect> {
        if self.reference.is_some() {
            None
        } else {
            self.last.registry.area_of_part(owner, part)
        }
    }

    fn layout(&self, id: Id) -> Option<LayoutFacts> {
        if self.reference.is_some() {
            None
        } else {
            self.last.layout_of(id)
        }
    }
}
