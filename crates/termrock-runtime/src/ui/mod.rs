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
use crate::hit::{Axes, Headroom, Registry};
use crate::id::{Id, Part, PartRef};
use crate::keymap::{
    Binding, BindingDescriptor, BindingExtensionError, BindingRegistry, DynamicBindingRegistry,
    FocusedHintKey, FocusedHints, HintLayer, KeyMap, ScopedBubbleBinding,
};
use crate::layer::{LayerId, LayerKind};
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
    pub(crate) cursors: Vec<CursorRequest>,
    pub(crate) retained_cursor_request: Option<RetainedCursorRequest>,
    pub(crate) typing: Vec<crate::runtime::typing::TypingDeclaration>,
    pub(crate) typing_bindings: BindingRegistry,
    pub(crate) typing_resolved: crate::runtime::typing::TypingResolved,
    pub(crate) layout: Vec<(Id, LayoutFacts)>,
    pub(crate) declared: Vec<(Id, StateFlags)>,
    pub(crate) bindings: BindingRegistry,
    pub(crate) scoped_bubble_bindings: Vec<ScopedBubbleBinding>,
    pub(crate) diagnostics: Diagnostics,
    pub(crate) roles: Vec<CellRoles>,
    pub(crate) screen: Rect,
    pub(crate) inert_floor: LayerId,
    pub(crate) top: LayerId,
    #[cfg(feature = "testing")]
    pub(crate) styled_parts: Vec<(Id, Part)>,
    #[cfg(feature = "testing")]
    pub(crate) styled_queries: Vec<StyledQuery>,
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

