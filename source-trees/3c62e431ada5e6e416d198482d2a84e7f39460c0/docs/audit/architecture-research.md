# Architecture Research — Component Model, Theme, Composition, Package Boundary

Scope: read-only research for `COMPONENT_ARCHITECTURE.md`. **Facts** are cited `file:line`. **Inference / Recommendation** sections are judgment and are labelled.

---

## 1. Theme current state (facts)

### 1.1 Every token field in `src/theme.rs`

`Theme` is a `Copy` struct of 30 `Color` fields plus a capability level (`src/theme.rs:98-142`):

| Group | Fields (line) |
|---|---|
| capability | `level: ColorLevel` (`theme.rs:100`) |
| planes | `canvas`, `surface`, `surface_elevated`, `surface_overlay`, `field`, `field_hover`, `popover` (`theme.rs:102-108`) |
| menu hues | `highlight` (`theme.rs:111`), `highlight_danger` (`theme.rs:113`), `error_soft` (`theme.rs:116`) |
| borders | `border_subtle`, `border_strong` (`theme.rs:118-119`) |
| fg ladder | `text_primary`, `text_secondary`, `text_muted`, `text_faint`, `text_ghost`, `text_on_accent` (`theme.rs:121-127`) |
| accent | `accent`, `accent_hover`, `accent_pressed`, `accent_bg`, `accent_bg_subtle`, `focus` (`theme.rs:129-134`) |
| status | `disabled`, `error`, `error_bg`, `warning`, `success`, `info` (`theme.rs:136-141`) |

Raw palette constants live in a private `mod palette` (`theme.rs:56-95`); `Theme::junie()` is the only binding of palette → tokens (`theme.rs:145-180`).

Resolvers (behaviour, not data) hang off the same struct: `base/on/primary/secondary/muted/faint/accent_fg/error_fg/title/label/key_hint_key/key_hint_action/border` (`theme.rs:229-291`), `backdrop` (`theme.rs:297-321`), `row` (`theme.rs:329-359`), `lift` (`theme.rs:362-372`), `gutter` (`theme.rs:376-385`), `button` (`theme.rs:387-451`), `field_style` (`theme.rs:454-464`), `placeholder` (`theme.rs:466-472`), `selection` (`theme.rs:474-476`), `scrollbar_track/thumb` (`theme.rs:478-490`), `tone` (`theme.rs:492-502`), `syntax` (`theme.rs:506-519`), `badge` (`theme.rs:521-528`).

Closed enums that gate customization: `ButtonKind{Primary,Secondary,Subtle,Danger,Toggle}` (`theme.rs:558-565`), `BadgeKind{Edit}` — one variant (`theme.rs:567-570`), `Tone` — 7 (`theme.rs:533-543`), `SyntaxTone` — 8 (`theme.rs:546-556`). There is **no** role for charts/meters, no read-only role, no `on_danger`, no explicit `selection_fg/bg` pair.

`DESIGN.md:326-328` records that `accent_bg_subtle`, `error_bg` and `info` are declared but dead.

### 1.2 Capability downgrade today

- Detection: `ColorLevel::detect()` reads `NO_COLOR`, `COLORTERM`, `TERM` (`theme.rs:30-43`); labels at `theme.rs:45-52`.
- Conversion: `Theme::for_level` clones Junie and rewrites fields through a `macro_rules! map` listing **all 30 field names by hand** (`theme.rs:189-223`).
- Per-colour: `downgrade` early-returns for any non-`Color::Rgb` (`theme.rs:572-573`); `Ansi256` → `nearest_256` (colour cube vs grey ramp, `theme.rs:587-602`); `Ansi16` → `nearest_16` (luma + dominant channel, `theme.rs:604-641`); `Mono` → 4 grey buckets on the channel mean (`theme.rs:578-583`).

Two structural defects follow from this:

1. **The macro list is the only registry of "what is a colour".** A new token added to `Theme` compiles fine and is silently skipped by every downgrade. Nothing tests exhaustiveness (`theme.rs:648-655` only checks `accent`, `error`, `canvas`).
2. **`for_level` can only downgrade Junie.** Its first statement is `let mut t = Self::junie();` (`theme.rs:184`). There is no `fn downgrade(theme: &Theme, level)` for a user-supplied theme, so §15 scenario 11 is not reachable today.
3. **Mono collapses distinct states.** `GREEN 0x48e054` has channel mean 126 → `Color::Gray`; `RED 0xe44545` has mean 122 → `Color::Gray`. Accent and error render identically, so state legibility depends entirely on glyph/modifier rules that live in the widgets, not in the theme (`DESIGN.md:321-327` states the intent; nothing enforces it).

### 1.3 Where widgets assume Junie-specific fields

- **`lift()` is a hard-coded plane ladder keyed by colour equality** (`theme.rs:362-372`): `canvas→surface_elevated`, `surface|surface_elevated→surface_overlay`, `field→field_hover`, *else* `popover`. Any theme with two equal planes, a light theme (where "lift" must darken), or a theme with a different number of planes gets wrong results. It is called from `theme.rs:342, 394, 427`, `list.rs:262`, `tabs.rs:360-362, 422, 334, 468, 482`, `statusbar.rs:282`, `progress.rs:289, 451`.
- **`backdrop()` matches raw colour equality against nine specific tokens** (`theme.rs:297-321`) — the dim algorithm is Junie's ladder encoded as `if c == self.text_primary || c == self.accent …`.
- **`button()` hard-codes one recipe per `ButtonKind`** (`theme.rs:387-451`), including literal choices such as "secondary is always `surface_overlay`, hover is always `popover`" (`theme.rs:412-415`) regardless of the container surface. There is no way to add a variant.
- **Widgets reach into token fields directly rather than through a resolver**, e.g. `tabs.rs:273` (`border_subtle`), `tabs.rs:394-399` (`text_secondary`/`text_muted`), `tabs.rs:408` (`accent`), `tabs.rs:412` (`error`), `tabs.rs:415` (`warning`), `tabs.rs:430-434` (`border_strong` vs `accent`); `list.rs:262-268, 292`; `input.rs:318, 322, 394-395, 421`; `button.rs:129, 149, 158`; `statusbar.rs:270` (status bar plane is unconditionally `surface_elevated`), `statusbar.rs:278` (chip = `surface_overlay`); `progress.rs:64-67, 71-72, 368`.
- **Meter colours and thresholds live in the widget, not the theme**: `METER_LOW_MAX = 59`, `METER_MEDIUM_MAX = 84` (`progress.rs:86-87`) and `level_color` mapping Low→`text_secondary`, Medium→`warning`, High→`error` (`progress.rs:195-199`). §15's "chart or meter roles" have no token home.

### 1.4 Hard-coded glyphs, dimensions, spacing

Glyphs: focus bar `"▎"` (`button.rs:143`, `list.rs:255`, `input.rs:338`, `panel.rs:92`); markers `"›"`/`"✓"` (`list.rs:256-260`); toggle `●`/`○` (`button.rs:138, 151`); tab close `"×"` (`tabs.rs:426`), overflow `‹N`/`N›` (`tabs.rs:324-333, 458-461`), new-tab `" + "` (`tabs.rs:485`), rules `"─"`/`"━"` (`tabs.rs:270-273, 437`; `progress.rs:70-73, 265-267, 368-370`); dirty `"•"` (`tabs.rs:415`), error `"!"` (`tabs.rs:412`, `input.rs:421`, `progress.rs:51`), done `"✓"`/paused `"‖"` (`progress.rs:49-52`); mask `"•"` (`input.rs:141`); ellipsis `"…"` (`input.rs:398, 409`); spinner `SPINNER` 10-frame const (`progress.rs:8-12`).

Dimensions/spacing: card inset `Margin::new(2,1)` (`panel.rs:89`); framed inner `+2 / -3` (`panel.rs:116-122`); dialog widths `54` / `66` (`dialog.rs:68, 134`), inset `Margin::new(3,2)` (`dialog.rs:387`), height formula `2+1+1+1+body+1+1+1` (`dialog.rs:187-188`), code preview cap `6` (`dialog.rs:431`); `TextInput::HEIGHT = 3` (`input.rs:184`); button width `label+2 (+2)` (`button.rs:62-65`); tab width formula (`tabs.rs:240-255`); status bar `GAP = 3`, `EDGE = 1` (`statusbar.rs:119-120`), `STATUS_METER_TRACK = 10` (`statusbar.rs:42`); list "hide meta below 12 cells" (`list.rs:278`); progress "track < 6 cells → percent only" (`progress.rs:55-59`); app `MIN_WIDTH/MIN_HEIGHT = 72/20` (`app.rs:20-21`), sidebar `24`/`19` (`app.rs:753-754`), inspector `30` (`app.rs:756`).

Cadence: press flash `140 ms` (`app.rs:328, 381`), animation tick `80 ms` / idle `400 ms` (`app.rs:318-324`), status timeout `4 s` (`app.rs:388`).

None of these are theme data. `DESIGN.md:426-437` defines them as *tokens*; the code defines them as literals in ~15 files.

### 1.5 How applications pass raw background colours

Every non-container component's render signature ends in `bg: Color`:
`Button::render(..., bg: Color)` (`button.rs:101-107`), `ListBox::render` (`list.rs:207`), `Tabs::render` (`tabs.rs:258`), `TextInput::render` (`input.rs:263-269`), `ScrollPanel::render(..., bg: Color, style_line: fn(&Theme,&str)->Style)` (`panel.rs:257-264`), `Meter::render` (`progress.rs:220`), `render_bar` (`progress.rs:23-31`), `render_meter` (`progress.rs:326-334`), `render_indeterminate` (`progress.rs:342-348`), `render_spinner` (`progress.rs:377`).

The caller obtains the colour from the container and threads it manually — showcase pattern (`bin/showcase/pages/buttons.rs:76-78, 94`):

