//! The draw-phase context (`COMPONENT_ARCHITECTURE.md` §5, §10, §17.0 A2).
//!
//! `Ui` carries a clip rect, the current surface, the overlay stack and the
//! draw target (the page or a pooled layer buffer). All painting goes
//! through it so a layer's written-cell bitset is always correct, clipping
//! is automatic, and roles are recorded per painted cell for `dim_layer`.

pub(crate) mod cx;
pub(crate) mod derived;
pub(crate) mod layer_buf;
pub(crate) mod paint;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Color;

use core::ops::{BitOr, BitOrAssign};

use cx::LastFrame;
pub use cx::{Cx, FrameRead, LayoutFacts};
use derived::{DerivedCache, ReferenceCacheKey};
use layer_buf::LayerPool;

use crate::action::ActionKey;
use crate::cursor::CursorRequest;
use crate::diagnostics::Diagnostic;
use crate::focus::{FocusEntry, FocusRing, Focusability, ScopeId, ScopeMode};
use crate::hit::{Axes, Headroom, Registry};
use crate::id::{Id, Part, PartRef};
use crate::keymap::{
    Binding, BindingRegistry, DynamicBindingRegistry, FocusedHintKey, FocusedHints, HintLayer,
    KeyMap,
};
use crate::layer::{LayerId, LayerKind};
use crate::response::StateFlags;
use crate::theme::resolve::StyleCache;
use crate::theme::{DesignTokens, Family, Overlay, Resolved, Role, Surface, Theme, Variant};

/// Roles recorded for one painted cell (`FrameOut::roles`, §11.6).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct CellRoles {
    pub(crate) fg: Option<Role>,
    pub(crate) bg: Option<Role>,
}

/// Per-frame output the runtime consumes after `app.draw` (§3.3 steps 12–15).
#[derive(Debug, Default)]
pub(crate) struct FrameState {
    pub(crate) registry: Registry,
    pub(crate) ring: FocusRing,
    pub(crate) layers: LayerPool,
    pub(crate) cursor: Option<CursorRequest>,
    pub(crate) layout: Vec<(Id, LayoutFacts)>,
    pub(crate) declared: Vec<(Id, StateFlags)>,
    pub(crate) bindings: BindingRegistry,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) roles: Vec<CellRoles>,
    pub(crate) screen: Rect,
    pub(crate) inert_floor: LayerId,
    pub(crate) top: LayerId,
    #[cfg(feature = "testing")]
    pub(crate) styled_parts: Vec<(Id, Part)>,
    #[cfg(feature = "testing")]
    pub(crate) styled_queries: Vec<StyledQuery>,
}

/// One recorded style query: who asked, under which family/variant, for which
/// part, and what came back (§16.4's theme-coupling migration contract).
/// `Runtime::resolved` reads this, so a migrated assertion sees the family the
/// component actually queried rather than a hardcoded guess (BL-7).
#[cfg(feature = "testing")]
pub type StyledQuery = (Id, Family, Variant, Part, Resolved);

/// Runtime-owned visual state injected into one inert reference rendering.
///
/// This type is intentionally opaque: reference fixtures can reproduce only
/// focus, hover and press state. Semantic state remains owned by component
/// props and caller-owned state.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct ReferenceState(u8);

impl ReferenceState {
    /// The target owns focus.
    pub const FOCUSED: Self = Self(1 << 0);
    /// Keyboard-visible focus is painted.
    pub const FOCUS_VISIBLE: Self = Self(1 << 1);
    /// The pointer is over the target.
    pub const HOVERED: Self = Self(1 << 2);
    /// The primary pointer button is down on the target.
    pub const PRESSED: Self = Self(1 << 3);

    /// Combine two reference-state flags in a constant context.
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

/// The exact owner, optional sub-region and runtime state of a reference cell.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ReferenceTarget {
    id: Id,
    part: Option<PartRef>,
    state: ReferenceState,
}

impl ReferenceTarget {
    /// Target one component owner with `state`.
    pub const fn new(id: Id, state: ReferenceState) -> Self {
        Self {
            id,
            part: None,
            state,
        }
    }