#[cfg(feature = "testing")]
pub type StyledQuery = (Id, Family, Variant, Part, Resolved);

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
        self.cursors.clear();
        self.retained_cursor_request = None;
        self.typing.clear();
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
        #[cfg(feature = "testing")]
        self.styled_parts.clear();
        #[cfg(feature = "testing")]
        self.styled_queries.clear();
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

    pub const fn is_inert(&self) -> bool {
        self.inert || self.reference.is_some()
    }

    /// Append a component-author diagnostic to this frame's bounded runtime
    /// diagnostic stream. Diagnostics are committed with the frame and cannot
    /// be cleared by an individual component.
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

    #[cfg(feature = "testing")]
    pub fn note_styled(
        &mut self,
        owner: Id,
        family: Family,
        variant: Variant,
        part: Part,
        resolved: Resolved,
    ) {
        self.frame.styled_parts.push((owner, part));
        self.frame
            .styled_queries
            .push((owner, family, variant, part, resolved));
    }

    #[cfg(feature = "testing")]
    pub fn styled_parts(&self) -> &[(Id, Part)] {
        &self.frame.styled_parts
    }

    #[cfg(feature = "testing")]
    pub fn styled_queries(&self) -> &[StyledQuery] {
        &self.frame.styled_queries
    }

    #[cfg(feature = "testing")]
    pub fn style_cache_stats(&self) -> (u64, u64) {
        self.core.style_cache.stats()
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
        if self.reference.is_some() || flags.is_empty() {
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

    /// Add bindings to the focused owner's current table while preserving hint metadata.
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

    pub fn report_layout(&mut self, id: Id, l: LayoutFacts) {
        if self.reference.is_some() {
            return;
        }
        self.frame.layout.push((id, l));
    }

    pub fn set_cursor(&mut self, owner: Id, pos: Position) {
        self.record_cursor(owner, pos, false, crate::cursor::CursorShape::Block);
    }

    /// Same as [`Self::set_cursor`] with a steady bar. Presentation emits the
    /// shape. Callers do not write cursor bytes.
    pub fn set_bar_cursor(&mut self, owner: Id, pos: Position) {
        self.record_cursor(owner, pos, false, crate::cursor::CursorShape::Bar);
    }

    /// Retains an already-rendered focused owner's cursor through a nonmodal
    /// popover that explicitly calls this method. The cursor request is moved to the current
    /// overlay layer without registering a second control or cursor. Ordinary
    /// popovers, modal layers, inert layers, and missing or stale owner requests
    /// do not retain a cursor.
    pub fn retain_owner_cursor(&mut self, owner: Id) -> bool {
        self.retain_cursor_request(owner, CursorRetentionMode::FocusedOwner)
    }

    /// Retains a real rendered owner's cursor through the current nonmodal
    /// popover, even when that popover owns keyboard focus. This is intended
    /// for visible terminal or editor carets that remain meaningful while a
    /// menu takes input. It never registers a control or invents a cursor.
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

    /// Steady bar for `owner` on the open top layer. Does not draw that layer,
    /// so a page stamp is not dimmed or covered.
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

    pub fn layer<R>(&mut self, id: Id, f: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> Option<R> {
        if self.reference.is_some() {
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
        let roles = match self.target {
            Target::Page => self.roles_at(src),
            Target::Layer(i) => self
                .frame
                .layers
                .active()
                .get(i)
                .map_or_else(CellRoles::default, |d| d.roles_at(src)),
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
                    }
                }
            }
            if let Some(cell) = self.buffer().cell_mut(src) {
                // `reset` drops every channel. A partial style then leaves
                // fg/bg at Default, which punches a hole through the row fill
                // when a right-aligned value shifts off those cells.
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
            // Preserve the source cell's semantic role metadata for the
            // vacated padding cell, then apply any reset-channel overrides.
            // Clearing roles here makes later visual passes (such as dimming)
            // treat padding differently from the row it belongs to.
            self.mark(src, Some(reset));
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

    /// Card fill that keeps the dimmed page glyph color and sets the card background.
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

    /// Space fill that keeps the ink already on the page.
    ///
    /// Baseline `fill` sets the glyph to a space and the background, and leaves
    /// the foreground. Call it after `modal_backdrop` so an empty dialog cell
    /// keeps the dimmed color under it.
    pub fn fill_keep_fg(&mut self, area: Rect, bg: ratatui_core::style::Color) {
        self.fill(area, crate::theme::PaintStyle::new().bg(bg));
    }

    /// Map truecolor inks. Indexed and named colors stay.
    ///
    /// A channel that maps to its existing color keeps its semantic paint
    /// role. Changed channels lose only their own role; the other channel and
    /// all non-color cell attributes remain intact.
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

    /// Map a cell's foreground and background together.
    ///
    /// Unchanged output channels keep their semantic paint roles.
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

    /// Replace a cell's foreground. `map` returns the new ink, or `None` to keep it.
    ///
    /// Returning the existing foreground also keeps its semantic paint role;
    /// background provenance is preserved in either case.
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
            d.composite_onto(self.page, &mut self.frame.roles);
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

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::Style;

    use super::cx::LastFrame;
    use super::{
        CellRoles, CursorRetentionMode, FrameRead, FrameState, LayoutFacts, ReferenceState,
        ReferenceTarget, RetainedCursorRequest, Ui, UiCore,
    };
    use crate::action::ActionKey;
    use crate::event::{Chord, KeyCode};
    use crate::focus::Focusability;
    use crate::hit::{Axes, Headroom};
    use crate::id::{Id, ItemKey, Part, PartRef};
    use crate::keymap::Binding;
    use crate::layer::{LayerId, LayerSpec};
    use crate::response::StateFlags;
    use crate::rowui::RowUi;
    use crate::theme::{
        Family, FgStep, Overlay, OverlayRule, Role, StylePatch, Surface, Theme, Variant,
    };

    const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 24,
        height: 3,
    };
    const OWNER: Id = Id::root("ui.owner");
    const OTHER: Id = Id::root("ui.other");

    #[derive(Clone, Copy)]
    enum TestCmd {
        Activate,
    }

    const TEST_BINDINGS: &[Binding<TestCmd>] = &[Binding {
        action: ActionKey::CONFIRM,
        chord: Some(Chord::key(KeyCode::Enter)),
        cmd: TestCmd::Activate,
        label: "Activate",
        priority: 1,
        visible: true,
    }];

    #[derive(Clone, Copy)]
    enum TestNavCmd {
        Up,
    }

    const TEST_NAV_BINDINGS: &[Binding<TestNavCmd>] = &[Binding {
        action: ActionKey::custom("ui.nav.up"),
        chord: Some(Chord::key(KeyCode::Up)),
        cmd: TestNavCmd::Up,
        label: "Up",
        priority: 2,
        visible: true,
    }];

    fn with_ui<R>(theme: &Theme, f: impl FnOnce(&mut Ui<'_>) -> R) -> (R, Buffer) {
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let out = {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, theme, &last);
            f(&mut ui)
        };
        (out, page)
    }

    #[test]
    fn author_binding_extension_is_explicit_and_additive_for_same_owner() {
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            ui.register_focus_only(OWNER, Focusability::Focusable);
            ui.publish_bindings(OWNER, StateFlags::FOCUSED, TEST_BINDINGS);
            crate::author::extend_bindings(
                ui,
                OWNER,
                StateFlags::FOCUSED,
                TEST_NAV_BINDINGS,
                |command| matches!(command, TestNavCmd::Up),
            )
            .expect("disjoint nav action extends the owner table");

            let (_, descriptors) = ui.frame.bindings.get(OWNER).expect("published owner");
            assert_eq!(descriptors.len(), 2);
            assert!(
                descriptors
                    .iter()
                    .any(|binding| binding.action == ActionKey::CONFIRM)
            );
            assert!(
                descriptors
                    .iter()
                    .any(|binding| binding.action == ActionKey::custom("ui.nav.up"))
            );
        });
    }

    #[test]
    fn reference_state_has_exactly_the_four_runtime_bits() {
        const CONST_COMBINED: ReferenceState =
            ReferenceState::FOCUS_VISIBLE.union(ReferenceState::PRESSED);
        assert_eq!(
            CONST_COMBINED.state_flags(),
            StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE | StateFlags::PRESSED
        );

        let choices = [
            (ReferenceState::FOCUSED, StateFlags::FOCUSED),
            (ReferenceState::FOCUS_VISIBLE, StateFlags::FOCUS_VISIBLE),
            (ReferenceState::HOVERED, StateFlags::HOVERED),
            (ReferenceState::PRESSED, StateFlags::PRESSED),
        ];
        for mask in 0u8..16 {
            let mut reference = ReferenceState::default();
            let mut expected = StateFlags::empty();
            for (i, (reference_bit, state_bit)) in choices.iter().copied().enumerate() {
                if mask & (1 << i) != 0 {
                    reference |= reference_bit;
                    expected |= state_bit;
                }
            }
            if expected.contains(StateFlags::FOCUS_VISIBLE) {
                expected |= StateFlags::FOCUSED;
            }
            assert_eq!(reference.state_flags(), expected, "mask {mask:04b}");
        }
    }

    #[test]
    fn reference_injects_only_the_exact_target_and_hides_stale_live_state() {
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let part = PartRef::item(Part::MARKER, ItemKey::num(7));
        let stale_part = PartRef::of(Part::LABEL);
        let last = LastFrame {
            declared: vec![
                (OWNER, StateFlags::ERROR | StateFlags::EDITING),
                (OTHER, StateFlags::BUSY),
            ],
            snapshot: super::cx::Snapshot {
                focus: Some(OTHER),
                focus_visible: true,
                hover: Some((OTHER, stale_part)),
                hover_suppressed: false,
                pressed: Some((OWNER, stale_part)),
                capture: Some(OTHER),
            },
            ..LastFrame::default()
        };
        let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);

        let target = ReferenceTarget::new(
            OWNER,
            ReferenceState::FOCUSED
                | ReferenceState::FOCUS_VISIBLE
                | ReferenceState::HOVERED
                | ReferenceState::PRESSED,
        )
        .part(part);
        ui.reference(Some(target), |ui| {
            assert_eq!(
                ui.state(OWNER),
                StateFlags::FOCUSED
                    | StateFlags::FOCUS_VISIBLE
                    | StateFlags::HOVERED
                    | StateFlags::PRESSED
            );
            assert_eq!(ui.state(OTHER), StateFlags::empty());
            assert_eq!(ui.hovered_part(OWNER), Some(part));
            assert_eq!(ui.pressed_part(OWNER), Some(part));
            assert_eq!(ui.hovered_part(OTHER), None);
            assert_eq!(ui.pressed_part(OTHER), None);
        });
        ui.reference(None, |ui| {
            assert_eq!(ui.state(OWNER), StateFlags::empty());
            assert_eq!(ui.state(OTHER), StateFlags::empty());
            assert_eq!(ui.hovered_part(OWNER), None);
            assert_eq!(ui.pressed_part(OWNER), None);
        });
        ui.reference(
            Some(ReferenceTarget::new(OWNER, ReferenceState::HOVERED)),
            |ui| {
                assert_eq!(ui.state(OWNER), StateFlags::HOVERED);
                assert_eq!(
                    ui.hovered_part(OWNER),
                    Some(PartRef::of(Part::CONTAINER)),
                    "an owner target canonicalizes to its container part"
                );
            },
        );
        ui.reference(
            Some(ReferenceTarget::new(OWNER, ReferenceState::PRESSED)),
            |ui| {
                assert_eq!(ui.pressed_part(OWNER), Some(PartRef::of(Part::CONTAINER)));
            },
        );
    }

    #[test]
    fn nested_reference_restores_the_outer_target_even_after_a_panic() {
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            assert!(!ui.is_inert());
            let outer =
                ReferenceTarget::new(OWNER, ReferenceState::HOVERED).part(PartRef::of(Part::LABEL));
            let inner = ReferenceTarget::new(OTHER, ReferenceState::PRESSED)
                .part(PartRef::of(Part::MARKER));
            ui.reference(Some(outer), |ui| {
                assert!(ui.is_inert());
                assert_eq!(ui.state(OWNER), StateFlags::HOVERED);
                ui.reference(Some(inner), |ui| {
                    assert_eq!(ui.state(OWNER), StateFlags::empty());
                    assert_eq!(ui.state(OTHER), StateFlags::PRESSED);
                });
                assert_eq!(ui.state(OWNER), StateFlags::HOVERED);

                let unwind = catch_unwind(AssertUnwindSafe(|| {
                    ui.reference(Some(inner), |_| panic!("reference probe"));
                }));
                assert!(unwind.is_err());
                assert_eq!(ui.state(OWNER), StateFlags::HOVERED);
                assert_eq!(ui.hovered_part(OWNER), Some(PartRef::of(Part::LABEL)));
            });
            assert!(!ui.is_inert());
        });
    }

    #[test]
    fn reference_sinks_every_registration_and_runtime_output() {
        const LAYER: Id = Id::root("ui.layer");
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        frame
            .layers
            .push(LAYER, LayerId(1), LayerSpec::modal(LAYER), SCREEN, SCREEN);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let mut layer_ran = false;
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            ui.reference(None, |ui| {
                ui.register_control(OWNER, SCREEN, Focusability::Focusable);
                ui.register_focus_only(OTHER, Focusability::Focusable);
                ui.register_editor(OWNER, SCREEN, Focusability::Focusable, StateFlags::EDITING);
                ui.register_part(OWNER, PartRef::of(Part::LABEL), SCREEN);
                ui.register_decor(OWNER, PartRef::of(Part::HELP), SCREEN);
                ui.register_scroll(OWNER, SCREEN, Axes::Both, Headroom::default());
                ui.declare_state(OWNER, StateFlags::ERROR);
                ui.report_layout(OWNER, LayoutFacts::new(1, 2, 3, 4));
                ui.set_cursor(OWNER, Position::new(1, 1));
                ui.publish_bindings(OWNER, StateFlags::FOCUSED, TEST_BINDINGS);
                ui.publish_dynamic_bindings(
                    OWNER,
                    StateFlags::FOCUSED,
                    core::iter::once((ActionKey::SAVE, Some(Chord::key(KeyCode::Char('s'))))),
                );
                assert!(ui.layer(LAYER, |_, _| layer_ran = true).is_none());
                ui.focus_scope(OWNER, crate::focus::ScopeMode::Trap, |ui| {
                    ui.register_control(OWNER, SCREEN, Focusability::Focusable);
                });
            });
        }

        assert!(!layer_ran);
        assert!(!frame.registry.has_owner(OWNER));
        assert!(!frame.registry.has_owner(OTHER));
        assert!(frame.ring.entries().is_empty());
        assert!(frame.declared.is_empty());
        assert!(frame.layout.is_empty());
        assert!(frame.cursors.is_empty());
        assert!(frame.bindings.get(OWNER).is_none());
        assert!(frame.diagnostics.items().is_empty());
        assert!(
            frame
                .layers
                .active()
                .first()
                .is_some_and(|layer| !layer.drawn)
        );
    }

    fn with_cursor_popover(
        kind: crate::layer::LayerKind,
        inert_below: bool,
        owner_focused: bool,
        owner_area: Rect,
        cursor_pos: Option<Position>,
        run: impl FnOnce(&mut Ui<'_>) -> bool,
    ) -> (bool, FrameState) {
        const POPUP: Id = Id::root("ui.cursor-popup");
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        frame.top = LayerId(1);
        frame.inert_floor = if inert_below {
            LayerId(1)
        } else {
            LayerId::PAGE
        };
        let mut spec = if kind == crate::layer::LayerKind::Modal {
            LayerSpec::modal(POPUP)
        } else {
            LayerSpec::popover(
                POPUP,
                crate::layer::Anchor::Screen(crate::layer::ScreenAlign::Center),
            )
        };
        spec.inert_below = inert_below;
        frame.layers.push(POPUP, LayerId(1), spec, SCREEN, SCREEN);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let mut last = LastFrame::default();
        if owner_focused {
            last.snapshot.focus = Some(OWNER);
        }
        let moved = {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            ui.register_editor(
                OWNER,
                owner_area,
                Focusability::Focusable,
                StateFlags::EDITING,
            );
            if let Some(pos) = cursor_pos {
                ui.set_bar_cursor(OWNER, pos);
            }
            let mut moved = false;
            let _ = ui.layer(POPUP, |ui, _| moved = run(ui));
            moved
        };
        (moved, frame)
    }

    #[test]
    fn focused_owner_cursor_can_be_retained_only_by_an_opted_in_nonmodal_popover() {
        use crate::layer::LayerKind;

        let (retained, frame) = with_cursor_popover(
            LayerKind::Popover,
            false,
            true,
            SCREEN,
            Some(Position::new(4, 1)),
            |ui| ui.retain_owner_cursor(OWNER),
        );
        assert!(retained);
        assert_eq!(
            frame.retained_cursor_request,
            Some(RetainedCursorRequest {
                owner: OWNER,
                parent_layer: LayerId::PAGE,
                overlay_layer: LayerId(1),
                cursor_index: 0,
                mode: CursorRetentionMode::FocusedOwner,
                ambiguous: false,
            })
        );
        assert_eq!(frame.cursors.len(), 1, "the real cursor request is reused");
        assert_eq!(frame.cursors[0].layer, LayerId(1));
        assert_eq!(frame.cursors[0].pos, Position::new(4, 1));
        assert_eq!(frame.cursors[0].shape, crate::cursor::CursorShape::Bar);

        let (ordinary, frame) = with_cursor_popover(
            LayerKind::Popover,
            false,
            true,
            SCREEN,
            Some(Position::new(4, 1)),
            |_| false,
        );
        assert!(!ordinary, "ordinary popovers do not retain owner cursors");
        assert_eq!(frame.retained_cursor_request, None);
        assert_eq!(frame.cursors[0].layer, LayerId::PAGE);

        for (kind, inert, focused, area, cursor) in [
            (
                LayerKind::Modal,
                false,
                true,
                SCREEN,
                Some(Position::new(4, 1)),
            ),
            (
                LayerKind::Popover,
                true,
                true,
                SCREEN,
                Some(Position::new(4, 1)),
            ),
            (
                LayerKind::Popover,
                false,
                false,
                SCREEN,
                Some(Position::new(4, 1)),
            ),
            (LayerKind::Popover, false, true, SCREEN, None),
            (
                LayerKind::Popover,
                false,
                true,
                Rect::new(0, 0, 2, 2),
                Some(Position::new(4, 1)),
            ),
        ] {
            let (retained, frame) = with_cursor_popover(kind, inert, focused, area, cursor, |ui| {
                ui.retain_owner_cursor(OWNER)
            });
            assert!(!retained, "invalid cursor retention must be rejected");
            assert_eq!(frame.retained_cursor_request, None);
            assert!(
                frame
                    .cursors
                    .iter()
                    .all(|cursor| cursor.layer == LayerId::PAGE),
                "rejected retention leaves the original cursor scope intact"
            );
        }

        let (retained, frame) = with_cursor_popover(
            LayerKind::Popover,
            false,
            true,
            SCREEN,
            Some(Position::new(4, 1)),
            |ui| {
                ui.register_control(OWNER, SCREEN, Focusability::Focusable);
                ui.retain_owner_cursor(OWNER)
            },
        );
        assert!(
            !retained,
            "an owner registered in the popup is not the parent control"
        );
        assert_eq!(frame.retained_cursor_request, None);
    }

    #[test]
    fn visible_owner_cursor_can_be_retained_without_keyboard_focus() {
        use crate::layer::LayerKind;

        let (retained, frame) = with_cursor_popover(
            LayerKind::Popover,
            false,
            false,
            SCREEN,
            Some(Position::new(4, 1)),
            |ui| ui.retain_visible_owner_cursor(OWNER),
        );
        assert!(retained);
        assert_eq!(
            frame.retained_cursor_request,
            Some(RetainedCursorRequest {
                owner: OWNER,
                parent_layer: LayerId::PAGE,
                overlay_layer: LayerId(1),
                cursor_index: 0,
                mode: CursorRetentionMode::VisibleOwner,
                ambiguous: false,
            })
        );
        assert_eq!(
            frame.cursors.len(),
            1,
            "the existing cursor request is reused"
        );
        assert_eq!(frame.cursors[0].layer, LayerId(1));
        assert_eq!(frame.cursors[0].pos, Position::new(4, 1));

        for (kind, inert, area, cursor) in [
            (LayerKind::Modal, false, SCREEN, Some(Position::new(4, 1))),
            (LayerKind::Popover, true, SCREEN, Some(Position::new(4, 1))),
            (
                LayerKind::Popover,
                false,
                Rect::ZERO,
                Some(Position::new(4, 1)),
            ),
            (LayerKind::Popover, false, SCREEN, None),
        ] {
            let (retained, frame) = with_cursor_popover(kind, inert, false, area, cursor, |ui| {
                ui.retain_visible_owner_cursor(OWNER)
            });
            assert!(!retained);
            assert_eq!(frame.retained_cursor_request, None);
            assert!(
                frame
                    .cursors
                    .iter()
                    .all(|request| request.layer == LayerId::PAGE),
                "a rejected request must not move the underlying cursor"
            );
        }
    }

    #[test]
    fn conflicting_visible_cursor_owners_reject_the_retention() {
        const POPUP: Id = Id::root("ui.cursor-conflict-popup");
        const OTHER: Id = Id::root("ui.cursor-conflict-other");
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        frame.top = LayerId(1);
        frame.layers.push(
            POPUP,
            LayerId(1),
            LayerSpec::popover(
                POPUP,
                crate::layer::Anchor::Screen(crate::layer::ScreenAlign::Center),
            ),
            SCREEN,
            SCREEN,
        );
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            ui.register_control(OWNER, SCREEN, Focusability::Focusable);
            ui.set_bar_cursor(OWNER, Position::new(4, 1));
            ui.register_control(OTHER, SCREEN, Focusability::Focusable);
            ui.set_bar_cursor(OTHER, Position::new(8, 1));
            let _ = ui.layer(POPUP, |ui, _| {
                assert!(ui.retain_visible_owner_cursor(OWNER));
                assert!(!ui.retain_visible_owner_cursor(OTHER));
                assert!(!ui.retain_visible_owner_cursor(OWNER));
            });
        }
        let retained = frame
            .retained_cursor_request
            .expect("ambiguous attempt recorded");
        assert!(retained.ambiguous);
        assert!(frame.diagnostics.items().iter().any(|diagnostic| matches!(
            diagnostic,
            crate::diagnostics::Diagnostic::CursorRejected { owner: OTHER, .. }
        )));
    }

    #[test]
    fn reference_cache_is_namespaced_from_live_and_sibling_scopes() {
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            let formerly_colliding = OWNER.sub("__reference").index(0);
            *ui.cache::<u32>(formerly_colliding) = 7;
            let first = ReferenceTarget::new(OWNER, ReferenceState::FOCUSED);
            let second = ReferenceTarget::new(OTHER, ReferenceState::FOCUSED);
            ui.reference(Some(first), |ui| {
                assert_eq!(*ui.cache::<u32>(OWNER), 0);
                *ui.cache::<u32>(OWNER) = 11;
                ui.reference(Some(second), |ui| {
                    assert_eq!(*ui.cache::<u32>(OWNER), 0);
                    *ui.cache::<u32>(OWNER) = 13;
                });
                assert_eq!(*ui.cache::<u32>(OWNER), 11);
            });
            ui.reference(Some(second), |ui| {
                assert_eq!(*ui.cache::<u32>(OWNER), 13);
            });
            ui.reference(Some(first), |ui| {
                assert_eq!(*ui.cache::<u32>(OWNER), 11);
            });
            assert_eq!(*ui.cache::<u32>(formerly_colliding), 7);
        });
    }

    #[test]
    fn targetless_sibling_reference_scopes_do_not_share_cache() {
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            ui.reference(None, |ui| {
                assert_eq!(*ui.cache::<u32>(OWNER), 0);
                *ui.cache::<u32>(OWNER) = 11;
            });
            ui.reference(None, |ui| {
                assert_eq!(
                    *ui.cache::<u32>(OWNER),
                    0,
                    "a targetless sibling is a distinct reference fixture"
                );
            });
        });
    }

    #[test]
    fn reference_panic_cannot_poison_the_live_cache_namespace() {
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            *ui.cache::<u32>(OWNER) = 7;
            let unwind = catch_unwind(AssertUnwindSafe(|| {
                ui.reference(
                    Some(ReferenceTarget::new(OWNER, ReferenceState::FOCUSED)),
                    |ui| {
                        *ui.cache::<u32>(OWNER) = 99;
                        panic!("reference cache probe");
                    },
                );
            }));
            assert!(unwind.is_err());
            assert_eq!(*ui.cache::<u32>(OWNER), 7);
        });
    }

    #[test]
    fn reference_hides_stale_tab_shaped_area_and_layout() {
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(2, SCREEN);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let mut registry = crate::hit::Registry::default();
        registry.reset(1);
        let _ = registry.register_control(OWNER, Rect::new(2, 1, 20, 1), LayerId::PAGE);
        let last = LastFrame {
            registry,
            layout: vec![(OWNER, LayoutFacts::new(3, 12, 1, 20))],
            ..LastFrame::default()
        };
        let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
        assert_eq!(ui.area(OWNER), Some(Rect::new(2, 1, 20, 1)));
        assert_eq!(ui.layout(OWNER), Some(LayoutFacts::new(3, 12, 1, 20)));
        ui.reference(
            Some(ReferenceTarget::new(OWNER, ReferenceState::FOCUSED)),
            |ui| {
                assert_eq!(ui.area(OWNER), None);
                assert_eq!(ui.layout(OWNER), None);
            },
        );
    }

    #[test]
    fn focused_hints_are_suppressed_inside_a_reference_scope() {
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame {
            snapshot: super::cx::Snapshot {
                focus: Some(OWNER),
                ..super::cx::Snapshot::default()
            },
            ..LastFrame::default()
        };
        let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
        ui.register_control(OWNER, SCREEN, Focusability::Focusable);
        ui.publish_bindings(OWNER, StateFlags::FOCUSED, TEST_BINDINGS);
        assert!(ui.with_focused_hints(|_, _| ()).is_some());
        ui.reference(
            Some(ReferenceTarget::new(OWNER, ReferenceState::FOCUSED)),
            |ui| assert!(ui.with_focused_hints(|_, _| ()).is_none()),
        );
    }

    #[test]
    fn focus_only_is_reachable_without_an_area_or_hit_target() {
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            ui.register_focus_only(OWNER, Focusability::Focusable);

            assert_eq!(ui.frame.ring.next(None), Some(OWNER));
            assert_eq!(ui.frame.ring.entry(OWNER).map(|e| e.area), Some(Rect::ZERO));
            assert_eq!(ui.frame.registry.area_of(OWNER), None);
            assert_eq!(ui.frame.registry.hit(Position::new(0, 0)), None);
        });
    }

    #[test]
    fn focus_only_click_only_is_a_no_op() {
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            ui.register_focus_only(OWNER, Focusability::ClickOnly);

            assert!(!ui.frame.ring.is_registered(OWNER));
            assert!(!ui.frame.registry.has_owner(OWNER));
        });
    }

    #[test]
    fn zero_area_control_still_registers_nothing() {
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            ui.register_control(OWNER, Rect::ZERO, Focusability::Focusable);

            assert!(!ui.frame.ring.is_registered(OWNER));
            assert!(!ui.frame.registry.has_owner(OWNER));
        });
    }

    #[test]
    fn focus_only_preserves_disabled_and_read_only_semantics() {
        const DISABLED: Id = Id::root("ui.disabled");
        const READ_ONLY: Id = Id::root("ui.read-only");

        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            ui.register_focus_only(DISABLED, Focusability::Disabled);
            ui.register_focus_only(READ_ONLY, Focusability::FocusableReadOnly);

            assert!(ui.frame.ring.is_registered(DISABLED));
            assert!(!ui.frame.ring.contains(DISABLED));
            assert!(ui.frame.ring.contains(READ_ONLY));
            assert_eq!(ui.frame.declared, vec![(READ_ONLY, StateFlags::READ_ONLY)]);
        });
    }

    #[test]
    fn with_part_resolves_once_and_records_the_role() {
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            #[cfg(feature = "testing")]
            let before = ui.style_cache_stats();
            let painted = ui.with_part(
                Family::LIST,
                Variant::DEFAULT,
                Part::CONTAINER,
                StateFlags::empty(),
                |ui, r| {
                    let area = Rect::new(0, 0, 4, 1);
                    ui.fill(area, r.style);
                    (area, r)
                },
            );
            #[cfg(feature = "testing")]
            {
                let after = ui.style_cache_stats();
                assert_eq!(after.1, before.1 + 1, "exactly one cache miss");
                assert_eq!(after.0, before.0, "and no hit: it resolved once");
            }
            let (area, r) = painted;

            assert_eq!(ui.full(), SCREEN);
            assert_eq!(ui.surface(), Surface::Canvas);
            for pos in area.positions() {
                assert_eq!(
                    ui.roles_at(pos),
                    CellRoles {
                        fg: Some(Role::Fg(FgStep::Primary)),
                        bg: Some(Role::CurrentSurface),
                    }
                );
            }
            assert_eq!(r.style.bg, Some(theme.bg(Surface::Canvas)));
        });
    }

    #[test]
    fn surface_style_is_the_left_operand_of_the_final_patch() {
        for theme in [Theme::junie(), Theme::paper()] {
            for s in [
                Surface::Canvas,
                Surface::Surface,
                Surface::Elevated,
                Surface::Overlay,
                Surface::Popover,
                Surface::Field,
            ] {
                with_ui(&theme, |ui| {
                    ui.with_surface(s, |ui| {
                        let inherited = ui.surface_style();
                        assert_eq!(inherited.bg, Some(theme.bg(s)));
                        assert_eq!(
                            inherited.fg,
                            crate::theme::resolve::bind_role(&theme, Role::Fg(FgStep::Primary), s)
                        );
                        assert_eq!(inherited.add_modifier, Style::new().add_modifier);
                        for &f in &[Family::LIST, Family::BUTTON, Family::FIELD] {
                            for &p in &[Part::CONTAINER, Part::LABEL, Part::GUTTER] {
                                for st in [StateFlags::empty(), StateFlags::FOCUSED] {
                                    let r = ui.style(f, Variant::DEFAULT, p, st);
                                    assert_eq!(r.over(inherited), inherited.patch(r.style));
                                }
                            }
                        }
                    });
                });
            }
        }
    }

    #[test]
    fn right_align_keeps_the_row_fill_in_the_vacated_prefix() {
        use ratatui_core::style::Color;

        use crate::rowui::CellUi;
        use crate::theme::{Align, PaintStyle};

        let theme = Theme::junie();
        let (_, page) = with_ui(&theme, |ui| {
            let area = Rect::new(0, 0, 12, 1);
            let row = PaintStyle::new()
                .fg(Color::Rgb(255, 255, 255))
                .bg(Color::Rgb(0, 0, 0));
            ui.fill(area, row);
            let mut cell = CellUi::new(ui.reborrow(), area, PaintStyle::new());
            cell.align(Align::Right);
            cell.text("10000");
        });
        for x in 0..7 {
            let cell = page.cell(Position::new(x, 0)).expect("prefix cell");
            assert_eq!(cell.symbol(), " ", "prefix symbol {x}");
            assert_eq!(cell.fg, Color::Rgb(255, 255, 255), "prefix fg {x}");
            assert_eq!(cell.bg, Color::Rgb(0, 0, 0), "prefix bg {x}");
        }
        let text: String = (7..12)
            .map(|x| {
                page.cell(Position::new(x, 0))
                    .expect("digit cell")
                    .symbol()
                    .to_owned()
            })
            .collect();
        assert_eq!(text, "10000");
    }

    #[test]
    fn right_aligned_padding_keeps_semantic_roles_through_dimming() {
        use crate::rowui::CellUi;
        use crate::theme::{Align, PaintStyle};

        let theme = Theme::junie();
        let (_, page) = with_ui(&theme, |ui| {
            let area = Rect::new(0, 0, 12, 1);
            let fill = ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Accent)
                    .set_bg(Role::Surface(Surface::Overlay)),
            );
            ui.fill(area, fill);
            let mut cell = CellUi::new(ui.reborrow(), Rect::new(0, 0, 6, 1), PaintStyle::new());
            cell.align(Align::Right);
            cell.text("1");
            drop(cell);
            ui.dim_layer(area, 1);
        });

        let padding = page.cell(Position::new(0, 0)).expect("padding cell");
        let row_fill = page.cell(Position::new(8, 0)).expect("row fill cell");
        assert_eq!((padding.fg, padding.bg), (row_fill.fg, row_fill.bg));
    }

    #[test]
    fn dim_layer_uses_the_role_of_the_painted_cell() {
        let theme = Theme::junie();
        let backdrop = crate::theme::resolve::bind_role(&theme, Role::BackdropFg, Surface::Canvas);
        let ((), page) = with_ui(&theme, |ui| {
            let row = Rect::new(0, 0, 10, 1);
            {
                let mut r = RowUi::new(
                    ui,
                    OWNER,
                    Family::LIST,
                    Variant::DEFAULT,
                    StateFlags::empty(),
                    ItemKey::num(1),
                    row,
                );
                let mut cell = r.part(Part::META, 6);
                cell.align(crate::theme::Align::Right);
                cell.text("ok");
            }

            for x in 12..24u16 {
                assert_eq!(ui.roles_at(Position::new(x, 0)), CellRoles::default());
            }
            ui.dim_layer(SCREEN, 2);
        });

        let far = page.cell(Position::new(20, 2)).expect("cell");
        assert_eq!(far.fg, backdrop.unwrap_or(far.fg));

        let painted = page.cell(Position::new(1, 0)).expect("cell");
        assert_eq!(painted.bg, theme.bg(Surface::Canvas));
    }

    static PANIC_OVERLAY_OUTER: [OverlayRule; 1] = [(
        Family::LIST,
        Variant::DEFAULT,
        Part::CONTAINER,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Warning),
    )];
    static PANIC_OVERLAY_INNER: [OverlayRule; 1] = [(
        Family::LIST,
        Variant::DEFAULT,
        Part::CONTAINER,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Danger),
    )];

    fn list_container_style(ui: &mut Ui<'_>) -> super::Resolved {
        ui.style(
            Family::LIST,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        )
    }

    #[test]
    fn with_area_and_with_surface_restore_the_outer_view_after_a_panic() {
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            let outer_clip = ui.full();
            let outer_surface = ui.surface();
            let inner = Rect::new(1, 1, 4, 1);

            ui.with_area(inner, |ui| assert_eq!(ui.full(), inner));
            let unwind = catch_unwind(AssertUnwindSafe(|| {
                ui.with_area(inner, |_| panic!("area probe"));
            }));
            assert!(unwind.is_err());
            assert_eq!(
                ui.full(),
                outer_clip,
                "a panicking body must not leave the clip narrowed"
            );

            ui.with_surface(Surface::Popover, |ui| {
                assert_eq!(ui.surface(), Surface::Popover);
            });
            let unwind = catch_unwind(AssertUnwindSafe(|| {
                ui.with_surface(Surface::Popover, |_| panic!("surface probe"));
            }));
            assert!(unwind.is_err());
            assert_eq!(
                ui.surface(),
                outer_surface,
                "a panicking body must not leave the surface switched"
            );
        });
    }

    #[test]
    fn with_overlay_restores_the_stack_after_a_panic() {
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            let plain = list_container_style(ui);
            let overlay = Overlay::new(&PANIC_OVERLAY_OUTER);
            let patched = ui.with_overlay(&overlay, list_container_style);
            assert_ne!(
                patched.style.fg, plain.style.fg,
                "the probe overlay must actually change the resolved style"
            );
            let depth = ui.core.overlays.len();
            let hash = ui.core.stack_hash;

            let unwind = catch_unwind(AssertUnwindSafe(|| {
                ui.with_overlay(&overlay, |_| panic!("overlay probe"));
            }));
            assert!(unwind.is_err());
            assert_eq!(ui.core.overlays.len(), depth);
            assert_eq!(ui.core.stack_hash, hash);
            assert_eq!(
                list_container_style(ui),
                plain,
                "a panicking overlay scope must not leak into later resolution"
            );
        });
    }

    #[test]
    fn nested_overlay_panic_restores_exactly_the_outer_overlay() {
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            let outer = Overlay::new(&PANIC_OVERLAY_OUTER);
            let inner = Overlay::new(&PANIC_OVERLAY_INNER);
            ui.with_overlay(&outer, |ui| {
                let outer_style = list_container_style(ui);
                let inner_style = ui.with_overlay(&inner, list_container_style);
                assert_ne!(outer_style.style.fg, inner_style.style.fg);
                assert_eq!(list_container_style(ui), outer_style);

                let unwind = catch_unwind(AssertUnwindSafe(|| {
                    ui.with_overlay(&inner, |_| panic!("nested overlay probe"));
                }));
                assert!(unwind.is_err());
                assert_eq!(ui.core.overlays.len(), 1, "exactly the outer overlay");
                assert_eq!(
                    list_container_style(ui),
                    outer_style,
                    "unwinding pops the inner overlay only"
                );
            });
            assert!(ui.core.overlays.is_empty());
            assert_eq!(ui.core.stack_hash, 0);
        });
    }

    #[test]
    fn focus_scope_restores_the_open_scope_after_a_panic() {
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            let root = ui.frame.ring.current_scope();
            let unwind = catch_unwind(AssertUnwindSafe(|| {
                ui.focus_scope(OWNER, crate::focus::ScopeMode::Trap, |ui| {
                    ui.register_control(OWNER, Rect::new(0, 0, 4, 1), Focusability::Focusable);
                    panic!("focus scope probe");
                });
            }));
            assert!(unwind.is_err());
            assert_eq!(ui.frame.ring.current_scope(), root);

            ui.register_control(OTHER, Rect::new(4, 0, 4, 1), Focusability::Focusable);
            let entry = ui
                .frame
                .ring
                .entries()
                .iter()
                .find(|e| e.id == OTHER)
                .copied()
                .expect("entry");
            assert_eq!(
                entry.scope, root,
                "a control after the panic belongs to the outer scope, not the trap"
            );
        });
    }

    #[test]
    fn layer_restores_the_page_target_and_scope_after_a_panic() {
        const LAYER: Id = Id::root("ui.panic.layer");
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        let layer_area = Rect::new(1, 1, 4, 1);
        frame.layers.push(
            LAYER,
            LayerId(1),
            LayerSpec::modal(LAYER),
            layer_area,
            SCREEN,
        );
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let root;
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            root = ui.frame.ring.current_scope();
            let unwind = catch_unwind(AssertUnwindSafe(|| {
                ui.layer(LAYER, |ui, area| {
                    assert_eq!(ui.full(), area);
                    assert_eq!(ui.layer_id(), LayerId(1));
                    assert_eq!(ui.surface(), Surface::Overlay);
                    assert_ne!(ui.frame.ring.current_scope(), root);
                    panic!("layer probe");
                });
            }));
            assert!(unwind.is_err());
            assert_eq!(ui.full(), SCREEN);
            assert_eq!(ui.layer_id(), LayerId::PAGE);
            assert_eq!(ui.surface(), Surface::Canvas);
            assert!(!ui.is_inert());
            assert_eq!(ui.frame.ring.current_scope(), root);

            ui.fill(SCREEN, Style::new().bg(theme.bg(Surface::Canvas)));
            ui.register_control(OWNER, Rect::new(0, 0, 4, 1), Focusability::Focusable);
        }

        let entry = frame
            .ring
            .entries()
            .iter()
            .find(|e| e.id == OWNER)
            .copied()
            .expect("entry");
        assert_eq!(entry.scope, root);
        assert_eq!(
            entry.layer,
            LayerId::PAGE,
            "a control after the panic registers on the page layer"
        );
        assert_eq!(
            page.cell(Position::new(20, 2)).expect("cell").bg,
            theme.bg(Surface::Canvas),
            "painting after the panic reaches the page outside the layer's area"
        );
    }
    #[test]
    fn independent_carrier_layer_composite_keeps_origin() {
        const LAYER: Id = Id::root("review.layer");
        let theme = Theme::junie();
        with_ui(&theme, |ui| {
            let page = ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .set_bg(Role::Surface(Surface::Canvas)),
            );
            ui.fill(SCREEN, page);
            ui.frame
                .layers
                .push(LAYER, LayerId(1), LayerSpec::modal(LAYER), SCREEN, SCREEN);
            ui.layer(LAYER, |ui, _| {
                let st = ui.paint_patch(
                    &StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Ghost))
                        .set_bg(Role::Surface(Surface::Overlay)),
                );
                ui.paint_str(Rect::new(1, 0, 1, 1), "X", st);
            });
            ui.composite(0);
            assert_eq!(
                ui.roles_at(Position::new(1, 0)),
                CellRoles {
                    fg: Some(Role::Fg(FgStep::Ghost)),
                    bg: Some(Role::Surface(Surface::Overlay))
                },
                "composited cells must carry layer origins, not page origins"
            );
            ui.dim_layer(SCREEN, 1);
            assert_eq!(
                ui.page.cell((1, 0)).unwrap().symbol(),
                " ",
                "Ghost on lower layer must erase under next modal"
            );
        });
    }

    #[test]
    fn layer_scope_parent_follows_active_order_after_id_gap() {
        const OUTER: Id = Id::root("ui.layer.outer");
        const INNER: Id = Id::root("ui.layer.inner");
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        frame
            .layers
            .push(OUTER, LayerId(1), LayerSpec::modal(OUTER), SCREEN, SCREEN);

        frame
            .layers
            .push(INNER, LayerId(3), LayerSpec::modal(INNER), SCREEN, SCREEN);
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            assert!(
                ui.layer(OUTER, |ui, _| {
                    ui.register_control(OWNER, SCREEN, Focusability::Focusable);
                })
                .is_some()
            );
            assert!(
                ui.layer(INNER, |ui, _| {
                    ui.register_control(OTHER, SCREEN, Focusability::Focusable);
                })
                .is_some()
            );
        }
        let outer = frame.ring.entry(OWNER).map(|entry| entry.scope);
        let inner = frame.ring.entry(OTHER).map(|entry| entry.scope);
        assert!(
            matches!((outer, inner), (Some(outer), Some(inner)) if frame.ring.within(inner, outer))
        );
    }
}