```rust
let panel = Panel::card(Some("Playground")).meta("hover · click · Tab · Enter / Space");
let bg = panel.bg(t);            // panel.rs:69-77
let inner = panel.render(rows[0], buf, t);
…
self.buttons[i].render(r, buf, ctx, bg);
```

`Panel` additionally exposes a public raw escape hatch `bg_override: Option<Color>` (`panel.rs:35, 70-72`). `Dialog` sidesteps the parameter by hard-coding `let bg = t.surface_elevated;` (`dialog.rs:378`) — a third, inconsistent convention. Showcase pages also construct styles directly from tokens (`buttons.rs:170-174` calls `t.button(...)`/`t.gutter(...)` and paints `" Label "` itself).

### 1.6 Additional facts that constrain the architecture

- **Render performs semantic mutation.** `TextInput::render` commits an edit when it observes focus loss (`input.rs:282-286`); `Dialog::render` flips `last.disabled` based on the acknowledgement text (`dialog.rs:465-470`); `Tabs::render` mutates `self.first`/`self.fit` (`tabs.rs:291-313`); `ScrollPanel::render` calls `jump_end()` when following (`panel.rs:290-291`). §11 of the goal forbids all four categories.
- **Event signatures disagree**: `Button::on_key -> (Outcome, bool)` (`button.rs:72`), `Tabs::on_key -> (Outcome, Option<TabEvent>)` (`tabs.rs:173`), `ListBox::on_key -> Outcome` (`list.rs:118`), `TextInput::on_key -> (Outcome, Option<InputEvent>)` (`input.rs:188`), `Dialog::on_key(&mut self, key, &mut Focus, &FocusRing) -> Outcome` (`dialog.rs:207-212` — mutates focus inside the handler).
- **`owns`/`locate` routing** is per-component and leaks to callers: `list.rs:186-193`, `tabs.rs:122-133`.
- **Identity is positional**: `ListBox::row_id(i) = id.child(i)` (`list.rs:82-84`), `Tabs::tab_id(i)` / `close_id(i)` (`tabs.rs:106-111`), and `TabEvent` carries `usize` (`tabs.rs:70-75`); `Tabs::remove` shifts every subsequent index (`tabs.rs:152-163`). Scenario E cannot hold.
- **Collections own duplicated strings**: `ListItem { label: String, meta: Option<String> }` (`list.rs:13-17`, `ListItem::new` copies at `list.rs:21`), `TabItem` likewise (`tabs.rs:19-29`). Scenario D forces a full owned rebuild.
- **Closed dialog body**: `enum DialogBody { Text, Input, Facts }` (`dialog.rs:18-28`) — explicitly prohibited by goal §14.
- **Extension via bare `fn` pointers**: `TextInput::validator: Option<fn(&str)->Option<String>>` (`input.rs:36, 105-108`), `ScrollPanel::render(style_line: fn(&Theme,&str)->Style)` (`panel.rs:263`). Neither can capture.
- **Public frame-local geometry**: `Button::area` (`button.rs:23`), `Tabs::areas` (`tabs.rs:60`), `ListBox::area` (`list.rs:54`), `TextInput::area` (`input.rs:33`), `Dialog::area` (`dialog.rs:52`).
- **Applications hand-run the interaction machine**: showcase owns `hover/pressed/mouse/hover_suppressed/flash` (`app.rs:202-212`), the full mouse state machine (`app.rs:576-694`), nav child-id decoding (`app.rs:696-698`), dialog focus save/restore (`app.rs:417-443`), and ring revalidation during render (`app.rs:723-732`). Pages scan for their own widget by id (`pages/buttons.rs:203, 213`).
- Infrastructure that is already right and should be preserved conceptually: frame-rebuilt focus ring with a barrier (`focus.rs:16-29`), last-registered-wins hit registry with barrier and scroll-only regions (`hit.rs:30-81`), `Interaction` snapshot with `focus_hidden`/`hover_suppressed` (`ctx.rs:17-41`), `VisualState` (`ctx.rs:44-54`), `RenderCtx::control/clickable/scrollable/begin_modal` (`ctx.rs:86-131`).

---

## 2. Component model (goal §9.1)

### 2.1 Option scoring against the 12 criteria

Scale: ++ strong / + adequate / ~ neutral / − weak / −− disqualifying.

| Criterion | A. Retained objects (today) | B. Stateless view + external state | C. Stateful widget (owns everything) | D. Behavior controller + renderer | E. Generic composition | F. Trait objects | G. Enum composition | H. Render closures | **I. Hybrid (recommended)** |
|---|---|---|---|---|---|---|---|---|---|
| Normal-use ergonomics | − (app runs the machine) | + | ++ | − (two objects per control) | + | + | ~ | − | **++** |
| Advanced customization | − | + | − | ++ | ++ | + | −− (closed) | ++ | **++** |
| Type safety | + | ++ | + | ++ | ++ | − (erased actions) | ++ | + | **++** |
| Borrowed data | −− (`String` items) | ++ | −− | ++ | ++ | − (`'static` pressure) | ~ | ++ | **++** |
| Lifetime complexity | + (none) | ~ | ++ | ~ | − (params spread) | + | + | ~ | **~ (confined to views)** |
| Heterogeneous composition | − | − | − | − | −− | ++ | + | ++ | **+ (`dyn` only where needed)** |
| Testability | − (render mutates) | ++ | − | ++ | + | ~ | + | ~ | **++** |
| Readability | + | + | + | ~ | − | ~ | + | − | **+** |
| Dynamic collections | −− (index ids) | + | − | + | + | ~ | − | ++ | **++ (keys)** |
| Performance | + | ++ | + | ++ | ++ | − (boxing/frame) | + | + | **++** |
| App migration | ~ | + | + | − | ~ | + | − | − | **+** |
| Component-author XP | ~ | + | ~ | + | − | + | −− | + | **++** |

Rejections, with reasons: **A** cannot express borrowed rows or stable keys and pushes routing into apps. **C** (component owns application data) reproduces `ListItem: String` and blocks Scenario D. **F** as the *primary* model costs a `Box` per node per frame and erases the action type (goal §10: "Do not make every component a trait object merely to support heterogeneous containers"). **G** is a closed set — forbidden by §16. **D** as a universal rule doubles the type count for a `Button`. **E** as a universal rule spreads generic parameters into every application signature — forbidden by §10.

### 2.2 Recommendation (inference)

**One hybrid with three fixed layers:**

1. **Durable interaction state** = concrete, caller-owned `XState` structs (`ListState`, `TextInputState`, `TabsState`). No lifetimes, `Debug`, serializable-shaped, unit-testable without a buffer. Stateless controls (`Button`, `Panel`, `StatusBar`, `Progress`) have no state struct.
2. **Per-frame view/props** = short-lived borrowing structs (`Button<'a>`, `List<'a, T, …>`) built with consuming builders and consumed by `show`.
3. **Two-phase frame with pre-resolved intents.** The runtime resolves raw input against the *previous* frame's hit registry and focus ring into a small `Intents` queue keyed by `Id` before any drawing. Each view's `update` consumes the intents addressed to its id and returns a typed action; `draw` only paints and registers. `show = update + draw`.

Phase separation is what satisfies §11: `update` never touches the buffer and is callable headless; `draw` never mutates semantics. Re-drawing an unchanged frame is a semantic no-op because the intent queue is already drained (a §25.2 conformance test).

Trait objects appear in exactly two places: an object-safe `Content` for heterogeneous child lists, and `&mut dyn FnMut` fallbacks in slot APIs. Nothing else is boxed.

### 2.3 Concrete type sketches

**Shared primitives**

```rust
// ---- identity -------------------------------------------------------------
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(u64);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKey { Index(usize), Num(u64), Hash(u64) }

impl Id {
    pub const fn root(path: &'static str) -> Id;      // `id!("orders.list")`
    pub const fn part(self, p: Part) -> Id;           // typed sub-part, no strings
    pub fn item(self, k: ItemKey) -> Id;              // stable per-item child
}
impl ItemKey {
    pub const fn index(i: usize) -> Self;             // documented as unstable under reorder
    pub const fn num(n: u64) -> Self;
    pub fn text(s: &str) -> Self;                     // FNV-1a of the bytes
}
#[macro_export] macro_rules! id { ($p:literal) => { $crate::Id::root(concat!(module_path!(), "::", $p)) } }

// debug-build collision detection; no cost in release
pub struct IdNames;                                    // Id -> &str, populated by `id!`
impl IdNames { pub fn name(id: Id) -> Option<&'static str>; }

// ---- interaction ----------------------------------------------------------
bitflags::bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct StateFlags: u16 {
        const FOCUSED = 1<<0; const HOVERED = 1<<1; const PRESSED = 1<<2;
        const SELECTED = 1<<3; const DISABLED = 1<<4; const READONLY = 1<<5;
        const ERROR = 1<<6; const WARNING = 1<<7; const BUSY = 1<<8;
        const EDITING = 1<<9; const ACTIVE = 1<<10;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Surface { Canvas, Raised, Elevated, Overlay, Popover, Field, FieldHover }

// ---- uniform response -----------------------------------------------------
#[must_use]
pub struct Response<A = ()> {
    id: Id,
    state: StateFlags,
    area: Rect,
    action: Option<A>,
    changed: bool,
}
impl<A> Response<A> {
    pub fn id(&self) -> Id;
    pub fn state(&self) -> StateFlags;
    pub fn area(&self) -> Rect;
    pub fn action(self) -> Option<A>;
    pub fn changed(&self) -> bool;
    pub fn focused(&self) -> bool;
    pub fn hovered(&self) -> bool;
    pub fn map<B>(self, f: impl FnOnce(A) -> B) -> Response<B>;
}
impl Response<Activated> { pub fn activated(&self) -> bool; }
```

`Response` replaces the four incompatible return shapes at `button.rs:72`, `tabs.rs:173`, `list.rs:118`, `input.rs:188`.