    /// Target one exact sub-region for hover/press queries.
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
    pub(crate) fn reset(&mut self, generation: u32, screen: Rect) {
        self.registry.reset(generation);
        self.ring.reset();
        self.layers.begin();
        self.cursor = None;
        self.layout.clear();
        self.declared.clear();
        self.bindings.reset();
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

/// State that outlives a frame: the derived cache, the style memo and the
/// overlay stack (§20.9-2 P4: constructed once per runtime, reused).
#[derive(Default)]
pub(crate) struct UiCore {
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

    /// Drop every derived cache (resize, theme change, generation gap).
    pub(crate) fn clear_caches(&mut self) {
        self.cache.clear();
        self.reference_cache.clear();
        self.style_cache.clear();
    }

    fn targetless_reference_key(&mut self) -> ReferenceCacheKey {
        let key = self.next_targetless_reference;
        self.next_targetless_reference = self.next_targetless_reference.wrapping_add(1);
        if self.next_targetless_reference == 0 {
            // Cache contents are derived-only, so clearing at the practically
            // unreachable wrap boundary preserves identity without semantics.
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

/// The draw-phase context.
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
    roles: CellRoles,
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
    pub(crate) fn new(
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
            roles: CellRoles::default(),
        }
    }

    /// A shorter-lived view sharing every reference (for `RowUi` and nested painters).
    pub(crate) fn reborrow(&mut self) -> Ui<'_> {
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
            roles: self.roles,
        }
    }

    /// The current clip rect.
    pub const fn full(&self) -> Rect {
        self.clip
    }

    /// The current surface.
    pub const fn surface(&self) -> Surface {
        self.surface
    }

    /// The current surface's background colour.
    pub fn bg(&self) -> Color {
        self.theme.bg(self.surface)
    }

    /// The layer being drawn.
    pub const fn layer_id(&self) -> LayerId {
        self.layer
    }

    /// Whether registrations from this layer are suppressed (`inert_below`).
    pub const fn is_inert(&self) -> bool {
        self.inert || self.reference.is_some()
    }

    /// Draw an inert reference fixture.
    ///
    /// `target` injects runtime-owned focus, hover and press state into exactly
    /// one owner. `None` is an inert default or semantic fixture with no
    /// synthetic runtime state. Painting and style resolution still run;
    /// interaction, layout, cursor, binding and layer output is suppressed.
    /// This is an application-showcase and test-fixture API; component
    /// implementations must derive live state from [`FrameRead`] instead.
    ///
    /// Nested calls replace the target only for the nested closure. The scope
    /// lives on a shorter reborrow, so normal return and panic unwinding both
    /// restore the exact outer target without cleanup code. An explicit target
    /// also supplies a stable derived-cache namespace. A targetless scope has
    /// no stable caller identity, so its derived cache is isolated to that one
    /// invocation instead of being shared with sibling fixtures.
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

    /// Resolve a part's style through the whole precedence chain, memoised.
    pub fn style(
        &mut self,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
    ) -> Resolved {
        let acc = self.core.style_cache.accumulate(
            self.theme,
            family,
            variant,
            part,
            flags,
            &self.core.overlays,
            self.core.stack_hash,
        );
        let r = crate::theme::resolve::bind(self.theme, acc, None, self.surface);
        self.roles = CellRoles {
            fg: acc.fg.get(),
            bg: acc.bg.get(),
        };
        r
    }

    /// Resolve with a per-instance patch (precedence 6).
    pub fn style_patched(
        &mut self,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
        patch: &crate::theme::StylePatch,
    ) -> Resolved {
        let acc = self.core.style_cache.accumulate(
            self.theme,
            family,
            variant,
            part,
            flags,
            &self.core.overlays,
            self.core.stack_hash,
        );
        let merged = acc.merge(*patch);
        self.roles = CellRoles {
            fg: merged.fg.get(),
            bg: merged.bg.get(),
        };
        crate::theme::resolve::bind(self.theme, acc, Some(patch), self.surface)
    }

    /// Record that `owner` resolved `part` under `family`/`variant`, and what
    /// it got (the declared-parts check and `Runtime::resolved`).
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

    /// The `(owner, part)` pairs styled this frame — the declared-parts check.
    #[cfg(feature = "testing")]
    pub fn styled_parts(&self) -> &[(Id, Part)] {
        &self.frame.styled_parts
    }

    /// The full style queries recorded this frame, with the family, variant
    /// and the `Resolved` each one produced.
    #[cfg(feature = "testing")]
    pub fn styled_queries(&self) -> &[StyledQuery] {
        &self.frame.styled_queries
    }

    /// `(hits, misses)` of the §11.1 A3 memo since the runtime was built.
    #[cfg(feature = "testing")]
    pub fn style_cache_stats(&self) -> (u64, u64) {
        self.core.style_cache.stats()
    }

    /// Resolve a part through the whole §11.3 chain **without** the memo and
    /// **without** recording roles — the `&self` path, for `Measure::measure`
    /// and any read that must not paint (Adjudication N2).
    ///
    /// Identical to [`Ui::style`] in result (same family/variant/state chain,
    /// same live overlay stack, same current surface); it differs only in what
    /// it does *not* do. Excludes the per-instance patch (precedence 6), which
    /// the caller merges with `StylePatch::merge` if it has one.
    ///
    /// Costs one uncached accumulation and zero allocations. Use [`Ui::style`]
    /// on the painting path: a measurement must not evict a painting entry
    /// from the 256-slot memo (§11.1 A3, §20.9-2).
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

    /// The glyph a role currently maps to (`design.glyphs`). `&self`, so it is
    /// reachable from `measure`; pair with `text::width` for its cell width.
    pub fn glyph_str(&self, g: crate::theme::GlyphRole) -> &'static str {
        self.theme.design.glyphs.get(g)
    }

