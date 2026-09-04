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

use cx::LastFrame;
pub use cx::{Cx, FrameRead, LayoutFacts};
use derived::DerivedCache;
use layer_buf::LayerPool;

use crate::cursor::CursorRequest;
use crate::diagnostics::Diagnostic;
use crate::focus::{FocusEntry, FocusRing, Focusability, ScopeId, ScopeMode};
use crate::hit::{Axes, Headroom, Registry};
use crate::id::{Id, Part, PartRef};
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

impl FrameState {
    pub(crate) fn reset(&mut self, generation: u32, screen: Rect) {
        self.registry.reset(generation);
        self.ring.reset();
        self.layers.begin();
        self.cursor = None;
        self.layout.clear();
        self.declared.clear();
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
    pub(crate) style_cache: StyleCache,
    overlays: Vec<Overlay>,
    stack_hash: u64,
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
    /// Drop every derived cache (resize, theme change, generation gap).
    pub(crate) fn clear_caches(&mut self) {
        self.cache.clear();
        self.style_cache.clear();
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
    roles: CellRoles,
}

impl core::fmt::Debug for Ui<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Ui")
            .field("clip", &self.clip)
            .field("surface", &self.surface)
            .field("layer", &self.layer)
            .field("inert", &self.inert)
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
        self.inert
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
        self.frame
            .ring
            .push_scope(ScopeId::new(id), mode, self.layer);
        let r = f(self);
        self.frame.ring.pop_scope();
        r
    }

    fn register_entry(&mut self, id: Id, area: Rect, f: Focusability, swallows_typing: bool) {
        if self.inert || area.is_empty() {
            return;
        }
        let area = area.intersection(self.clip);
        if area.is_empty() {
            return;
        }
        if let Some(d) = self.frame.registry.register_control(id, area, self.layer) {
            self.frame.diagnostics.push(d);
        }
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
        if flags.is_empty() {
            return;
        }
        if let Some((_, f)) = self.frame.declared.iter_mut().find(|(i, _)| *i == id) {
            *f |= flags;
        } else {
            self.frame.declared.push((id, flags));
        }
    }

    /// Register a `Part` region under `owner`.
    pub fn register_part(&mut self, owner: Id, part: PartRef, area: Rect) {
        if self.inert {
            return;
        }
        self.frame
            .registry
            .register_part(owner, part, area.intersection(self.clip), self.layer);
    }

    /// Register a `Decorative` region under `owner`.
    pub fn register_decor(&mut self, owner: Id, part: PartRef, area: Rect) {
        if self.inert {
            return;
        }
        self.frame
            .registry
            .register_decor(owner, part, area.intersection(self.clip), self.layer);
    }

    /// Register a `Scroll` region.
    pub fn register_scroll(&mut self, id: Id, area: Rect, axes: Axes, head: Headroom) {
        if self.inert {
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
        self.frame.layout.push((id, l));
    }

    /// Request the hardware cursor; kept iff this is the top layer and
    /// `owner` is focused (§8.4).
    pub fn set_cursor(&mut self, owner: Id, pos: Position) {
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
        self.frame.ring.push_scope(ScopeId::new(id), mode, layer);
        let r = f(self, area);
        self.frame.ring.pop_scope();
        (self.target, self.clip, self.layer, self.inert, self.surface) = saved;
        Some(r)
    }

    /// Derived, non-semantic per-component cache (rule R8). Keyed by
    /// `(Id, TypeId)`; cleared on resize, theme change and generation gap.
    pub fn cache<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        self.core.cache.get_mut::<T>(id)
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
        self.last.state(id)
    }

    fn hovered_part(&self, owner: Id) -> Option<PartRef> {
        self.last.hovered_part(owner)
    }

    fn theme(&self) -> &Theme {
        self.theme
    }

    fn design(&self) -> &DesignTokens {
        &self.theme.design
    }

    fn area(&self, id: Id) -> Option<Rect> {
        self.last.registry.area_of(id)
    }

    fn layout(&self, id: Id) -> Option<LayoutFacts> {
        self.last.layout_of(id)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::Style;

    use super::cx::LastFrame;
    use super::{CellRoles, FrameState, Ui, UiCore};
    use crate::collection::RowUi;
    use crate::id::{Id, ItemKey, Part};
    use crate::response::StateFlags;
    use crate::theme::{Family, FgStep, Role, Surface, Theme, Variant};

    const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 24,
        height: 3,
    };
    const OWNER: Id = Id::root("ui.owner");

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