**The `Ui` context** (successor to `RenderCtx`, `ctx.rs:56-132`)

```rust
pub struct Ui<'a> {
    buf:     &'a mut Buffer,
    theme:   &'a Theme,
    styles:  StyleStack<'a>,      // scoped overlays, no allocation per frame
    surface: Surface,             // replaces every `bg: Color` parameter
    inter:   Interaction,         // focus/hover/pressed snapshot (ctx.rs:17-41, kept)
    intents: &'a mut Intents,     // pre-resolved semantic input, drained by `update`
    reg:     &'a mut Registry,    // hit regions + focus ring + scroll regions
    out:     &'a mut FrameOut,    // cursor, overlay queue, hint layers
}

impl<'a> Ui<'a> {
    // --- surface inheritance (replaces raw bg) ---
    pub fn surface(&self) -> Surface;
    pub fn bg(&self) -> Color;                                  // theme.bg(self.surface)
    pub fn with_surface<R>(&mut self, s: Surface, f: impl FnOnce(&mut Ui) -> R) -> R;
    pub fn raised(&self) -> Surface;                            // semantic lift, see §3.6

    // --- author-level API (public, in `tui::author`) ---
    pub fn flags(&self, id: Id) -> StateFlags;
    pub fn style(&self, f: Family, v: Variant, p: Part, s: StateFlags) -> Resolved;
    pub fn glyph(&self, g: GlyphRole) -> &str;
    pub fn space(&self, s: SpaceToken) -> u16;
    pub fn register_control(&mut self, id: Id, area: Rect, k: Focusability);
    pub fn register_click(&mut self, id: Id, area: Rect);
    pub fn register_scroll(&mut self, id: Id, area: Rect);
    pub fn set_cursor(&mut self, pos: Position);
    pub fn take_intents(&mut self, id: Id) -> IntentIter<'_>;

    // --- scopes ---
    pub fn with_styles<R>(&mut self, o: &Overlay, f: impl FnOnce(&mut Ui) -> R) -> R;
    pub fn focus_scope<R>(&mut self, id: Id, m: ScopeMode, f: impl FnOnce(&mut Ui) -> R) -> R;
    pub fn overlay(&mut self, id: Id, anchor: Anchor, f: impl FnOnce(&mut Ui, Rect));

    // --- testing ---
    pub fn headless(theme: &'a Theme, size: Size) -> UiOwned;   // update() without a terminal
}
```

**Button** (stateless, replaces `button.rs:14-164`)

```rust
pub struct Button<'a> {
    id: Id,
    label: &'a str,
    variant: Variant,                 // Variant::PRIMARY … or a user-registered one
    density: Option<Density>,
    disabled: bool,
    status: Status<'a>,               // Idle | Busy | Error(&str) | …
    toggle: Option<bool>,
    leading: Option<GlyphRole>,
    patch: Option<&'a StylePatch>,
    part_patches: &'a [(Part, StylePatch)],
}

#[derive(Clone, Copy, PartialEq, Eq)] pub struct Activated;

impl<'a> Button<'a> {
    pub fn new(id: Id, label: &'a str) -> Self;                 // default variant from the recipe
    pub fn variant(self, v: Variant) -> Self;
    pub fn density(self, d: Density) -> Self;
    pub fn disabled(self, yes: bool) -> Self;
    pub fn status(self, s: Status<'a>) -> Self;                 // Busy ⇒ spinner + no activation
    pub fn toggle(self, on: bool) -> Self;
    pub fn leading(self, g: GlyphRole) -> Self;
    pub fn patch(self, p: &'a StylePatch) -> Self;              // per-instance
    pub fn patch_part(self, ps: &'a [(Part, StylePatch)]) -> Self;

    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::MARKER, Part::LABEL];

    pub fn measure(&self, ui: &Ui) -> Size;                     // min + preferred
    pub fn update(&self, ui: &mut Ui) -> Option<Activated>;     // no buffer writes
    pub fn draw(&self, ui: &mut Ui, area: Rect, act: Option<Activated>);
    pub fn show(self, ui: &mut Ui, area: Rect) -> Response<Activated> {
        let a = self.update(ui); self.draw(ui, area, a);
        Response::new(self.id, ui.flags(self.id), area, a)
    }
}
```

Call site (compare `pages/buttons.rs:186-223` — the whole `handle` disappears):

```rust
if Button::new(id!("run"), "Run task").variant(Variant::PRIMARY)
        .show(ui, r).activated() {
    self.run_task();
}
```

**TextInput with a caller-owned value** (replaces `input.rs:19-446`)

```rust
#[derive(Debug, Default, Clone)]
pub struct TextInputState {
    editing: bool,
    draft: String,        // in-flight edit; never written back until commit
    cursor: usize,        // byte offset
    selection: Option<Range<usize>>,
    scroll: usize,
    error: Option<String>,
}
impl TextInputState {
    pub fn is_editing(&self) -> bool;
    pub fn begin(&mut self, current: &str);
    pub fn commit(&mut self, value: &mut String) -> Result<(), FieldError>;
    pub fn cancel(&mut self);
    pub fn set_error(&mut self, e: Option<FieldError>);
    pub fn clear_secret(&mut self);                    // zeroes draft + snapshot
}
impl std::fmt::Debug for TextInputState { /* redacts `draft` when `secret` */ }

pub enum TextAction { Changed, Committed, Cancelled, MovedNext, MovedPrev }

pub struct TextInput<'a, V: Validate = NoValidate> {
    id: Id,
    value: &'a mut String,                             // caller owns the value
    state: &'a mut TextInputState,
    label: Option<&'a str>,
    help: Option<&'a str>,
    placeholder: Option<&'a str>,
    required: bool, read_only: bool, disabled: bool,
    secret: Option<SecretPolicy>,                      // mask + reveal_tail
    validate: V,                                       // closure or type, not `fn`
    patch: Option<&'a StylePatch>,
}

pub trait Validate { fn check(&self, s: &str) -> Result<(), FieldError>; }
impl<F: Fn(&str) -> Result<(), FieldError>> Validate for F { … }   // closures welcome

impl<'a, V: Validate> TextInput<'a, V> {
    pub fn new(id: Id, value: &'a mut String, state: &'a mut TextInputState) -> TextInput<'a, NoValidate>;
    pub fn label(self, l: &'a str) -> Self;
    pub fn required(self, yes: bool) -> Self;
    pub fn read_only(self, yes: bool) -> Self;
    pub fn secret(self, p: SecretPolicy) -> Self;
    pub fn validate<W: Validate>(self, v: W) -> TextInput<'a, W>;
    pub const PARTS: &'static [Part] =
        &[Part::CONTAINER, Part::GUTTER, Part::LABEL, Part::FIELD, Part::TEXT, Part::MARKER, Part::HELP];
    pub fn measure(&self, ui: &Ui) -> Size;            // uses design.size.field_height
    pub fn update(&mut self, ui: &mut Ui) -> Option<TextAction>;
    pub fn draw(&self, ui: &mut Ui, area: Rect);
    pub fn show(mut self, ui: &mut Ui, area: Rect) -> Response<TextAction>;
}
```

`update` is the **only** place that commits; focus loss produces an explicit `Intent::FocusLost` in the intent queue, so the commit is testable headless. This removes `input.rs:282-286`.

**List over borrowed domain rows with a custom row renderer**

```rust
#[derive(Debug, Default, Clone)]
pub struct ListState {
    cursor: Option<ItemKey>,
    chosen: Option<ItemKey>,
    checked: KeySet,
    anchor: Option<ItemKey>,
    scroll: ScrollState,
    mode: SelectMode,
}
impl ListState {
    pub fn cursor(&self) -> Option<ItemKey>;
    pub fn chosen(&self) -> Option<ItemKey>;
    pub fn checked(&self) -> impl Iterator<Item = ItemKey> + '_;
    pub fn select(&mut self, k: ItemKey);
    /// Drop cursor/selection entries whose keys vanished; defined focus transition.
    pub fn reconcile(&mut self, keys: impl Iterator<Item = ItemKey>) -> Reconciliation;
    /// Pure keyboard step, no buffer: the unit-test entry point.
    pub fn on_key(&mut self, k: &KeyEvent, len: usize) -> Option<ListAction>;
}

pub enum ListAction { Moved, Chose(ItemKey), Toggled(ItemKey), Activated(ItemKey), ToggledAll }

pub struct List<'a, T, K, R> {
    id: Id,
    state: &'a mut ListState,
    items: &'a [T],                    // borrowed domain rows — no cloning
    key: K,                            // Fn(&T) -> ItemKey
    row: R,                            // FnMut(&T, &mut RowUi<'_>)
    empty: Option<&'a str>,
    disabled: Option<&'a dyn Fn(&T) -> bool>,
    patch: Option<&'a StylePatch>,
}

/// Row-scoped painter: parts are pre-styled for this row's resolved state.
pub struct RowUi<'u> { /* … */ }
impl RowUi<'_> {
    pub fn flags(&self) -> StateFlags;
    pub fn marker(&mut self, g: GlyphRole);
    pub fn label(&mut self, s: &str);
    pub fn label_styled(&mut self, s: &str, p: &StylePatch);
    pub fn meta(&mut self, s: &str);            // dropped all-or-none, DESIGN.md:478
    pub fn columns(&mut self, widths: &[Constraint]) -> ColumnsUi<'_>;
    pub fn raw(&mut self) -> (&mut Buffer, Rect);   // last-resort escape hatch
}

impl<'a, T> List<'a, T, fn(&T) -> ItemKey, fn(&T, &mut RowUi)> {
    pub fn new(id: Id, state: &'a mut ListState) -> ListBuilder<'a, T>;
}
impl<'a, T, K, R> List<'a, T, K, R>
where K: Fn(&T) -> ItemKey, R: FnMut(&T, &mut RowUi<'_>) {
    pub fn items(self, items: &'a [T]) -> Self;
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> List<'a, T, K2, R>;
    pub fn row<R2: FnMut(&T, &mut RowUi<'_>)>(self, r: R2) -> List<'a, T, K, R2>;
    pub fn empty(self, text: &'a str) -> Self;
    pub const PARTS: &'static [Part] =
        &[Part::CONTAINER, Part::ROW, Part::GUTTER, Part::MARKER, Part::LABEL, Part::META,
          Part::TRACK, Part::THUMB, Part::EMPTY];
    pub fn show(self, ui: &mut Ui, area: Rect) -> Response<ListAction>;
}
```