    /// The style a child inherits from the current surface: `bg` is
    /// `theme.bg(ui.surface())`, `fg` is `Role::Fg(FgStep::Primary)` bound on
    /// that surface, no modifiers. The **left** operand of §11.3's final
    /// layering — write it as `resolved.over(ui.surface_style())`.
    pub fn surface_style(&self) -> ratatui_core::style::Style {
        let mut st = ratatui_core::style::Style::new();
        st.bg = Some(self.theme.bg(self.surface));
        st.fg = crate::theme::resolve::bind_role(
            self.theme,
            Role::Fg(crate::theme::FgStep::Primary),
            self.surface,
        );
        st
    }

    /// Resolve `part` once and paint with it: equivalent to binding
    /// `let r = ui.style(family, variant, part, flags);` and then painting —
    /// including the memo lookup and the per-cell role recording `dim_layer`
    /// reads — but expressible as one statement.
    ///
    /// Binds a value only: it pushes **no** clip and **no** surface (use
    /// [`Ui::with_area`] / [`Ui::with_surface`] for those). A component with a
    /// per-instance patch keeps the two-step [`Ui::style_patched`] shape;
    /// there is deliberately no `with_` form for precedence 6.
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

    /// Run `f` with the clip rect intersected with `area`.
    pub fn with_area<R>(&mut self, area: Rect, f: impl FnOnce(&mut Ui<'_>) -> R) -> R {
        let saved = self.clip;
        self.clip = self.clip.intersection(area);
        let r = f(self);
        self.clip = saved;
        r
    }

    /// Run `f` on surface `s`; children inherit it.
    pub fn with_surface<R>(&mut self, s: Surface, f: impl FnOnce(&mut Ui<'_>) -> R) -> R {
        let saved = self.surface;
        self.surface = s;
        let r = f(self);
        self.surface = saved;
        r
    }

    /// Run `f` with `ov` pushed on the overlay stack (precedence 5).
    pub fn with_overlay<R>(&mut self, ov: &Overlay, f: impl FnOnce(&mut Ui<'_>) -> R) -> R {
        let saved_hash = self.core.stack_hash;
        self.core.overlays.push(*ov);
        self.core.stack_hash = saved_hash.rotate_left(7) ^ ov.hash();
        let r = f(self);
        self.core.overlays.pop();
        self.core.stack_hash = saved_hash;
        r
    }

    /// Run `f` inside a focus scope.
    pub fn focus_scope<R>(
        &mut self,
        id: Id,
        mode: ScopeMode,
        f: impl FnOnce(&mut Ui<'_>) -> R,
    ) -> R {
        if self.reference.is_some() {
            return f(self);
        }
        self.frame
            .ring
            .push_scope(ScopeId::new(id), mode, self.layer);
        let r = f(self);
        self.frame.ring.pop_scope();
        r
    }

    fn register_entry(&mut self, id: Id, area: Rect, f: Focusability, swallows_typing: bool) {
        if self.registrations_suppressed() || area.is_empty() {
            return;
        }
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return;
        }
        if let Some(d) = self.frame.registry.register_control(id, area, self.layer) {
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

    /// Register a `Control` region and its ring entry.
    pub fn register_control(&mut self, id: Id, area: Rect, f: Focusability) {
        self.register_entry(id, area, f, false);
    }

    /// Register a hidden keyboard focus stop without creating a hit region.
    ///
    /// `Focusable` and `FocusableReadOnly` are reachable in normal traversal;
    /// `Disabled` is recorded but unreachable. `ClickOnly` is a no-op because
    /// focus-only registration has no pointer target. The ring entry uses
    /// `Rect::ZERO`, so `area_of(id)` remains `None` and a harness `click_id`
    /// is ignored with [`Diagnostic::UnaddressableId`].
    pub fn register_focus_only(&mut self, id: Id, f: Focusability) {
        if self.registrations_suppressed() {
            return;
        }
        self.register_focus_entry(id, Rect::ZERO, f, false);
    }

    /// Register a `Control` whose entry swallows typing (a text control)
    /// and declares `flags` (`EDITING` while an edit is in flight), so
    /// paste and bare-`Char` capture chords are routed correctly.
    pub fn register_editor(&mut self, id: Id, area: Rect, f: Focusability, flags: StateFlags) {
        self.register_entry(id, area, f, true);
        self.declare_state(id, flags);
    }

    /// Declare non-runtime flags for `id` (`EDITING`, `DIRTY`, `BUSY`, …) so
    /// `FrameRead::state` reports them next frame.
    ///
    /// **One-frame contract (D-6)**: declared flags live in *last* frame's
    /// `declared` list, so they are read back on the **next** frame — the same
    /// rule as `cx.area` (§4 S3). A paste in the same `handle` that began an
    /// edit is therefore not routed as editing; the edit must have been
    /// declared by a previous draw.
    ///
    /// It is a `draw`-phase write the runtime consumes, alongside
    /// [`Ui::report_layout`] (§5 R2).
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

    /// Publish the focused control's declared command table for next input
    /// routing and same-frame derived hints.
    pub fn publish_bindings<C: Copy + 'static>(
        &mut self,
        owner: Id,
        flags: StateFlags,
        table: &'static [Binding<C>],
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
        if let Some(action) = self.frame.bindings.publish(owner, flags, self.layer, table) {
            self.frame
                .diagnostics
                .push(Diagnostic::DuplicateBindingAction { owner, action });
        }
    }

    pub(crate) fn publish_dynamic_bindings<I>(&mut self, owner: Id, flags: StateFlags, bindings: I)
    where
        I: Iterator<Item = (ActionKey, Option<crate::event::Chord>)> + Clone,
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
        let base = self
            .frame
            .bindings
            .get(owner)
            .map(|(published, _)| published.table);
        let (descriptors, revision) = self.core.dynamic_bindings.update(owner, base, bindings);
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

    pub(crate) fn effective_chord(
        &self,
        owner: Id,
        action: ActionKey,
        default: Option<crate::event::Chord>,
    ) -> Option<crate::event::Chord> {
        self.core.keymap.component_chord(owner, action, default)
    }

    pub(crate) fn with_focused_hints<R>(
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

    /// Register a `Part` region under `owner`.
    pub fn register_part(&mut self, owner: Id, part: PartRef, area: Rect) {
        if self.registrations_suppressed() {
            return;
        }
        self.frame
            .registry
            .register_part(owner, part, area.intersection(self.clip), self.layer);
    }

    /// Register a `Decorative` region under `owner`.
    pub fn register_decor(&mut self, owner: Id, part: PartRef, area: Rect) {
        if self.registrations_suppressed() {
            return;
        }
        self.frame
            .registry
            .register_decor(owner, part, area.intersection(self.clip), self.layer);
    }

    /// Register a `Scroll` region.
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

    /// Report layout facts upward (§4 S6).
    pub fn report_layout(&mut self, id: Id, l: LayoutFacts) {
        if self.reference.is_some() {
            return;
        }
        self.frame.layout.push((id, l));
    }

    /// Request the hardware cursor; kept iff this is the top layer and
    /// `owner` is focused (§8.4).
    pub fn set_cursor(&mut self, owner: Id, pos: Position) {
        if self.reference.is_some() {
            return;
        }
        let req = CursorRequest {
            layer: self.layer,
            owner,
            pos,
            inert: self.inert,
            focused: self.state(owner).contains(StateFlags::FOCUSED),
        };
        // §8.4 makes filtering the runtime's job, so components write
        // unconditionally: keep the *best* candidate — higher layer first,
        // then the focused owner, then the later write — never the first
        // arrival, which would hand the frame's only cursor slot to whoever
        // happened to draw first (BL-6).
        let keep = match self.frame.cursor {
            None => true,
            Some(cur) => (req.layer, req.focused) >= (cur.layer, cur.focused),
        };
        let loser = if keep {
            let prev = self.frame.cursor;
            self.frame.cursor = Some(req);
            prev
        } else {
            Some(req)
        };
        if let Some(l) = loser
            && !l.inert
        {
            self.frame.diagnostics.push(Diagnostic::CursorRejected {
                owner: l.owner,
                layer: l.layer,
            });
        }
    }

    /// Draw layer `id`'s content. Resolves `id` to the `LayerId` assigned at
    /// `open_layer`, paints into its pooled buffer and pushes its focus
    /// scope; returns `None` without running `f` if `id` is not open, and
    /// records `DuplicateLayerDraw` on a second call in one frame.
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
        let saved = (self.target, self.clip, self.layer, self.inert, self.surface);
        self.target = Target::Layer(idx);
        self.clip = area;
        self.layer = layer;
        self.inert = layer < self.frame.inert_floor;
        self.surface = match kind {
            LayerKind::Modal => Surface::Overlay,
            LayerKind::Popover | LayerKind::Tooltip => Surface::Popover,
        };
        let mode = match kind {
            LayerKind::Modal => ScopeMode::Trap,
            LayerKind::Popover | LayerKind::Tooltip => ScopeMode::Normal,
        };
        let parent = self
            .frame
            .layers
            .active()
            .iter()
            .find(|candidate| candidate.layer.index().saturating_add(1) == layer.index())
            .map(|candidate| ScopeId::new(candidate.id));
        self.frame
            .ring
            .ensure_scope(ScopeId::new(id), mode, layer, parent);
        self.frame.ring.push_scope(ScopeId::new(id), mode, layer);
        let r = f(self, area);
        self.frame.ring.pop_scope();
        (self.target, self.clip, self.layer, self.inert, self.surface) = saved;
        Some(r)
    }

    /// Derived, non-semantic per-component cache (rule R8). Live entries are
    /// keyed by `(Id, TypeId)`; reference entries also include their scope
    /// namespace. Cleared on resize, theme change and generation gap.
    pub fn cache<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        // A structurally separate store prevents any caller-chosen `Id` from
        // aliasing live derived state. Its target dimension is stable across
        // frames, unlike draw-order ordinals. The style memo is safe to share:
        // its key already contains every style input.
        if let Some(scope) = self.reference {
            self.core
                .reference_cache
                .get_mut_reference::<T>(id, scope.cache_key)
        } else {
            self.core.cache.get_mut::<T>(id)
        }
    }

    // ── internals shared with paint.rs and the runtime ──

    /// The buffer and `area` clipped to the current clip rect, marking
    /// **`area`** written — not the whole clip, which [`Ui::raw`] marks.
    ///
    /// The internal painters that move already-painted cells (`CellUi`'s
    /// alignment shift, `RowUi::raw`) must use this: marking the component's
    /// whole clip would make a layer's written-cell bitset all-true and
    /// composite unpainted cells over the page (§3.3 step 12, R3), and would
    /// stamp the current role over every cell `dim_layer` later walks.
    pub(crate) fn buffer_in(&mut self, area: Rect) -> (&mut Buffer, Rect) {
        let a = area.intersection(self.clip);
        self.mark_area(a);
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

    pub(crate) fn mark(&mut self, pos: Position) {
        match self.target {
            Target::Page => {
                if let Some(i) = self.frame.role_index(pos)
                    && let Some(r) = self.frame.roles.get_mut(i)
                {
                    *r = self.roles;
                }
            }
            Target::Layer(i) => {
                if let Some(d) = self.frame.layers.active_mut().get_mut(i) {
                    d.mark(pos);
                }
            }
        }
    }

    pub(crate) fn mark_area(&mut self, area: Rect) {
        let area = area.intersection(self.clip);
        match self.target {
            Target::Page => {
                for pos in area.positions() {
                    self.mark(pos);
                }
            }
            Target::Layer(i) => {
                if let Some(d) = self.frame.layers.active_mut().get_mut(i) {
                    d.mark_area(area);
                }
            }
        }
    }

    pub(crate) const fn theme_ref(&self) -> &'f Theme {
        self.theme
    }

    pub(crate) fn set_roles(&mut self, roles: CellRoles) {
        self.roles = roles;
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

    /// The number of layers drawn this frame (open layers).
    pub(crate) fn layer_count(&self) -> usize {
        self.frame.layers.active().len()
    }

    /// `(backdrop, drawn)` of layer `i`.
    pub(crate) fn layer_meta(&self, i: usize) -> Option<(crate::layer::Backdrop, bool)> {
        self.frame
            .layers
            .active()
            .get(i)
            .map(|d| (d.spec.backdrop, d.drawn))
    }

    /// Copy layer `i`'s written cells onto the page.
    pub(crate) fn composite(&mut self, i: usize) {
        if let Some(d) = self.frame.layers.active().get(i) {
            d.composite_onto(self.page);
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
        CellRoles, FrameRead, FrameState, LayoutFacts, ReferenceState, ReferenceTarget, Ui, UiCore,
    };
    use crate::action::ActionKey;
    use crate::collection::RowUi;
    use crate::event::{Chord, KeyCode};
    use crate::focus::Focusability;
    use crate::hit::{Axes, Headroom};
    use crate::id::{Id, ItemKey, Part, PartRef};
    use crate::keymap::Binding;
    use crate::layer::{LayerId, LayerSpec};
    use crate::response::StateFlags;
    use crate::theme::{Family, FgStep, Role, Surface, Theme, Variant};

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
        assert!(frame.cursor.is_none());
        assert!(frame.bindings.get(OWNER).is_none());
        assert!(frame.diagnostics.is_empty());
        assert!(
            frame
                .layers
                .active()
                .first()
                .is_some_and(|layer| !layer.drawn)
        );
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

    /// `with_part` is a convenience over `style`, not a second resolution
    /// path: it resolves exactly once and records the role for `dim_layer`.
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
            // it binds a value only: no clip and no surface were pushed
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

    /// §11.3's final layering is `inherited.patch(resolved.style)`;
    /// `Resolved::over` is that expression and `Ui::surface_style` is its
    /// left operand.
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

    /// BL-3: a right-aligned `CellUi` shifts painted cells through the buffer
    /// and must mark **its own rect**, not the component's clip. Marking the
    /// clip stamped the cell's role over every cell of the row, so
    /// `dim_layer` dimmed unpainted cells with the wrong role.
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
            // the cells outside the row keep the default role: the shift did
            // not mark the whole clip
            for x in 12..24u16 {
                assert_eq!(ui.roles_at(Position::new(x, 0)), CellRoles::default());
            }
            ui.dim_layer(SCREEN, 2);
        });
        // a never-painted cell dims to the backdrop foreground …
        let far = page.cell(Position::new(20, 2)).expect("cell");
        assert_eq!(far.fg, backdrop.unwrap_or(far.fg));
        // … while a painted row cell keeps a surface-derived background
        let painted = page.cell(Position::new(1, 0)).expect("cell");
        assert_eq!(painted.bg, theme.bg(Surface::Canvas));
    }
}