Only visible rows invoke `R` (virtualization by construction), so a 1 M-row slice costs O(viewport).

```rust
let act = List::new(id!("orders"), &mut self.list)
    .items(&self.orders)                              // &[Order], borrowed
    .key(|o: &Order| ItemKey::num(o.id))
    .row(|o: &Order, r: &mut RowUi| {
        r.marker(GlyphRole::Chosen);
        r.label(&o.customer);                          // &str, no allocation
        r.meta(&o.total_display);
    })
    .show(ui, area)
    .action();
```

**Dynamic Tabs with stable keys** (replaces `tabs.rs:52-492`)

```rust
#[derive(Debug, Default, Clone)]
pub struct TabsState { active: Option<ItemKey>, cursor: Option<ItemKey>, first: Option<ItemKey> }
impl TabsState {
    pub fn active(&self) -> Option<ItemKey>;
    pub fn activate(&mut self, k: ItemKey);
    pub fn reconcile(&mut self, keys: impl Iterator<Item = ItemKey>) -> Reconciliation;
}

pub enum TabsAction { Activated(ItemKey), Close(ItemKey), New }

pub struct Tabs<'a, T, K, L> {
    id: Id, state: &'a mut TabsState, items: &'a [T],
    key: K, label: L,
    badge: Option<&'a dyn Fn(&T) -> TabBadge>,     // Dirty | Busy | Error | None
    prefix: Option<&'a dyn Fn(&T) -> Option<&str>>,
    closable: Option<&'a dyn Fn(&T) -> bool>,
    allow_new: bool,
    level: TabLevel,                               // Document | Nested (the "quiet" rule)
}
impl<'a, T, K, L> Tabs<'a, T, K, L>
where K: Fn(&T) -> ItemKey, L: Fn(&T) -> &str {
    pub const PARTS: &'static [Part] =
        &[Part::CONTAINER, Part::TAB, Part::LABEL, Part::PREFIX, Part::BADGE,
          Part::CLOSE, Part::RULE, Part::OVERFLOW, Part::NEW];
    pub fn show(self, ui: &mut Ui, area: Rect) -> Response<TabsAction>;
}
```

`show` calls `state.reconcile(items.iter().map(&key))` **before** drawing, so removal or reorder keeps the active document (Scenario E). Every action carries an `ItemKey`, never a `usize`.

**Composed dialog with arbitrary body** (replaces the closed `DialogBody`, `dialog.rs:18-28`)

```rust
pub struct Dialog<'a> {
    id: Id, title: &'a str,
    width: Option<u16>,                 // default from design.size.dialog_width
    dismiss: Dismiss,                   // Esc + click-outside policy
    initial_focus: Option<Id>,
}
pub enum DialogOutcome { Action(Id), Dismissed, Open }

pub struct DialogUi<'u, 'a> { /* … */ }
impl<'u, 'a> DialogUi<'u, 'a> {
    pub fn body<R>(&mut self, f: impl FnOnce(&mut Ui, Rect) -> R) -> R;   // ARBITRARY content
    pub fn description(&mut self, text: &str);
    pub fn actions(&mut self, f: impl FnOnce(&mut ActionRow<'_>));
    pub fn measured_body(&mut self, h: u16);                              // body height hint
}
pub struct ActionRow<'u> { /* right-aligned, design.space.inline gaps */ }
impl ActionRow<'_> {
    pub fn push(&mut self, b: Button<'_>) -> Response<Activated>;
    pub fn cancel(&mut self, b: Button<'_>) -> Response<Activated>;       // marks the Esc target
}

impl<'a> Dialog<'a> {
    pub fn new(id: Id, title: &'a str) -> Self;
    pub const PARTS: &'static [Part] =
        &[Part::BACKDROP, Part::CONTAINER, Part::BORDER, Part::TITLE, Part::BODY, Part::ACTIONS];
    pub fn show(self, ui: &mut Ui,
                f: impl FnOnce(&mut DialogUi<'_, '_>)) -> DialogOutcome;

    // convenience constructors built on exactly the same path
    pub fn confirm(id: Id, title: &'a str, text: &'a str, ok: &'a str) -> Confirm<'a>;
    pub fn destructive(id: Id, title: &'a str, text: &'a str, ok: &'a str) -> Confirm<'a>;
    pub fn prompt<'v>(id: Id, title: &'a str, v: &'v mut String, s: &'v mut TextInputState) -> Prompt<'a, 'v>;
    pub fn acknowledge(id: Id, title: &'a str, token: &'a str) -> Acknowledge<'a>;
}
```

```rust
match Dialog::new(id!("save"), "Unsaved changes").show(ui, |d| {
    d.body(|ui, area| {
        let (top, bottom) = area.split_v(3);
        TextInput::new(id!("name"), &mut self.name, &mut self.name_state)
            .label("Name").show(ui, top);
        Props::new(&self.facts).show(ui, bottom);
    });
    d.actions(|a| {
        a.cancel(Button::new(id!("cancel"), "Cancel").variant(Variant::SUBTLE));
        a.push(Button::new(id!("save"), "Save").variant(Variant::PRIMARY));
    });
}) {
    DialogOutcome::Action(id) if id == id!("save") => self.save(),
    DialogOutcome::Dismissed => self.close(),
    _ => {}
}
```

`Dialog::show` pushes the modal focus scope and hit barrier (`focus.rs:20`, `hit.rs:53`), saves and restores focus, dims the backdrop, and clamps to the terminal — none of that is application code any more (compare `app.rs:417-443`).

**Custom downstream component** (Scenario G — public author API only)

```rust
use tui::author::{Ui, Id, Part, StateFlags, Family, Variant, Focusability, Response};

pub const QUOTA: Family = Family::custom("app.quota");
pub const TRACK: Part = Part::custom("track");
pub const VALUE: Part = Part::custom("value");

pub struct Quota<'a> { id: Id, label: &'a str, pct: u8 }

impl<'a> Quota<'a> {
    pub fn new(id: Id, label: &'a str, pct: u8) -> Self { Self { id, label, pct } }
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, TRACK, VALUE, Part::LABEL];

    pub fn show(self, ui: &mut Ui, area: Rect) -> Response<()> {
        let mut st = ui.flags(self.id);
        if self.pct >= ui.theme().design.meter.medium_max { st |= StateFlags::WARNING; }
        let track = ui.style(QUOTA, Variant::DEFAULT, TRACK, st);
        let value = ui.style(QUOTA, Variant::DEFAULT, VALUE, st);
        let filled = area.width as u32 * self.pct as u32 / 100;
        for x in 0..area.width {
            let g = if (x as u32) < filled { ui.glyph(GlyphRole::RuleActive) }
                    else { ui.glyph(GlyphRole::RuleQuiet) };
            ui.paint(area.x + x, area.y, g, track.style);
        }
        ui.paint_str(area.x, area.y, self.label, value.style);
        ui.register_control(self.id, area, Focusability::Clickable);
        Response::new(self.id, st, area, None)
    }
}
```

The author registers a recipe once (`theme.define_family(QUOTA, …)`) or falls back to the default recipe for unknown families. No private import is required.

---

## 3. Theme representation (goal §9.2)

### 3.1 Comparison

| Option | Custom themes | Partial override | Per-instance | Generic spread | Dispatch cost | Verdict |
|---|---|---|---|---|---|---|
| Concrete data theme | ++ | + (struct update syntax) | −− | none | none | **adopt as the base** |
| Semantic token structs | ++ | ++ | −− | none | none | **adopt** |
| Component recipes | + | ++ (family/variant/part/state) | + | none | array index | **adopt** |
| Typed style patches | ~ | ++ | ++ | none | tiny merge | **adopt** |
| Theme trait | + | − (users implement 40 methods) | − | −− (`<T: Theme>` everywhere) | vtable per query | **reject** |
| Generic theme parameter | + | − | − | −− | none | **reject** |
| Theme trait object | + | − | − | + | vtable per query | **reject** |
| Scoped overlays | ~ | ++ (subtree) | ++ | none | stack walk ≤ 4 | **adopt** |

**Recommendation:** concrete `Theme` **data** + typed `Recipes` + `StylePatch` + a scoped overlay stack. No trait, no generic parameter, no dynamic dispatch on the hot path. This directly answers §9.2's "make custom themes and local overrides easy without spreading generics… or forcing dynamic dispatch everywhere". The README's "one `Theme` trait" suggestion is rejected: a trait forces every custom theme author to implement resolution logic, whereas the resolution logic is exactly the part that must stay uniform.

### 3.2 Semantic token set (covers every §15 role including syntax and meter)

```rust
pub const SURFACE_LEVELS: usize = 5;   // Canvas, Raised, Elevated, Overlay, Popover
pub const FG_STEPS: usize = 5;         // Primary, Secondary, Muted, Faint, Ghost

pub struct ColorTokens {
    // canvas & surfaces — an ORDERED ladder, not ad-hoc names
    pub surfaces: [Color; SURFACE_LEVELS],
    pub field: Color, pub field_hover: Color,

    // foreground hierarchy — an ORDERED ladder
    pub fg: [Color; FG_STEPS],
    pub on_accent: Color, pub on_danger: Color, pub on_surface_inverse: Color,

    // borders
    pub border_subtle: Color, pub border_strong: Color,

    // accent + focus
    pub accent: Color, pub accent_hover: Color, pub accent_pressed: Color, pub accent_tint: Color,
    pub focus: Color, pub focus_ring: Color,

    // selection / hover / elevation
    pub selection_bg: Color, pub selection_fg: Color,
    pub highlight_bg: Color, pub highlight_fg: Color,               // menu cursor (blue)
    pub highlight_danger_bg: Color, pub highlight_danger_fg: Color,

    // overlays
    pub backdrop_fg: Color, pub backdrop_bg: Color,

    // status roles
    pub danger: Color, pub danger_soft: Color, pub danger_tint: Color,
    pub warning: Color, pub warning_tint: Color,
    pub success: Color, pub info: Color,

    // disabled / read-only
    pub disabled_fg: Color, pub disabled_bg: Color, pub read_only_fg: Color,

    pub syntax: SyntaxTokens,
    pub meter:  MeterTokens,
}

pub struct SyntaxTokens {
    pub keyword: Color, pub ident: Color, pub string: Color, pub number: Color,
    pub operator: Color, pub punct: Color, pub comment: Color, pub plain: Color,
    pub type_name: Color, pub function: Color, pub constant: Color,
    pub invalid: Color, pub deprecated: Color,
    pub match_bg: Color, pub match_current_bg: Color, pub bracket_match: Color,
    pub diagnostic_error: Color, pub diagnostic_warning: Color, pub diagnostic_info: Color,
}

pub struct MeterTokens {
    pub low: Color, pub medium: Color, pub high: Color,
    pub track: Color, pub fill_rest: Color,
    pub stale: Color, pub unknown: Color,
    pub series: [Color; 6],           // chart / categorical roles
}
```

Rationale for arrays: `Surface`/`FgStep` become indices, so "one plane up" is `min(i+1, LAST)` instead of the colour-equality chain at `theme.rs:362-372`. That single change is what makes a light theme, a high-contrast theme, or a theme with duplicate plane colours behave correctly.

### 3.3 Colour capability downgrade for arbitrary themes

```rust
impl ColorTokens {
    /// Exhaustive by construction: a new field is a compile error here.
    pub fn map_colors(&self, f: &mut impl FnMut(Color) -> Color) -> ColorTokens {
        let ColorTokens {
            surfaces, field, field_hover, fg, on_accent, on_danger, on_surface_inverse,
            border_subtle, border_strong, accent, accent_hover, accent_pressed, accent_tint,
            focus, focus_ring, selection_bg, selection_fg, highlight_bg, highlight_fg,
            highlight_danger_bg, highlight_danger_fg, backdrop_fg, backdrop_bg,
            danger, danger_soft, danger_tint, warning, warning_tint, success, info,
            disabled_fg, disabled_bg, read_only_fg, syntax, meter,
        } = self;                                   // ← exhaustive destructure
        ColorTokens {
            surfaces: surfaces.map(|c| f(c)),
            fg: fg.map(|c| f(c)),
            field: f(*field), /* … every field … */
            syntax: syntax.map_colors(f),
            meter: meter.map_colors(f),
        }
    }
}

pub fn downgrade_theme(t: &Theme, level: ColorLevel) -> Theme {
    let mut out = t.clone();
    out.capability.color = level;
    out.color = t.color.map_colors(&mut |c| downgrade_color(c, level));
    if level == ColorLevel::Mono { out.recipes.apply_mono_fallbacks(); }
    out
}
```

This replaces `theme.rs:189-223` (hand-written macro list) and `theme.rs:184` (Junie-only). `downgrade_color` keeps the existing `nearest_256` / `nearest_16` / mono bucketing (`theme.rs:587-641`), which are sound.

`apply_mono_fallbacks` is the mechanism that satisfies §15 item 12: at `Mono` it forces, per state, a modifier or glyph that is *not* colour — `FOCUSED ⇒ BOLD + gutter glyph`, `SELECTED ⇒ marker glyph`, `ERROR ⇒ trailing "!" + UNDERLINED`, `DISABLED ⇒ no gutter, no marker`, `EDITING ⇒ UNDERLINED + cursor`, `WARNING ⇒ "•"`. That codifies `DESIGN.md:321-327` in data rather than in prose.

### 3.4 Recipe / part / state resolution

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)] pub struct Family(u16);
#[derive(Clone, Copy, PartialEq, Eq, Hash)] pub struct Variant(u16);
#[derive(Clone, Copy, PartialEq, Eq, Hash)] pub struct Part(u16);

impl Family  { pub const BUTTON: Family = Family(0); /* … */ pub const fn custom(name: &'static str) -> Family; }
impl Variant { pub const DEFAULT: Variant = Variant(0); pub const PRIMARY: Variant = Variant(1);
               pub const SECONDARY: Variant = Variant(2); pub const SUBTLE: Variant = Variant(3);
               pub const DANGER: Variant = Variant(4); pub const fn custom(name: &'static str) -> Variant; }
impl Part    { pub const CONTAINER: Part = Part(0); pub const BORDER: Part = Part(1);
               pub const GUTTER: Part = Part(2); pub const MARKER: Part = Part(3);
               pub const LABEL: Part = Part(4);  pub const META: Part = Part(5);
               pub const ICON: Part = Part(6);   pub const BODY: Part = Part(7);
               pub const ACTIONS: Part = Part(8);pub const TRACK: Part = Part(9);
               pub const THUMB: Part = Part(10); pub const ROW: Part = Part(11); /* … */
               pub const fn custom(name: &'static str) -> Part; }

pub struct StateRule { pub when: StateFlags, pub patch: StylePatch }

pub struct PartRecipe {
    pub base: StylePatch,
    pub states: SmallVec<[StateRule; 8]>,     // matched when `when ⊆ live`, more bits wins
    pub glyph: Slot<GlyphRole>,
    pub size:  Slot<u16>,
}

pub struct Recipe {
    pub default_variant: Variant,
    pub parts: PartMap<PartRecipe>,                        // dense, part index
    pub variants: SmallVec<[(Variant, PartMap<PartRecipe>); 6]>,
}

pub struct Recipes { by_family: Box<[Recipe]> }            // index = Family.0
```

Resolution — O(1), no allocation, no hashing on the hot path:

```rust
pub struct Resolved { pub style: Style, pub glyph: Option<GlyphRole>, pub size: Option<u16> }

impl Ui<'_> {
    pub fn style(&self, f: Family, v: Variant, p: Part, s: StateFlags) -> Resolved {
        let r = &self.theme.recipes[f];
        let mut acc = r.parts[p].base;                                  // 1. family base
        if let Some(vp) = r.variant(v) { acc = acc.merge(vp[p].base); } // 2. variant
        for rule in r.parts[p].states.iter()                            // 3. state rules
            .filter(|k| s.contains(k.when))
            .sorted_by_key(|k| k.when.bits().count_ones()) { acc = acc.merge(rule.patch); }
        for ov in self.styles.iter() {                                  // 4-5. global + scoped
            if let Some(p2) = ov.lookup(f, v, p, s) { acc = acc.merge(p2); }
        }
        if let Some(inst) = self.styles.instance() { acc = acc.merge(*inst); } // 6. per-instance
        acc.resolve(&self.theme.color, self.surface, self.theme.capability)
    }
}
```

Note the last line: `StylePatch` stores **roles**, not `Color`. Resolution binds roles to colours *at the end*, against the live theme, the current `Surface` and the colour capability — so a user patch written once works under every theme and every colour level. This is the single most important property of the design.

### 3.5 `StylePatch` merge semantics

```rust
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Slot<T> { #[default] Inherit, Set(T), Clear }

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct StylePatch {
    pub fg: Slot<Role>,
    pub bg: Slot<Role>,
    pub underline: Slot<Role>,
    pub add: Modifier,
    pub remove: Modifier,
    pub glyph: Slot<GlyphRole>,
    pub size: Slot<u16>,
}

impl StylePatch {
    /// `over` wins where it speaks; `Inherit` is the identity element.
    pub fn merge(self, over: StylePatch) -> StylePatch {
        StylePatch {
            fg: over.fg.over(self.fg),
            bg: over.bg.over(self.bg),
            underline: over.underline.over(self.underline),
            add: (self.add | over.add) - over.remove,
            remove: (self.remove | over.remove) - over.add,
            glyph: over.glyph.over(self.glyph),
            size: over.size.over(self.size),
        }
    }
}
impl<T: Copy> Slot<T> {
    fn over(self, base: Slot<T>) -> Slot<T> {
        match self { Slot::Inherit => base, other => other }   // Set and Clear both win
    }
}
```

Laws (each becomes a unit test):
- **identity**: `merge(x, StylePatch::default()) == x`
- **absorption**: `merge(x, Set(v)).fg == Set(v)`
- **clear**: `merge(Set(a), Clear).fg == Clear` → resolves to "no colour", i.e. the surface's inherited fg / terminal default, never a panic
- **associativity**: `merge(merge(a,b),c) == merge(a,merge(b,c))`
- **modifier symmetry**: a later `remove(BOLD)` beats an earlier `add(BOLD)`; a later `add(BOLD)` beats an earlier `remove(BOLD)`

### 3.6 Override APIs and precedence

```rust
// (4) global family override — a whole component family, everywhere
let theme = Theme::junie().override_family(Family::BUTTON, |r| {
    r.part(Part::CONTAINER).set_bg(Role::SurfaceOverlay);
    r.when(StateFlags::FOCUSED).part(Part::GUTTER).set_fg(Role::Warning);
});

// (5) variant override + custom variant
const GHOST: Variant = Variant::custom("ghost");
let theme = theme
    .override_variant(Family::BUTTON, Variant::PRIMARY, |r| { r.part(Part::LABEL).add(Modifier::BOLD); })
    .define_variant(Family::BUTTON, GHOST, |r| {
        r.part(Part::CONTAINER).clear_bg();
        r.part(Part::LABEL).set_fg(Role::FgSecondary);
        r.when(StateFlags::HOVERED).part(Part::LABEL).set_fg(Role::FgPrimary);
    });

// (6) scoped override — a subtree only, no clone of the theme
static DANGER_ZONE: Overlay = Overlay::new()
    .family(Family::BUTTON, Variant::SECONDARY, Part::LABEL, StylePatch::new().set_fg(Role::Danger));
ui.with_styles(&DANGER_ZONE, |ui| { self.danger_pane(ui, area); });

// (7) per-instance
static WIDE: StylePatch = StylePatch::new().set_size(20);
Button::new(id!("save"), "Save").patch(&WIDE).show(ui, r);

// (8) per-instance, one logical part
static MARKER: [(Part, StylePatch); 1] =
    [(Part::MARKER, StylePatch::new().set_fg(Role::Warning).set_glyph(GlyphRole::Dirty))];
List::new(id!("rows"), &mut st).patch_part(&MARKER) /* … */;
```

**Precedence (lowest → highest, documented and tested):**

1. family recipe base part style
2. variant delta (the variant selected on the instance, else `recipe.default_variant`)
3. state rules, ordered by specificity (`when.count_ones()`), ties broken by declaration order
4. theme-level global family/variant override
5. scope overlay stack, outermost → innermost
6. per-instance `patch` / `patch_part`

Invariants: overlays are borrowed, never cloned, and never mutate the `Theme` (a §25.2 conformance test asserts the theme is byte-identical before and after a scoped render). Levels 4–6 are all `StylePatch`, so the merge law above governs the whole chain.

### 3.7 Where each concern lives (explicit decision)

| Concern | Home | Reason |
|---|---|---|
| colour roles (all of §15) | `ColorTokens` | one place the whole system is re-coloured; the downgrade visitor covers it |
| spacing / padding | `DesignTokens.space` (`gutter, inline, gap, column_gap, form_gap, card_inset, frame_inset, dialog_inset, tree_indent`) | `DESIGN.md:426-437` already names these; today they are literals in ~10 files |
| dimensions | `DesignTokens.size` (`field_height, tabs_height, dialog_width, dialog_width_wide, popup_max_rows, min_width, min_height`), overridable per recipe via `PartRecipe.size` | a variant may want a wider dialog without a new theme |
| glyphs | `DesignTokens.glyphs: GlyphSet` keyed by `GlyphRole` | an ASCII-only theme must be one substitution; mono fallbacks depend on it |
| border glyph sets | `DesignTokens.borders: BorderSet` (rounded/square/ascii + quiet `─` / active `━` rules) | same |
| focus indicator | glyph in `GlyphSet::FocusBar`; *whether and where* it shows in the recipe's `Part::GUTTER` | separates "what the mark is" from "which parts wear it" |
| selection indicator | glyph in `GlyphSet::{Chosen, Checked}`; placement in the `Part::MARKER` recipe | same |
| scrollbar symbols | `GlyphSet::{ScrollTrack, ScrollThumb}`; tone in the scrollbar recipe (`Part::TRACK`/`Part::THUMB`) | same |
| animation cadence | `DesignTokens.motion { spinner_frames, tick_ms, press_flash_ms, status_ms }` | today split between `progress.rs:8-12` and `app.rs:318-328, 388` |
| component density | `DesignTokens.density: Density` + per-instance `.density(…)` + per-variant size deltas | one global switch plus local escape |
| variant defaults | `Recipe.default_variant` per family | lets a theme make "subtle" the default button without touching call sites |
| meter thresholds | `DesignTokens.meter { low_max, medium_max }` | today consts at `progress.rs:86-87`; a product may tune them |

### 3.8 How surface inheritance replaces raw `bg`

Every `bg: Color` parameter (§1.5) disappears. `Ui` carries the current `Surface`; containers push it:

```rust
impl Theme {
    pub fn bg(&self, s: Surface) -> Color;           // index into color.surfaces / field
    pub fn raise(&self, s: Surface) -> Surface {     // semantic "one plane up"
        match s {
            Surface::Field => Surface::FieldHover,
            other => Surface::from_level((other.level() + 1).min(SURFACE_LEVELS - 1)),
        }
    }
}
impl Ui<'_> {
    pub fn with_surface<R>(&mut self, s: Surface, f: impl FnOnce(&mut Ui) -> R) -> R {
        let prev = std::mem::replace(&mut self.surface, s);
        let r = f(self); self.surface = prev; r
    }
}
// Panel::show pushes the surface it fills, so children inherit it:
Panel::card(id!("card")).title("Connections").show(ui, area, |ui, inner| {
    Button::new(id!("save"), "Save").show(ui, inner);   // no bg argument, ever
});
```

The only remaining public raw-colour surface is `Role::Custom(Color)` inside a `StylePatch` — the deliberate, documented escape hatch for a user who genuinely wants a literal, and it still passes through capability downgrade.

---

## 4. Composition (goal §9.4)

| Capability | Mechanism | Sketch |
|---|---|---|
| arbitrary content in panels/dialogs | slot closures (`FnOnce(&mut Ui, Rect)`), generic → no `Box` | `Panel::show(self, ui, area, body)`; `Dialog::show(self, ui, f)` |
| header/body/footer/actions | a slot object with one method per slot | `DialogUi::{body, description, actions}`; `Panel::{title, meta, badge, footer}` |
| custom rows/cells | `RowUi` / `CellUi` painters + `FnMut` renderers | `List::row(…)`, `Table::cell(…)`, `Grid::cell(…)` |
| wrap/decorate | nest `show` closures; `Response` reports the used `Rect` | `let r = inner.show(ui, area); badge(ui, r.area());` |
| override logical parts | `.patch_part(Part, …)` restyles; `.slot(Part, closure)` **replaces** | see below |
| higher-level components | a plain struct with a `show` method composed of primitives | `struct ConnectionCard<'a>` |
| reuse behavior, new presentation | the `*State` machines are public and buffer-free | `ListState::on_key` drives a custom renderer |
| borrowed content, no allocation | `&'a [T]` + accessor closures; only visible items are visited | `List::items(&self.orders)` |

**Part slots — the Rust translation of `asChild`/`Slot`:**

```rust
pub struct Slots<'a> { entries: &'a mut [(Part, &'a mut dyn FnMut(&mut Ui, Rect))] }

impl<'a, T, K, R> List<'a, T, K, R> {
    /// Replace the rendering of one logical part; the component still owns
    /// layout, hit registration, focus and state.
    pub fn slot(self, p: Part, f: &'a mut dyn FnMut(&mut Ui, Rect)) -> Self;
}
```

`&mut dyn FnMut` (not `Box`) keeps it allocation-free and non-`'static`. This is the only place `dyn` appears in a component's public surface, and it is opt-in.

**Higher-level component built from primitives (no privileged access):**

```rust
pub struct ConnectionCard<'a> {
    id: Id, conn: &'a Connection, state: &'a mut ConnectionCardState,
}
impl<'a> ConnectionCard<'a> {
    pub fn show(self, ui: &mut Ui, area: Rect) -> Response<ConnectionAction> {
        Panel::card(self.id.part(Part::CONTAINER))
            .title(&self.conn.name)
            .meta(&self.conn.host)
            .show(ui, area, |ui, inner| {
                let (top, actions) = inner.split_bottom(1);
                Props::new(&self.conn.facts).show(ui, top);
                let mut act = None;
                ActionRow::new(actions).show(ui, |a| {
                    if a.push(Button::new(self.id.part(Part::custom("test")), "Test")).activated() {
                        act = Some(ConnectionAction::Test);
                    }
                    if a.push(Button::new(self.id.part(Part::custom("connect")), "Connect")
                              .variant(Variant::PRIMARY)).activated() {
                        act = Some(ConnectionAction::Connect);
                    }
                });
                Response::new(self.id, ui.flags(self.id), inner, act)
            })
    }
}
```

**Reusing behaviour with different presentation** — `ListState` has no rendering dependency, so a bespoke renderer reuses cursor/selection/scroll/reconcile logic verbatim:

```rust
if let Some(a) = self.list.on_key(&key, self.rows.len()) { /* … */ }
for (i, row) in self.list.visible(self.rows.len()).enumerate() { my_paint(ui, row, i); }
```

---

## 5. Public API conventions (goal §10) — one named vocabulary

| Question | Convention (name) | Form |
|---|---|---|
| construction | **Id-first constructor** | `X::new(id, required…)`; alternates only for semantically different modes (`Dialog::destructive`) |
| configuration | **consuming builder** | `fn variant(self, v) -> Self`; no `with_` prefix, no `set_` on views |
| borrowed vs owned | **views borrow, state owns** | `X<'a>` holds `&'a` data; `XState` holds only interaction state |
| caller state | **`XState` parameter** | `X::new(id, &mut state)` or `.state(&mut state)`; caller stores it in the app struct |
| library state | **`Ui`-managed** | focus, hover, pressed, flash, cursor, hit regions, overlays, style stack |
| controlled value | **`&mut T` value** | `TextInput::new(id, &mut self.name, &mut self.name_state)`; the draft lives in state, the value is written on commit |
| uncontrolled | **`XState::value()`** | for throwaway fields the state can own a `String`; documented as the exception |
| variants & sizes | **`.variant(Variant)` / `.density(Density)`** | typed newtypes, extensible via `Variant::custom` |
| disabled / read-only | **`.disabled(bool)` / `.read_only(bool)`** | distinct: read-only stays in the focus ring, disabled does not |
| loading/busy/error/editing | **`Status` enum + `StateFlags`** | `.status(Status::Busy)`, `.status(Status::Error("…"))`; `EDITING` is owned by the state, never a prop |
| events | **`Response<XAction>`** | one shape everywhere; `.action()`, `.activated()`, `.changed()` |
| focus | **automatic registration** | `ui.register_control(id, area, Focusability)`; `.autofocus()`; `ui.focus_scope(id, ScopeMode::Trap, …)` |
| render | **`show(ui, area)`**, split into `update` + `draw` | `show` is the normal path; the split is the testing and no-side-effect guarantee |
| measure | **`measure(&self, ui, Constraints) -> Size`** | `Size { min: (u16,u16), preferred: (u16,u16) }` |
| parts | **`X::PARTS: &'static [Part]`** | documented per component; used by theming, overrides, event metadata, tests |
| local override | **`.patch(&StylePatch)` / `.patch_part(&[(Part, StylePatch)])`** | plus `.slot(Part, &mut dyn FnMut)` to replace a part outright |
| item renderer | **`.row(FnMut(&T, &mut RowUi))` / `.cell(…)`** | closures, never `fn` pointers |
| identity | **`.key(Fn(&T) -> ItemKey)`** | actions carry `ItemKey`; `Id::item(key)` derives child ids |
| errors | **`FieldError` / `LayoutError`** | typed, `Display` + `Error`; no panics on any interaction path |
| testing | **`Harness`** | `Harness::new(theme, size)`, `.key(…)`, `.click(id!("…"))`, `.actions()`, `.snapshot()`, `Ui::headless` |
| API layers | **`tui::*` (app authors) vs `tui::author::*` (component authors)** | the second is `pub` but curated and separately documented |

---

## 6. Package boundary (goal §9.5, §21)

### 6.1 Honest evaluation

A single package **already** enforces "applications consume only `pub` API": `src/bin/showcase/main.rs` is a separate crate that links `junie_tui` (`Cargo.toml:11-25`), so it cannot reach `pub(crate)` items today. The workspace's advantage is therefore **not** that claim. The real, mechanically checkable gains:

1. **Dependency direction is enforced by Cargo, not by grep.** App crates depend on `tui`; `tui` depends on nothing app-shaped. `cargo tree -p tui` is the §25.5 check for "generic library code does not depend on TablePro or Jackin domain modules" — impossible to violate by accident.
2. **Dev-dependency and feature isolation.** Today any dev-dependency an app test needs enters the library's package. In a workspace, `tui`'s own dep set stays minimal and auditable.
3. **Doc / lint scoping.** `#![deny(missing_docs)]` and `RUSTDOCFLAGS="-D warnings" cargo doc -p tui` apply to the library alone; app code does not have to carry rustdoc on every private screen struct.
4. **Publishability and semver checks** on the library independently of three binaries (`cargo package -p tui --dry-run`, `cargo-semver-checks`).
5. **Examples as genuine external consumers**: `crates/tui/examples/*.rs` build against exactly the published surface, and `cargo build -p tui --examples` is a gate.

**Recommendation:** move to a workspace, but keep the library split small — one library crate plus a testing crate. A four-way `core/theme/widgets/runtime` split would create cyclic pressure (recipes need `Part`, `Part` is used by components) for no enforcement benefit.

### 6.2 Proposed layout

```
Cargo.toml                       # [workspace] members, shared [workspace.dependencies]
crates/tui/                      # the library — the only publishable crate
  src/lib.rs                     #   pub use of the curated surface
  src/id.rs  src/event.rs  src/focus.rs  src/hit.rs  src/scroll.rs  src/text.rs
  src/layout/                    #   Constraints, split helpers, Size
  src/theme/                     #   tokens.rs palette.rs recipes.rs patch.rs downgrade.rs glyphs.rs
  src/theme/themes/junie.rs      #   the ONLY place literal palette values live (plus one contrast theme)
  src/ui/                        #   Ui, Registry, Intents, FrameOut, Surface
  src/author.rs                  #   pub mod author — the component-author layer
  src/components/                #   button.rs list.rs tabs.rs dialog.rs …
  examples/                      #   the 12 compiling examples from goal §24
  tests/architecture.rs          #   text checks the compiler cannot express
crates/tui-testing/              # Harness, conformance suites, TestBackend helpers
apps/showcase/                   # [[bin]] name = "showcase"
apps/tablepro/                   # [[bin]] name = "tablepro"
apps/jackin-preview/             # [[bin]] name = "jackin-preview"
tools/                           # capture.sh etc., unchanged
```

Binary names are preserved via `[[bin]] name = …` in each app crate (goal §21). Edition 2024 / MSRV 1.88 / the three dependencies are unchanged (`Cargo.toml:4-5, 27-30`).

### 6.3 Visibility policy

`pub` (app-author layer): `Id`, `id!`, `ItemKey`, `Response`, `StateFlags`, `Status`, `Density`, `Surface`, every component type and its `XState`, every `XAction`, `Theme`, `ColorTokens`, `DesignTokens`, `Recipes`, `Recipe` builders, `StylePatch`, `Slot`, `Role`, `GlyphRole`, `Overlay`, `Variant`, `Part`, `Family`, `ColorLevel`, `Runtime`, `Ui` (with the author methods behind `author`).

`pub` but curated in `tui::author`: `Ui::{register_control, register_click, register_scroll, style, glyph, space, flags, take_intents, paint, paint_str, set_cursor}`, `Focusability`, `Resolved`, `Intent`, `PartMap`.

`pub(crate)`: buffer painting internals, id hashing, `Registry` internals, `StyleStack` internals, layout arithmetic, downgrade colour maths, `IdNames` storage.

**Never `pub`:** frame-local rectangles and caches on components (removes `button.rs:23`, `tabs.rs:60`, `list.rs:54`, `input.rs:33`, `dialog.rs:52`), raw palette constants (they stay in `theme/themes/`).

### 6.4 §25.5 architecture checks

| Invariant | Enforcement |
|---|---|
| apps import no private modules | Cargo crate boundary (compiler) |
| library has no app dependency | workspace dependency direction; `cargo tree -p tui` asserted in CI |
| no generic component copies in apps | `crates/tui/tests/architecture.rs` grep of `apps/**` for `impl .* fn render(.*Buffer` outside allowed files |
| literal palette values confined | grep `Color::Rgb(` / `rgb(0x` under `crates/tui/src`, allow-list `theme/themes/*.rs` and `#[cfg(test)]` fixtures |
| no raw-bg render parameters | grep public `fn show(`/`fn draw(` signatures for `: Color`; allow-list is empty |
| no `owns`/`locate` routing in apps | grep `apps/**` for `\.owns\(` / `\.locate\(` — must be zero |
| public components documented | `#![deny(missing_docs)]` + `RUSTDOCFLAGS="-D warnings" cargo doc -p tui` |
| every family has a recipe for every declared part | unit test iterating `Family::ALL × X::PARTS` against `Recipes` |
| examples compile | `cargo build -p tui --examples` in CI |
| downgrade covers every token | the exhaustive destructure in `map_colors` is a compile error when a field is added |

---

## 7. Prior-art translation table

| Reference | Problem it solves | Does the problem exist here? | How Rust ownership / terminal changes it | Adopted / Rejected | Why |
|---|---|---|---|---|---|
| **shadcn/ui — compound components** | arbitrary composed content inside a shell | Yes: `DialogBody` is a closed enum (`dialog.rs:18-28`) | JSX children → `FnOnce(&mut Ui, Rect)` slots; no runtime tree | **Adopted** (as slot closures) | Gives arbitrary bodies with zero allocation and full type safety |
| **shadcn/ui — open code** | users can read and fork the component | Partly: code is readable, but customization requires forking | Forking a Rust crate is worse than forking a JSX file; instead expose the *author layer* so users compose rather than fork | **Adapted** | §5 forbids a registry/CLI; the equivalent is a documented author API |
| **shadcn/ui — `cva` variants** | one component, many variant×state style tables | Yes: `ButtonKind` is closed (`theme.rs:558-565`) and its recipe is `match`-hard-coded (`theme.rs:387-451`) | Class strings → typed `Variant` + `Recipe` deltas; no string parsing | **Adopted** | §16 requires custom variants without replacing behaviour |
| **shadcn/ui — `cn` / tailwind-merge** | later declarations override earlier ones predictably | Yes: there is no override mechanism at all | String merging → `StylePatch::merge` with `Inherit/Set/Clear` and a proven identity/associativity law | **Adopted** | §15 requires deterministic merging with an explicit clear |
| **shadcn/ui — `Slot` / `asChild`** | replace a part's element while keeping behaviour | Yes: no way to replace a marker or label renderer | Element cloning → `.slot(Part, &mut dyn FnMut)`, layout/hit/focus stay with the component | **Adopted (narrow)** | §16 "override meaningful logical parts" |
| **Radix — focus scope / focus guard** | modal trapping and focus restoration | Yes, but the app runs it (`app.rs:417-443`) | DOM guards → `FocusRing::push_barrier` (`focus.rs:20`) + `HitRegistry::push_barrier` (`hit.rs:53`) already model this correctly | **Adopted, moved into `Dialog`/`Ui`** | §13 and Scenario F |
| **Ratatui `StatefulWidget`** | caller-owned widget state | Yes: state is inside components today | `render_stateful_widget(w, area, buf, &mut state)` cannot carry theme/focus/hit context | **Pattern adopted, trait rejected** | The `XState` split is right; the trait signature is too narrow for `Ui` |
| **Ratatui `Widget`/`WidgetRef`** | uniform draw entry point | Marginal | `Widget::render` has no context parameter → forces globals | **Rejected as the component trait** | §5 forbids a superficial universal `Widget` trait |
| **iced (Elm + `Element<Message>`)** | heterogeneous composition, typed messages, layout engine | Composition yes; a layout engine no | Retained tree of `Box<dyn Widget>` costs one allocation per node per frame; a terminal frame is cheap and diffing buys little | **Message idea adopted narrowly (per-family `XAction`); retained tree rejected** | §5 forbids a virtual DOM; §25.6 forbids full-tree scans per event |
| **egui (immediate mode, `Response`, `Memory`)** | zero-plumbing call sites; `if ui.button("x").clicked()` | Yes — the biggest ergonomic gap (`pages/buttons.rs:186-223`) | Rust closures + a per-frame `Ui` make this natural; the terminal has no retained scene either way | **`Response` and the call shape adopted; `Memory` and call-site auto-ids rejected** | Hidden global memory breaks §11 determinism; call-site ids break stable identity under reorder (§12), hence explicit `ItemKey` |
| **egui — `Id` from source location** | no manual naming | Yes (manual `WidgetId::of` today) | Source-location ids are unstable across reorder and invisible in tests | **Rejected**; `id!("path")` macro keeps names readable and stable | §12 "readable debugging", "stable identity across frames" |
| **tuirealm — `Component` trait + `AttrValue` property bag** | uniform interface, subscriptions | Uniform interface yes; property bag no | Stringly/enum-typed attributes erase types and defeat rustdoc | **Rejected** | §5 (giant universal trait) and §16 (no arbitrary string maps) |
| **cursive — `Box<dyn View>` tree + `Rc<RefCell>` callbacks** | heterogeneous children, callbacks | Heterogeneity yes | Interior mutability makes borrowed domain data (`&'a [Order]`) impossible and defeats the borrow checker's guarantees | **Rejected** | Scenario D requires borrowed rows |
| **bubbletea (Elm) + lipgloss (style inheritance)** | one style struct that inherits, with explicit unset | Yes: no inheritance model exists | Go's `Style.Inherit()` maps onto `Slot::{Inherit,Set,Clear}`; Go strings map onto `Buffer` cells | **Style-inheritance model adopted; Elm loop rejected** | Proves the merge model works in a terminal; the Elm loop would rewrite all three apps |

---

## 8. Trade-offs, risks, and executable acceptance conditions

### 8.1 Known trade-offs (inference)

1. **`show` performs interaction.** Even with the `update`/`draw` split, a reviewer may read `show` as "render mutates state". Mitigation: `update` is buffer-free and separately public; the doc states the rule as *"drawing never changes semantics; `update` changes semantics only in response to a queued intent"*; a conformance test proves `draw`-twice is a no-op.
2. **One-frame input latency.** Intents resolve against the previous frame's registry (as they already do — `app.rs:711-722` rebuilds hits/ring after drawing). Layout changes therefore take effect on the next frame. This is the existing behaviour; it must be documented rather than accidentally relied on.
3. **Generic parameters on collection views.** `List<'a, T, K, R>` has four parameters. They are inferred at the call site and never appear in application signatures because the view is consumed immediately. The cost is longer rustdoc type names and slower compiles for grid-heavy modules.
4. **Recipe indirection cost.** Style resolution moves from a `match` (`theme.rs:387-451`) to a merge chain. Mitigation: resolve once per part per row (not per cell), keep `states` in a `SmallVec` of ≤ 8, and add an optional per-frame memo only if a benchmark demands it.
5. **`ItemKey` burden.** Callers must supply a key for correct reorder behaviour. `ItemKey::index` keeps trivial cases short but is documented as unstable under reorder; a debug assertion warns when a keyed collection changes length while `Index` keys are in use.
6. **Surface ladder vs. today's `lift`.** The array-based `raise` changes behaviour for any theme where two planes share a colour, and keeps `Field → FieldHover` as an explicit special case (matching `theme.rs:367-368`). Junie output is unchanged, but the change must be proven by capture diff, not asserted.
7. **Mono legibility is a *fix*, not a preservation.** Today accent and error both become `Color::Gray`. Adding mono glyph/modifier fallbacks is an intentional visual change at `--color none` and must be recorded as such under goal §3.
8. **Workspace churn.** Paths in `tools/capture.sh`, CI, and every `use junie_tui::…` change at once. This is a single mechanical slice and should not be interleaved with API changes.

### 8.2 Acceptance conditions for the 12 scenarios of §15

Each is an executable artifact. `E#` = a file under `crates/tui/examples/` that must compile and run headless; `T#` = a test in the named crate.

| §15 scenario | Executable acceptance condition |
|---|---|
| 1. default Junie theme, no configuration | `T1` `tui::theme::tests::junie_default_is_complete`: `Theme::junie()` yields a recipe for every `(Family, Part)` pair in `Family::ALL`, and `Theme::default() == Theme::junie()`. `E1 examples/01_button.rs` renders a default button in ≤ 20 lines with no theme mention. |
| 2. complete custom theme from custom colours | `E2 examples/02_custom_theme.rs` builds `Theme::from_tokens(ColorTokens { … })` with a light, non-green palette. `T2` `tui-testing::conformance::custom_theme_touches_every_component`: for every component in the catalog, the Junie render and the custom render differ in ≥ 1 cell style, and no rendered cell uses a colour absent from the custom `ColorTokens` (proves no literal leaks). |
| 3. change a few roles, derive the rest | `E3 examples/03_partial_override.rs` uses `Theme::junie().builder().accent(rgb(0x7aa2f7)).build()`. `T3`: every token not named in the builder is byte-identical to Junie; every token derived from `accent` (hover, pressed, tint, focus) changed; `assert!(contrast(on_accent, accent) >= 3.0)`. |
| 4. override one component family globally | `E4 examples/04_family_recipe.rs`. `T4`: after `override_family(Family::BUTTON, …)`, every rendered `Button` in the showcase catalog reflects the change and no `List`/`Tabs`/`Dialog` cell changes. |
| 5. override one variant globally | `T5`: `override_variant(BUTTON, PRIMARY, …)` changes primary buttons only; secondary/subtle/danger renders are byte-identical. `T5b`: `define_variant(BUTTON, GHOST, …)` renders and participates in focus/hover/press without a code change to `Button`. |
| 6. scoped override (subtree) | `E6 examples/06_scoped_override.rs` renders two identical panes, one inside `ui.with_styles(&OVERLAY, …)`. `T6`: the two panes differ; `assert_eq!(theme_before, theme_after)` proves the global theme was not mutated (§25.2). |
| 7. per-instance override | `E7`/`T7`: two buttons, same id-path prefix, same variant; one carries `.patch(&P)`; renders differ in exactly the patched attribute and nowhere else. |
| 8. logical-part override | `T8` `part_override_matrix`: for each of `CONTAINER, BORDER, GUTTER, MARKER, LABEL, META, ICON, BODY, ACTIONS, TRACK, THUMB, ROW`, applying `patch_part` changes ≥ 1 cell and the diff is confined to that part's registered sub-rect. |
| 9. state-specific overrides | `T9` `state_override_matrix`: for each of `FOCUSED, HOVERED, PRESSED, SELECTED, DISABLED, ERROR, BUSY, EDITING`, a `StateRule` patch changes the render only when the flag is live; and precedence order 1→6 is asserted by six layered patches whose composition equals the documented winner. |
| 10. app-specific custom component on the same theme | `E10 examples/10_custom_component.rs` implements `Quota` (§2.3) using only `tui::author`. `T10`: it responds to focus, hover and press; it re-renders under the custom theme of `E2` with different colours; `cargo build -p tui --examples` proves no private import. |
| 11. truecolor / 256 / 16 / no-color for built-in **and** user themes | `T11` parameterised over `[Theme::junie(), custom_theme()] × [TrueColor, Ansi256, Ansi16, Mono]`: `downgrade_theme` returns a theme in which **no** `Color::Rgb` survives below `TrueColor` (this is the regression test for `theme.rs:189-223`), and a full catalog render succeeds at every level. |
| 12. state meaning survives without colour | `T12` `mono_states_are_distinguishable`: at `ColorLevel::Mono`, for every component and every state in `{default, focused, hovered, pressed, selected, disabled, error, busy, editing}`, the rendered `(symbol, modifier)` pairs of the state's own rect differ from the default state's. Colour is excluded from the comparison. |

Cross-cutting gates that back the recommendation itself:

- `T-A` **Scenario A**: an integration test builds a screen with a field, a button, a list and a dialog; the test asserts the source file contains no `Focus`, `HitRegistry`, `owns`, `locate`, `hover`, `pressed` or `set_cursor` token.
- `T-B` **§11 purity**: `draw` twice with identical inputs produces an identical buffer *and* identical state (`assert_eq!(state_before, state_after)`), for every component.
- `T-C` **`update` is headless**: every `update` is exercised through `Ui::headless` with no `Buffer` in scope.
- `T-D` **Scenario D**: a list over `&[Order]` renders 100 000 rows; a `#[global_allocator]` counting shim asserts zero allocations during the render frame.
- `T-E` **Scenario E**: insert, remove and reorder tabs and list rows; assert active tab, checked set, cursor and pending edit all still resolve to the same `ItemKey`.
- `T-F` **Scenario F**: dialog → menu → picker; assert only the topmost registers hit regions and ring entries, Esc unwinds one level per press, and focus returns to the pre-dialog id.
- `T-G` **merge laws**: identity, absorption, clear, associativity, modifier symmetry (§3.5).
- `T-H` **precedence**: six layered patches, one assertion per level, in both "each level alone" and "all levels together" configurations.

---

### Facts vs. inference — summary

- **Facts** (§1, and every `file:line` citation elsewhere): the token list, the downgrade mechanism and its two structural defects, the Junie-specific assumptions in `lift`/`backdrop`/`button`, the hard-coded glyph/dimension/spacing inventory, the raw-`bg` threading pattern, the render-time semantic mutations, the four incompatible event signatures, the positional identity model, the owned-`String` collections, and the closed `DialogBody`.
- **Inference** (§2.2 onward): the hybrid component model and its scoring, all Rust type sketches, the theme/recipe/patch design and precedence order, the token-home decision table, the composition mechanisms, the API vocabulary, the workspace recommendation and its honest caveat, the prior-art adoptions, and the trade-offs and acceptance conditions.
