# Modern-API and Practices Audit — Binding Guidance

**Scope.** Crate/feature choice for the new workspace, current-code API drift against ratatui 0.30.2 / crossterm 0.29.0 / unicode-width 0.2.2, Rust 2024 + MSRV-1.88 practice, the `smallvec`/`bitflags` question, the MSRV question, and a binding rule set + architecture check for builders.

**Method.** Every `[F]` claim below is read from the unpacked registry sources under
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` (`ratatui-0.30.2`, `ratatui-core-0.1.2`, `ratatui-widgets-0.3.2`, `ratatui-crossterm-0.1.2`, `ratatui-macros-0.7.2`, `crossterm-0.29.0`, `unicode-width-0.2.2`), not from memory. Repository citations are `path:line` in `/Users/donbeave/Projects/terminal-components-claude`.

Facts are marked **[F]**. Everything else is a recommendation, a rejection, or a rationale.

---

## 1. Crate choice — `ratatui-core`, not `ratatui`

### 1.1 Facts

**[F] Upstream's own guidance is explicit.** `ratatui-core-0.1.2/src/lib.rs:16-18`: *"Widget libraries should generally depend on `ratatui-core`, benefiting from a stable API and reducing the need for frequent updates."* Repeated at `:43-53` and, at length, in `ratatui-0.30.2/src/widgets.rs:599-613` ("Depend on `ratatui_core`… lighter dependencies, better compile times, future-proofing").

**[F] What `ratatui-core` 0.1.2 exports** (`ratatui-core-0.1.2/src/lib.rs:81-88`):

| module | contents (verified) |
|---|---|
| `buffer` | `Buffer`, `Cell`, `CellDiffOption`, `CellWidth`, `BufferDiff` (`src/buffer.rs:10-13`) |
| `layout` | `Layout`, `Spacing`, `Constraint`, `Direction`, `Flex`, `Margin`, `Offset`, `Position`, `Size`, `Rect`, `Rows`, `Columns`, `Positions`, `Alignment`/`HorizontalAlignment`/`VerticalAlignment` (`src/layout.rs:324-333`) |
| `style` | `Color`, `ParseColorError`, `Style`, `Modifier`, `Styled`, `Stylize`, `style::palette::{material, tailwind}` (`src/style.rs:74-86`, `src/style/palette.rs:5-6`) |
| `symbols` | `bar`, `block`, `border`, `braille`, `half_block`, `line`, `marker`, `merge`, `pixel`, `scrollbar`, `shade`, `Marker`, `DOT` (`src/symbols.rs:3-15`) |
| `terminal` | `Terminal`, `TerminalOptions`, `Frame`, `CompletedFrame`, `Viewport` (`src/terminal.rs:55-56`, `:397`, `:465`) |
| `text` | `Line`, `Span`, `Text`, `Masked`, `StyledGrapheme`, `ToLine`, `ToSpan`, `ToText` (`src/text.rs:51-64`) |
| `widgets` | **only** `Widget` and `StatefulWidget` (`src/widgets.rs:5-9`) |
| `backend` | `Backend`, `ClearType`, `WindowSize`, **`TestBackend` (unconditional)** (`src/backend.rs:112-113`) |

**[F] What `ratatui-core` does *not* provide**, and where it lives instead:

* `Block`, `BlockExt`, `Padding`, `TitlePosition`, `Shadow`, `CellEffect`, `Dimmed`, `dimmed`, `Borders`, `BorderType`, `Clear`, `Fill`, `Paragraph`, `List`, `Table`, `Tabs`, `Gauge`, `Scrollbar`/`ScrollbarState`/`ScrollbarOrientation`/`ScrollDirection`, `Sparkline`, `Chart`, `Canvas`, `RatatuiLogo` → **`ratatui-widgets`** (`ratatui-0.30.2/src/widgets.rs:668-689`, `ratatui-widgets-0.3.2/src/lib.rs:115-138`).
* `WidgetRef`, `StatefulWidgetRef`, `FrameExt` → **`ratatui` only**, behind `unstable-widget-ref` (`ratatui-0.30.2/src/widgets.rs:690-691`, `:759-771`).
* `run()`, `init()`, `try_init()`, `restore()`, `try_restore()`, `init_with_options()`, `DefaultTerminal` → **`ratatui` only** (`ratatui-0.30.2/src/init.rs:199-213`, `:318-367`, `:397`).
* `CrosstermBackend` and the version-unified `crossterm` re-export → **`ratatui-crossterm`** (`ratatui-crossterm-0.1.2/src/lib.rs:86-98`, `:159-163`).

**[F] Default-feature shapes.**
* `ratatui-core`: `default = []`, `#![no_std]`; `std` is an opt-in feature (`ratatui-core-0.1.2/Cargo.toml:63-87`, `src/lib.rs:1`, `:77-79`).
* `ratatui`: `default = ["all-widgets", "crossterm", "layout-cache", "macros", "underline-color"]` (`ratatui-0.30.2/Cargo.toml:74-80`) — i.e. depending on `ratatui` at all forces `ratatui-widgets` (calendar, canvas, chart, barchart, sparkline, mascot…), `ratatui-macros`, and `layout-cache` + `critical-section` into the graph.
* `ratatui-crossterm`: `default = ["crossterm_0_29", "underline-color"]` (`ratatui-crossterm-0.1.2/Cargo.toml:62-65`) — exactly what we want, and it does **not** turn on `ratatui-core/std`.
* All three declare `edition = "2024"`, `rust-version = "1.88.0"` (`ratatui-core-0.1.2/Cargo.toml:13-14`, `ratatui-0.30.2/Cargo.toml:13-14`, `ratatui-crossterm-0.1.2/Cargo.toml:13-14`).

**[F] `ratatui-macros` 0.7.2 surface**: `span!`, `line!`, `text!`, `constraint!`, `constraints!`, `vertical!`, `horizontal!`, `row!` (`ratatui-macros-0.7.2/src/lib.rs:25`, `:30-117`). The layout macros produce `Layout`/`Constraint`; the text macros produce owned `Span`/`Line`/`Text`; `row!` produces `ratatui_widgets::table::Row`.

### 1.2 Decision

**`crates/tui` (`junie-tui`) depends on `ratatui-core` for everything paint/theme/layout/text, and on `ratatui-crossterm` behind a default-on `crossterm` feature for the input boundary and the terminal session. It never depends on `ratatui`, `ratatui-widgets`, or `ratatui-macros`. The applications depend on `junie-tui` alone.**

The last clause is the important one and it strengthens the architecture: §16.5's `architecture::applications_depend_only_on_the_library_facade` becomes a one-line `cargo tree -p showcase -e normal --depth 1` assertion instead of a source scan, because `junie-tui` re-exports the handful of ratatui types the public API mentions (`Rect`, `Position`, `Size`, `Buffer`, `Frame`, `Color`, `Modifier`, `Style`, `Line`, `Span`, `Text`) under `junie_tui::` and `junie_tui::author::`. §17's example 2 (`use ratatui::style::Color::Rgb as rgb;`, `COMPONENT_ARCHITECTURE.md:2116`) must be rewritten to `junie_tui::Color::from_u32(...)`; otherwise the boundary claim is false at the first example.

This is **not** a pure widget-library split: `junie-tui` owns the runtime and the input model, so the crossterm backend is a legitimate part of it. The split is honest and testable — `cargo check -p junie-tui --no-default-features` must compile the whole paint/theme/layout/collection core with no backend at all, which is the mechanical proof that the widget layer is backend-independent.

**Key vocabulary.** `Key`, `Chord`, `KeyMap` keep `crossterm::event::{KeyCode, KeyModifiers}` (§6.1), obtained **only** through `ratatui_crossterm::crossterm` (`ratatui-crossterm-0.1.2/src/lib.rs:86-98`), never through a second direct `crossterm = "0.29"` line in our manifest. The re-export exists precisely to prevent the version skew a duplicate entry would risk (`ratatui-crossterm-0.1.2/src/lib.rs:16-40`). Rejected alternative: define our own `KeyCode`/`KeyModifiers` enums. It would remove crossterm from the public surface and let us drop `SUPER`/`HYPER`/`META` (which are unreachable without keyboard-enhancement flags, §2.6), but it is ~40 hand-written variants plus `From` impls to replace a small amount of well-understood code — exactly what goal §21 tells us not to do.

### 1.3 Exact manifests

**Workspace root `Cargo.toml` (virtual):**

```toml
[workspace]
resolver = "3"                      # explicit: a virtual manifest has no edition to imply it
members         = ["crates/tui", "crates/tui-testing", "apps/showcase", "apps/tablepro", "apps/jackin-preview", "xtask"]
default-members = ["crates/tui", "crates/tui-testing", "apps/showcase", "apps/tablepro", "apps/jackin-preview"]

[workspace.package]
version      = "0.1.0"
edition      = "2024"
rust-version = "1.88"
license      = "MIT"
repository   = "…"

[workspace.dependencies]
junie-tui         = { path = "crates/tui" }
junie-tui-testing = { path = "crates/tui-testing" }
ratatui-core      = { version = "0.1.2", default-features = false, features = ["std", "underline-color"] }
ratatui-crossterm = { version = "0.1.2" }          # default = crossterm_0_29 + underline-color
unicode-width        = "0.2.2"
unicode-segmentation = "1.13"
bitflags             = "2.13"

[profile.release]
lto = "thin"
codegen-units = 1
```

**`crates/tui/Cargo.toml`:**

```toml
[package]
name = "junie-tui"
version.workspace = true; edition.workspace = true; rust-version.workspace = true
license.workspace = true; repository.workspace = true

[lints]
workspace = true

[features]
default   = ["crossterm"]
crossterm = ["dep:ratatui-crossterm"]   # Input::from_crossterm, TerminalSession, run()
testing   = []                          # Runtime/Ui inspection surface (§17.0 A1, A2)

[dependencies]
ratatui-core.workspace = true
ratatui-crossterm = { workspace = true, optional = true }
unicode-width.workspace = true
unicode-segmentation.workspace = true
bitflags.workspace = true

[dev-dependencies]
junie-tui-testing.workspace = true      # dev-only cycle; see below
trybuild = "1"                          # §16.1 must_use_is_enforced, secret::is_not_clone_not_eq
```

**`crates/tui-testing/Cargo.toml`:**

```toml
[package]
name = "junie-tui-testing"
publish = false
version.workspace = true; edition.workspace = true; rust-version.workspace = true

[lints]
workspace = true

[dependencies]
junie-tui   = { workspace = true, features = ["testing"] }
ratatui-core.workspace = true           # TestBackend, Terminal, Buffer
bitflags.workspace = true               # conformance Caps (§16.2)
```

The `junie-tui` ⇄ `junie-tui-testing` cycle is a **dev-dependency** cycle, which Cargo permits, and under resolver 2/3 the `testing` feature it enables does not leak into `cargo build -p showcase`. This is what keeps `Runtime::hover()`/`state_of()`/`resolved()` (`COMPONENT_ARCHITECTURE.md:1645-1657`) out of shipped binaries.

**`apps/*/Cargo.toml`:**

```toml
[dependencies]
junie-tui.workspace = true              # the ONLY normal dependency

[dev-dependencies]
junie-tui-testing.workspace = true
```

**`xtask/Cargo.toml`:** `syn` (features `full`, `visit`, `parsing`), `cargo_metadata`, `walkdir`, `regex` — versions pinned at implementation time. Nothing depends on `xtask`, so none of these reach the library or the apps; `default-members` excludes it so `cargo build` never compiles it.

### 1.4 Feature-flag justification

| Feature | Decision | Reason |
|---|---|---|
| `ratatui-core/std` | **on — mandatory** | `default = []` and `#![no_std]` (`ratatui-core-0.1.2/Cargo.toml:65`, `src/lib.rs:1`). We need `HashMap` (`FocusState.restore`, `Registry::names`), `Instant`, the panic hook, and `Terminal`'s cursor-restore `eprintln` path (`src/terminal.rs:481-484`). |
| `ratatui-core/underline-color` | **on — mandatory** | `StylePatch.underline: Slot<Role>` (§11.3) is inert without it: `Style::underline_color` and the backend's `SetUnderlineColor` are both `#[cfg(feature = "underline-color")]` (`ratatui-crossterm-0.1.2/src/lib.rs:77-78`, `:267-272`, `:497-501`). Must be on for **both** core and crossterm or the backend silently drops the colour; `ratatui-crossterm`'s default already includes it and forwards to core (`Cargo.toml:71`). |
| `ratatui-crossterm/crossterm_0_29` | **on (default)** | Highest supported crossterm; selects the re-exported crate (`ratatui-crossterm-0.1.2/src/lib.rs:89-90`). |
| `ratatui-core/layout-cache` | **off** | It caches `Layout::split` results behind `critical-section` + an LRU (`ratatui-core/Cargo.toml:66`, `src/layout.rs:282-287`). §10 deliberately uses no constraint solver, so there is nothing to cache; and a process-global LRU perturbs the deterministic allocation counts §16.6 asserts (`style_resolve_10k_parts` "exactly 0 allocs", `debug_and_release_alloc_counts_match`). Note `ratatui`'s default enables it — a further reason not to take `ratatui`. Revisit only if a component adopts `Layout`, and re-bless `perf_baseline.txt` in the same commit. |
| `ratatui-core/palette` | **off** | It adds only `From<Srgb>` / `From<LinSrgb>` for `Color` (`src/style/palette_conversion.rs:22-52`). **There is no `Color::from_hsl` in 0.30** — the only constructor is `Color::from_u32` (`src/style/color.rs:133-138`), which is `const` and unfeatured. `downgrade_color` (§11.4) is exact integer arithmetic and must stay deterministic; pulling `palette` + `libm` for two `From` impls is a framework-sized dependency for a small amount of well-understood code (goal §21). |
| `ratatui-core/scrolling-regions` | **off** | Only affects `Terminal::insert_before` for **inline** viewports and adds `ScrollUpInRegion`/`ScrollDownInRegion` backend commands (`src/terminal.rs:255-257`, `:382-384`; `ratatui-crossterm/src/lib.rs:362-386`, `:705-792`). We render fullscreen with full-frame diffing. Zero benefit. |
| `unstable-widget-ref` / `unstable-*` | **off** | Only in `ratatui`, gated by `instability::unstable` (`ratatui-0.30.2/src/widgets.rs:690-691`), and we implement no ratatui widget trait at all (§3.2 rejects a universal trait). `unstable-rendered-line-info` is a `Paragraph` feature. Adopting an unstable API in a library that claims a stable public surface (G1) is a contradiction. |
| `serde`, `anstyle`, `portable-atomic`, `all-widgets`, `widget-calendar`, `macros` | **off** | Unused; `macros`/`all-widgets` are `ratatui`-only and would drag in crates we reject. |
| `junie-tui/crossterm` | **on by default** | Apps need it; `--no-default-features` is the boundary proof. |
| `junie-tui/testing` | **off by default** | Enabled only via the dev-dependency path. |

### 1.5 `ratatui-macros` — rejected

1. `constraints!`/`vertical!`/`horizontal!` produce `Layout` + `Constraint` (`ratatui-macros-0.7.2/src/lib.rs:73-97`). §10 fixes `Track::{Fixed, Flex, Auto}` and states "No general constraint solver". Adopting these introduces a **second layout vocabulary** in the same crate, which is precisely the API-inconsistency class this refactor exists to remove (G1).
2. `line!`/`span!`/`text!` (`:30-66`) allocate a `Span`/`Line`/`Text` per invocation and are `format!`-shaped. §16.6 demands **0 allocs/frame** on the row path, and §17.0 A8 already specifies the correct primitive: `RowUi::label_fmt(core::fmt::Arguments<'_>)`, which formats in place. `line!` also shadows `std::line!`.
3. `row!` builds `ratatui_widgets::table::Row`, and `DataTable` is deleted (§18.2).
4. Goal §5 ("Do not hide behavior behind excessive procedural macros… must remain open, readable, debuggable") and §2.2 ("no macro DSL"). The library's only macro stays `id!` (§7.1).

---

## 2. Current-code API drift

Each item: **[F]** what the code does now (`path:line`), **[F]** the modern API with its exact unpacked-source path, then the recommendation.

### 2.1 Hand-rolled fill vs `Buffer::set_style` / `Rect::positions()`

**[F] Now:** `src/ui/ctx.rs:135-144` —
```rust
pub fn fill(buf: &mut Buffer, area: Rect, style: Style) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut(Position::new(x, y)) { cell.set_symbol(" "); cell.set_style(style); }
```
Called from `list.rs:254`, `list.rs:9`, `viewport.rs:604`, `viewport.rs:646`, and (per API §3.6) ~20 more sites.

**[F] Modern:**
* `Buffer::set_style(area, style)` — style-only, already `intersection`-clipped (`ratatui-core-0.1.2/src/buffer/buffer.rs:405-413`).
* `Rect::positions() -> Positions` iterating every `Position` in a rect (`ratatui-core-0.1.2/src/layout/rect.rs:6`, `:67`; documented example at `src/layout.rs:277-279`).
* `Cell::set_symbol(..).set_style(..)` chains (`buffer.rs:360`).
* Upstream now also ships a `Fill` widget for exactly this (`ratatui-widgets-0.3.2/src/fill.rs:51-80`) and `Dimmed`/`dimmed()` for backdrop dimming (`ratatui-widgets-0.3.2/src/block/shadow.rs:321-344`).

**Recommend.** Two distinct `Ui` methods, both iterating `area.positions()` instead of a nested `for y/for x`:
* `Ui::paint_style(area, style)` → `buf.set_style(clip.intersection(area), style)` — restyle without touching symbols.
* `Ui::fill(area, style)` → per-position `set_symbol(" ").set_style(style)` — the current `fill` semantics.

Do **not** take `ratatui-widgets::Fill`/`Dimmed`: they are foreign `Widget`s that write straight to the `Buffer` and therefore cannot mark the layer's written-cell bitset (R3), and `Dimmed` halves RGB channels and falls back to `Color::Black` for non-RGB (`shadow.rs:329-338`) — colour-space arithmetic where §11.3 A2 requires role arithmetic, and it cannot exclude the footer row (`DESIGN.md:537`). Keep `Ui::dim_layer` walking `FrameOut::roles` (§11.6). Record both as deliberate re-implementations so a later reader does not "fix" them.

### 2.2 Pre-truncated `String` + `set_string` vs `set_stringn`

**[F] Now:** `src/widgets/list.rs:284` `buf.set_string(row.x + 3, y, crate::ui::text::fit(&item.label, lw), st)` — one `String` allocated per visible row per frame (`ui/text.rs:80-84` = `truncate` + `format!` + `" ".repeat`). Same shape at `button.rs:142-144`, `list.rs:221-228`, `viewport.rs:635`.

**[F] Modern:** `Buffer::set_stringn(x, y, string, max_width, style) -> (u16, u16)` (`buffer.rs:336-370`) clips to `max_width` **and** to the buffer's right edge, filters zero-width graphemes and control characters, resets the trailing cells of multi-width graphemes, and returns the end position — everything `fit` was doing, allocation-free.

**Recommend.** `Ui::paint_str(area, text, style) -> u16` is `buf.set_stringn(area.x, area.y, text, area.width as usize, style)`, returning columns written. `ui::text::fit` / `fit_right` are deleted from every render path (§18.1 already says so; §16.6 `fit_10k_grapheme_line_to_80` must then record **0**). `truncate`/`truncate_middle` survive only for non-render callers that need an owned ellipsised string.

### 2.3 Display-width: our `width()` disagrees with what ratatui actually paints — **correctness bug class**

**[F] Now:** `src/ui/text.rs:6-8` `pub fn width(s: &str) -> usize { UnicodeWidthStr::width(s) }`, used by `button.rs:64`, `list.rs:226/275`, `viewport.rs:17`, and every layout computation in the library.

**[F] Modern:** ratatui 0.30 measures with its own `CellWidth` trait, not raw `unicode-width`:
```rust
// ratatui-core-0.1.2/src/buffer/cell_width.rs:24-46
impl CellWidth for str {
    fn cell_width(&self) -> u16 {
        if self.len() == 1 { 1 }                       // single-byte incl. control chars
        else { (self.width() as u16).saturating_add(count_halfwidth_sound_marks(self)) }
    }
}
```
`count_halfwidth_sound_marks` adds **+1 per U+FF9E / U+FF9F** because `unicode-width` reports them as zero-width (Grapheme_Extend) while terminals render them as one cell (`cell_width.rs:48-75`, with citations to reline #832 and Microsoft Terminal #18087). `Buffer::set_stringn` consumes columns with `cell_width` (`buffer.rs:352`), and it is exported (`src/buffer.rs:12`).

**Consequence today:** for halfwidth-katakana text our layout reserves N columns and the buffer consumes N+k — off-by-k overflow past the reserved rect, exactly the class R4/R5 exist to prevent.

**[F] unicode-width 0.2.2 semantics** (`unicode-width-0.2.2/src/lib.rs:56-122`): `UnicodeWidthStr::width` is *not* a sum of char widths — `"\r\n"` is width 1; fully-qualified emoji ZWJ sequences, emoji modifier sequences and emoji presentation sequences are width **2**; several script ligatures (Arabic Lam-Alef, Hebrew Alef-Lamed, Lisu tone pairs, Tifinagh joins) collapse to width 1; `Default_Ignorable_Code_Point` and `Grapheme_Extend` chars are 0. `UnicodeWidthChar::width` returns `Option<usize>` — **`None` for control characters** (`:194-202`); `UnicodeWidthStr::width` returns `usize` and simply omits them. `width_cjk` (Ambiguous → 2) is behind the default-on `cjk` feature (`:28-49`).

**Recommend.**
1. **One width function in the whole workspace**: `crates/tui/src/text/measure.rs::width(s: &str) -> u16` delegating to `<str as ratatui_core::buffer::CellWidth>::cell_width`. Every measurement — `RowUi`, `CellUi`, `truncate`, `wrap`, `TextEditorCore`, the viewport's `Cell.w`, `StatusBar`'s priority drop — goes through it.
2. Ban direct `unicode_width::` imports outside that file (grep rule 12 in §6).
3. Do **not** use `width_cjk`; the architecture has no East-Asian context switch, and adding one would fork every measurement.
4. Test `text::width_matches_ratatui_cell_width` over a corpus including `"ｶﾞ"`, `"あ"`, `"a\u{FF9E}"`, `"\r\n"`, a ZWJ family emoji, a combining-mark cluster, and `"\u{7}"`. This is the pin that keeps §16.1's `wide_characters_count_as_two_columns` and `zwj_emoji_is_one_grapheme` honest.
5. `unicode-width` stays a direct dependency only so `measure.rs` can name the crate for the `width_cjk`-free `cell_width` fallback path and for the grapheme-boundary tests; it is already in the graph via `ratatui-core` (`ratatui-core/Cargo.toml:158-159`, `version = ">=0.2.0"`), so our `0.2.2` unifies to a single copy — which is *required*, otherwise two width tables could disagree.

### 2.4 `unicode-segmentation`

**[F] Now:** `ui/text.rs:20/42/130`, `core/text.rs:7` use `s.graphemes(true)`; `core/text.rs` addresses by byte offset and moves by grapheme.
**[F] Modern:** unchanged API; `ratatui-core` itself uses `UnicodeSegmentation::graphemes(s, true)` (`buffer.rs:350`). `grapheme_indices(true)` yields `(byte_offset, &str)` in one pass.

**Recommend.** `ui::text::fuzzy` (`ui/text.rs:145-171`) currently lowercases the label and then returns **byte offsets into the lowercased string**, which the caller uses to index the *original* (§1.3 latent defect). Rewrite with `grapheme_indices(true)` over the original, comparing case-folded graphemes — this fixes the mis-highlight and satisfies `fuzzy_returns_grapheme_indices_into_the_original_label` (§16.1) in one change. Ban `unicode_segmentation::` outside `crates/tui/src/text/**`.

### 2.5 Mouse normalisation drops modifiers and three button events

**[F] Now:** `src/core/event.rs:110-129`:
```rust
Event::Mouse(MouseEvent { kind, column, row, .. }) => {   // `modifiers` discarded
    let kind = match kind {
        MouseEventKind::Down(MouseButton::Right) => MouseKind::Secondary,
        …
        _ => return None,                                  // Up(Right), Drag(Right), Down/Up/Drag(Middle)
```
**[F] Modern:** crossterm 0.29 `MouseEvent` carries `pub modifiers: KeyModifiers` (`crossterm-0.29.0/src/event.rs:777-786`); `MouseEventKind` is a closed 8-variant enum — `Down/Up/Drag(MouseButton)`, `Moved`, `ScrollDown`, `ScrollUp`, `ScrollLeft`, `ScrollRight` (`:800-817`).

**Recommend.** §6.1's `Mouse { kind, pos, mods }` and `MouseKind::{…, Secondary, SecondaryUp, Wheel(Axis, i16)}` are the right target. Implement the match **exhaustively — no `_` arm** over `MouseEventKind`, so a crossterm 0.30 variant is a compile error instead of a silently dropped event. `ScrollLeft`/`ScrollRight` are already handled (`event.rs:121-122`) and must stay; `Wheel(Axis::H, ±1)` is their home.

### 2.6 Key events: `KeyEventKind`, and the keyboard-enhancement decision

**[F] Now:** `src/core/event.rs:101-109` matches `kind: KeyEventKind::Press | KeyEventKind::Repeat`. That is correct.
**[F] Modern conveniences:** `Event::is_key_press()`, `is_key_repeat()`, `as_key_press_event()` (`crossterm-0.29.0/src/event.rs:587-689`); `KeyEvent::is_press()/is_release()/is_repeat()` (`:1009-1022`).
**[F] Semantics to record:** `KeyEvent.kind` is *only* populated on Unix when `KeyboardEnhancementFlags::REPORT_EVENT_TYPES` has been pushed; on Windows it is always set (`:944-947`). `KeyEvent.state` and `KeyModifiers::{SUPER, HYPER, META}` require `DISAMBIGUATE_ESCAPE_CODES` (`:834-848`, `:911-932`). `KeyEvent`'s `PartialEq`/`Hash` normalise ASCII case against `SHIFT` (`:995-1007`, `:1036-1072`) — which is exactly the behaviour `Chord: Eq + Hash` (§6.1) wants and should be documented rather than re-implemented.

**Recommend — do not push keyboard-enhancement flags.** `PushKeyboardEnhancementFlags` / `PopKeyboardEnhancementFlags` exist (`:493`, `:522`) and are tempting, but:
* `DISAMBIGUATE_ESCAPE_CODES` makes bare `Esc` arrive as a CSI-u sequence, forcing a second code path through the §9 Esc ladder and `Dismiss.esc`;
* `REPORT_EVENT_TYPES` starts delivering `Release` events that §3.3 step 1 drops anyway — pure input-rate cost;
* support is terminal-dependent (the doc lists kitty, foot, WezTerm, alacritty, notcurses, neovim, kakoune, dte — `:483-491`), so behaviour would differ per terminal, which contradicts §16.4's determinism claim;
* the `Chord` set in §13.1 needs only Ctrl/Alt/Shift.

Record it as an explicit rejection with the flag names, and add `KeyboardEnhancementFlags` to the forbidden-pattern grep so it cannot appear silently. If it is ever wanted, gate it on `crossterm::terminal::supports_keyboard_enhancement()` and a `Capability` field, and re-bless the app baselines.

Corollary for tests: `key_release_is_dropped` (§16.1) must **synthesize** a `KeyEvent` with `kind: Release`; on Unix without the flags no terminal will ever produce one.

### 2.7 Terminal session: `ratatui::init/restore` vs the hand-rolled guard

**[F] Now:** `src/runtime.rs:47-67` does `enable_raw_mode()` → `execute!(EnterAlternateScreen, EnableMouseCapture, EnableBracketedPaste)` → `CrosstermBackend::new(stdout())` → `Terminal::new` → `take_hook`/`set_hook`. Restore at `:88-95` writes a **raw ANSI byte string** for DECAWM: `const ENABLE_WRAP: &str = "\x1b[?7h";` (`runtime.rs:42`, used at `:91`).

**[F] Modern:** `ratatui::try_init()` performs raw mode + alternate screen + panic hook (`ratatui-0.30.2/src/init.rs:369-399`), `try_restore()` undoes it, `run(closure)` wraps both (`:318-326`). The comparison table at `:182-188` shows what each does. Crucially, **none of them enable mouse capture or bracketed paste** — that stays the application's job. `Terminal::try_draw` is the fallible sibling of `draw` (`ratatui-core-0.1.2/src/terminal.rs:101-102`, `:136-139`). `DefaultTerminal = Terminal<CrosstermBackend<Stdout>>` (`ratatui-0.30.2/src/init.rs:213`).

**Recommend, and record the cost.** Because we take `ratatui-core` + `ratatui-crossterm` and not `ratatui`, `init`/`restore`/`run`/`DefaultTerminal` are **not available** — they live in `ratatui/src/init.rs`. This is the one real cost of §1.2 and must be written down. Keep `TerminalSession`, but make it a faithful mirror of `try_init` plus our two extra modes:

* install the chained panic hook **before** the first mode change (`runtime.rs:58-62` already chains; the ordering note is at `init.rs:196-197` / `terminal.rs:316-319`);
* `type DefaultTerminal = Terminal<CrosstermBackend<Stdout>>` as our own alias;
* replace `ENABLE_WRAP`/`"\x1b[?7h"` with the typed commands `crossterm::terminal::{DisableLineWrap, EnableLineWrap}` inside the same `execute!` — a raw escape byte string in a library is the exact "hand-rolled where a typed command exists" smell this audit is for, and the grep in §6 forbids `\x1b[`;
* restore in one reverse-order `execute!` and keep `leave()` idempotent (already correct at `runtime.rs:73-79`);
* `terminal.draw(|f| …)` stays (`runtime.rs:114`) because our render closure is infallible; document `try_draw` as the alternative if a fallible `App::draw` is ever introduced.

### 2.8 Cursor

**[F] Now:** `src/ui/ctx.rs:119-123` `RenderCtx::set_cursor(Position)` sets a field; `viewport.rs:669` writes it; the app forwards it to the frame. Last writer wins.
**[F] Modern:** `Frame::set_cursor_position` is the single sanctioned request; the terminal applies it after the buffer diff so the cursor lands on top (`ratatui-core-0.1.2/src/terminal.rs:11-12`, `:122-123`, `:369-372`).

**Recommend.** §3.3 step 15 + §8.4 are already right. Enforce mechanically: **exactly one** `frame.set_cursor_position` call site in the whole workspace, in `Runtime::draw`; components call `ui.set_cursor(owner, pos)` only. Grep rule 23.

### 2.9 `Stylize` — deliberately **banned**, contrary to the usual advice

**[F] Now:** no `Stylize` use anywhere; styles are built as `Style::new().fg(..).bg(..).add_modifier(..)` (`theme.rs:229-291`, `:329-359`; `button.rs:127-132`; `list.rs:253-269`; `viewport.rs:615-627`).
**[F] Modern:** `Stylize` gives `"hello".red().on_blue().bold()` and is implemented for every `Styled` type (`ratatui-core-0.1.2/src/style.rs:26-56`, `:76`).

**Recommend: keep it out.** Every `Stylize` colour shorthand names a literal ANSI colour, which is precisely what goal §15 forbids ("reusable components must not contain literal palette colors") and what §11.3 forbids structurally (roles bind to colours only at the end of resolution, A2). The only legitimate style source inside a component is `ui.style(family, variant, part, flags) -> Resolved`. Ban `Stylize` in `crates/tui/src/**` and `apps/**/src/**` (grep rule 10); it may appear in `crates/tui-testing` fixtures and doctests where a literal colour is the point. This is a deliberate inversion of the common ratatui idiom and must be documented as such, or a reviewer will "modernise" it back.

Also standardise on **`Style::new()`**, never `Style::default()` — one spelling (grep rule 8).

### 2.10 `Style::patch`

**[F] Now:** unused. `theme.rs` layers by successive assignment (`st = st.bg(...)`, `st = st.fg(...)`, `.add_modifier(...)`) — e.g. `theme.rs:333-357`, which cannot express "remove BOLD" and loses `sub_modifier` entirely.
**[F] Modern:** `Style::patch(other)` is the canonical layering operation, used internally by `Buffer::set_line` (`buffer.rs:385`), and it carries correct `add_modifier`/`sub_modifier` semantics.

**Recommend.** `StylePatch::merge` (§11.3) is a *role-level* merge and is not replaceable by `Style::patch`. But the **final** step — applying `Resolved.style` over the inherited surface style — must be `inherited.patch(resolved.style)`, because §11.3's "modifier symmetry" merge law (a later `remove(BOLD)` beats an earlier `add(BOLD)`) is exactly `Style::patch`'s semantics. Pin it: `theme::patch_merge_matches_ratatui_style_patch_for_modifiers`.

### 2.11 Layout: `Layout`, `Flex`, `Spacing`, `Constraint`, `Layout::areas::<N>()`, `Rect::centered`, `Rect::clamp`

**[F] Modern:** `Layout::areas(area) -> [Rect; N]` for compile-time-known splits, `Layout::split` for runtime counts (`ratatui-core-0.1.2/src/layout.rs:97-112`); `Flex::{Start, End, Center, SpaceBetween, SpaceAround, SpaceEvenly, Legacy}` (`:206-216`); `Layout::spacing(Spacing::Space(n))` and `.margin(n)` (`:241-246`); `Constraint::{Min, Max, Length, Percentage, Ratio, Fill}` resolved in that priority order (`:193-202`); `Rect::centered / centered_horizontally / centered_vertically` (`src/layout/rect.rs:54-59`); `Rect::clamp(other)`, `intersection`, `union`, `inner(Margin)`, `outer`, `offset`, `resize`, `is_empty`, `area`, `contains`, `intersects`, `rows()`, `columns()`, `positions()` (`rect.rs:38-67`); `Rect::ZERO` (`rect.rs:153`).

**Recommend — split the decision.**
* **Keep §10's hand-written `layout::{rows, columns, responsive_columns, action_row, inset, split_v, split_h}`.** `Track::{Fixed, Flex, Auto}` is a 3-case integer distribution: deterministic, 0-alloc for the fixed-size cases, and it expresses `Auto` (content-derived) which `Constraint` cannot. Routing it through the Cassowary solver would allocate per call, need `layout-cache` to be fast, and reintroduce a global cache. Document "no constraint solver" (already in §10) and keep it.
* **But stop re-implementing what `Rect` already does.** `dialog.rs:376`'s `Rect::centered` (slated for deletion in §18.2) is replaced by `Rect::centered(Constraint, Constraint)` from core. `Anchor::Screen(ScreenAlign)` resolution uses `centered_horizontally`/`centered_vertically`. §9.1's "flip **then clamp**" clamp step is `Rect::clamp`, not fresh min/max arithmetic. `layout::inset` uses `Rect::inner(Margin)` for the symmetric case and keeps `Insets` only for the asymmetric one. `Rect::ZERO` replaces `Rect::new(0,0,0,0)`.
* **Borrow the vocabulary, not the engine.** Name `RowAlign` values so they read against `Flex` (`RowAlign::{Start, End}` rather than `{Left, Right}`) and call the gap parameter `spacing`, so a ratatui-literate reader is not surprised. `layout::action_row(area, widths, gap, RowAlign::End)` is the exact analogue of `Layout::horizontal(...).flex(Flex::End).spacing(Spacing::Space(gap))`; say so in the rustdoc.
* Ban `Layout::` / `Constraint::` / `Flex::` inside `crates/tui/src/components/**` (grep rule 24) — that is what keeps `layout-cache` unnecessary and the perf baseline honest.

### 2.12 Symbols: replace our bespoke `BorderSet` and scrollbar consts

**[F] Now:** `src/widgets/scrollbar.rs:8-9` `pub const TRACK: &str = "│"; pub const THUMB: &str = "┃";`. §11.2 declares `pub struct BorderSet { pub tl, tr, bl, br, h, v: &'static str }` and `GlyphSet` entries `ScrollTrack`/`ScrollThumb`.

**[F] Modern:**
* `ratatui_core::symbols::border::Set<'a>` has **eight** fields — `top_left`, `top_right`, `bottom_left`, `bottom_right`, `vertical_left`, `vertical_right`, `horizontal_top`, `horizontal_bottom` — with `Default = PLAIN` and consts `PLAIN`, `ROUNDED`, `DOUBLE`, … (`ratatui-core-0.1.2/src/symbols/border.rs:3-71`).
* `ratatui_core::symbols::scrollbar::Set<'a> { track, thumb, begin, end }` with `VERTICAL`, `HORIZONTAL`, `DOUBLE_VERTICAL`, `DOUBLE_HORIZONTAL` (`src/symbols/scrollbar.rs:12-46`). Note the built-in thumbs are `block::FULL` (`█`), **not** our `┃`.
* `symbols::line`, `symbols::block`, `symbols::shade`, `symbols::merge::MergeStrategy` also available.

**Recommend.**
* **Delete `BorderSet` and alias it:** `pub type BorderSet = ratatui_core::symbols::border::Set<'static>;`. Ours is strictly narrower (6 fields vs 8) and cannot express asymmetric top/bottom or left/right runs. Aliasing lets a theme author hand us `symbols::border::ROUNDED` / `PLAIN` / `DOUBLE` directly instead of retyping six glyphs, and `ThemeBuilder::borders_set(BorderSet::SQUARE)` (§17.0 A5) becomes `borders_set(symbols::border::PLAIN)`. Junie's rounded set is `symbols::border::ROUNDED` verbatim — verify against the current output before blessing.
* Keep the **Junie values** for the scrollbar (`│`/`┃`, which none of the built-in sets match) but type them as `symbols::scrollbar::Set<'static>`, so `GlyphRole::{ScrollTrack, ScrollThumb}` resolve from a typed set rather than two loose consts. `scrollbar.rs:8-9` deleted.
* Same for rules/seams: `symbols::line::Set`.

### 2.13 `Scrollbar` / `ScrollbarState` — rejected

**[F]** `ratatui_widgets::scrollbar::{Scrollbar, ScrollbarState, ScrollbarOrientation, ScrollDirection}` (`ratatui-0.30.2/src/widgets.rs:684-686`).

**Reject**, for three structural reasons, not taste: (a) it is a `Widget` that paints into a `Buffer` and cannot register `Part::TRACK` / `Part::THUMB` hit regions, which §12.2 requires so the thumb can be dragged through pointer capture (§8.2); (b) it cannot resolve through `ui.style(SCROLLBAR, variant, Part::THUMB, flags)`, so it would bypass the recipe system entirely (G6); (c) `ScrollbarState` would be a second source of truth beside `ScrollState` (§18.1), which is the defect class this refactor removes. Adopt only its **symbol sets** (§2.12). Add `Scrollbar|ScrollbarState|ScrollbarOrientation` to the forbidden grep.

### 2.14 `Block`, `Padding`, titles

**[F]** `Block` gained `title_top` / `title_bottom` / `border_set` / `merge_borders` / `shadow` in 0.30 (`ratatui-widgets-0.3.2/src/block.rs:49-60`); the block module exports `Padding`, `CellEffect`, `Dimmed`, `Shadow`, `dimmed` (`:20-21`) and `ratatui` re-exports `Block, BlockExt, CellEffect, Dimmed, Padding, Shadow, TitlePosition, dimmed` (`ratatui-0.30.2/src/widgets.rs:669-671`). **There is no `Title` type** — any recollection of `ratatui::widgets::block::Title` with an alignment field is stale; titles are `Line`s and alignment lives on the line.

**Recommend.** All of it stays out with `ratatui-widgets`. `Ui::frame(area, style) -> Rect` (§17.0 A2) draws the theme `BorderSet` and returns the inner rect — the 20 lines that `Block::bordered().inner()` would have given us, but participating in the clip rect, the written-cell bitset, and role resolution. `Padding` is replaced by `Insets` (§10). Record the swap so nobody reaches for `Block::bg(...)`.

`Shadow` is the one genuinely tempting piece for popovers; leave it out for now and note it as a candidate if §9's popover chrome ever wants a drop shadow — at which point taking `ratatui-widgets` for one widget must be re-argued.

### 2.15 Deprecated / trap APIs

**[F]** `Buffer::get` and `Buffer::get_mut` are `#[deprecated]` in 0.30 with the message "use `Buffer[(x, y)]` instead. To avoid panicking, use `Buffer::cell`/`cell_mut`" (`buffer.rs:128-134`, `:148-154`). Our code already uses `cell_mut` (`ctx.rs:138`) — correct. Indexing `buf[(x,y)]` / `buf[pos]` panics on OOB (`buffer.rs:511-523`); `cell`/`cell_mut` return `Option` (`:179-214`).

**[F] Security trap — `ratatui_core::text::Masked`:**
```rust
// ratatui-core-0.1.2/src/text/masked.rs:50-56
impl fmt::Debug for Masked<'_> {
    /// Debug representation of a masked string is the underlying string
    fn fmt(&self, f) -> fmt::Result { fmt::Display::fmt(&self.inner, f) }   // ← raw secret
}
```
`Display` masks; **`Debug` prints the secret verbatim.** Any `Masked` reachable from a `#[derive(Debug)]` struct leaks. Goal §19 and §29 both forbid this.

**Recommend.** Forbid `ratatui_core::text::Masked` in the library and the apps (grep rule 11). `Secret` + `SecretPolicy` (§15) with a manual redacting `Debug` and a **synthetic** tail is the only masking path; `conformance::secret_never_appears_in_debug` (§16.2 case 18) already covers it — extend the fixture to assert that no `Masked` is constructible from a `Secret`.

### 2.16 `Text` / `Line` / `Span` / `ToLine` / `ToSpan`

**[F]** `ratatui_core::text` exports `Line`, `Span`, `Text`, `StyledGrapheme`, `ToLine`, `ToSpan`, `ToText` (`src/text.rs:51-64`). `Buffer::set_line(x, y, &Line, max_width) -> (u16,u16)` clips per span and applies `line.style.patch(span.style)` (`buffer.rs:373-392`); `Buffer::set_span` likewise (`:395-397`).

**[F] Now:** `src/widgets/viewport.rs:22-31` defines its own `Span { text: String, tone: Tone, bold, italic, underline, reversed }`.

**Recommend.**
* **Keep our own span type.** It stores a `Tone`/`Role`, not a resolved `Style` — required so a viewport re-themes without rebuilding and so `Ui::dim_layer` can walk roles (§11.6). §18.2 already re-specifies its storage as `{ range: Range<u32>, w: u8, style_ix: u16 }`; that also fixes the per-cell `String` at `viewport.rs:91`.
* **But use `Buffer::set_line` for the multi-span paint step.** `RowUi::label_spans(&[Span<'_>])` (§12.2) should build a borrowed `ratatui_core::text::Line<'_>` and call `set_line`, instead of the hand-written per-span cursor at `viewport.rs:609-638` — it gets per-span clipping and modifier patching for free and cannot drift from `set_stringn`'s width accounting.
* **Reject `ToSpan`/`ToLine`** as the `Display → row label` bridge: they go through `to_string()` and allocate. §17.0 A8's `RowUi::label_fmt(core::fmt::Arguments<'_>)` is the correct, allocation-free primitive and should be documented as the reason `ToSpan` is not used.

### 2.17 `Color::from_u32`

**[F] Now:** `src/theme.rs:59-65` hand-rolls `const fn rgb(hex: u32) -> Color`.
**[F] Modern:** `Color::from_u32(0x00RRGGBB)` is `const` and unfeatured (`ratatui-core-0.1.2/src/style/color.rs:133-138`). There is **no** `Color::from_hsl` in 0.30 — the `palette` feature adds only `From<Srgb>` / `From<LinSrgb>` (`src/style/palette_conversion.rs:22-52`).

**Recommend.** Delete `mod palette::rgb` (`theme.rs:56-65`); write `Color::from_u32(0x48_E0_54)` in `theme/builtin/junie.rs` and `paper.rs`. Extend §16.5's `palette_literals_are_confined_to_theme_builtins` grep to include `Color::from_u32(` alongside `Color::Rgb(` and `#[0-9a-fA-F]{6}`, or the check will pass while every literal moves one call deeper.

### 2.18 Multi-width safety in `Ui::paint_cell`

**[F]** `Buffer::set_stringn` resets the cells shadowed by a multi-width grapheme (`buffer.rs:359-368`); the diff algorithm assumes "no double-width cell is followed by a non-blank cell" (`buffer.rs:476-477`).

**Recommend.** `Ui::paint_cell(pos, symbol, style)` (§17.0 A2) must replicate that reset, or a wide grapheme written cell-by-cell corrupts the diff. Make it a documented invariant and cover it in `render::components::*` with a CJK fixture.

---

## 3. Rust 2024 / MSRV-1.88 practice for the new crates

### 3.1 Language features — all available at 1.88, all already justified

| Feature | Available since | Use here | Evidence in repo |
|---|---|---|---|
| edition 2024 | 1.85 | workspace-wide | `Cargo.toml:4` today |
| let-chains | 1.88 (edition 2024) | already idiomatic in this codebase; keep | `core/text.rs:99-103`, `viewport.rs:656-663`, `runtime.rs:126-128`, `list.rs:285-288`, `button.rs:146-148` |
| `#[expect(lint, reason = "…")]` | 1.81 | **replaces every `#[allow]`** | policy, §3.2 |
| `core::error::Error` | 1.81 | `FieldError`, `LayoutError` (§13) — `core`, not `std`, to match ratatui (`Backend::Error: core::error::Error`, `backend.rs:162`; `impl core::error::Error for ParseColorError`, `color.rs:257`) | |
| RPIT in inherent fns | 1.75 | `FocusRing::reachable() -> impl Iterator<Item = &FocusEntry> + '_` (§8.1) | |
| RPITIT | 1.75 | **avoid** — it makes a trait non-dyn-safe, and §12.1 relies on `&dyn Fn` slots | |
| `impl Trait` in assoc. types | not stable | n/a; `IntentIter<'f>` (§17.0 A2) is correctly a **named** type — that is what lets it outlive the `&Cx` borrow (§21 item 6). Do not "modernise" it to `impl Iterator`. | |
| `std::sync::LazyLock` | 1.80 | **not in `crates/tui`** — see §3.2 | |

**No language feature the architecture needs is missing at 1.88.** Every `const fn` in §7.1/§17.0 (`Id::root/sub/part/index`, `Part::custom`, `ActionKey::custom`, `LayerSpec::*`, `Dismiss::*`, `Overlay::new`, `StylePatch::*`) operates on integers and `&'static str` and needs no const-trait support.

### 3.2 Attribute and structure policy

* **`#![forbid(unsafe_code)]`** in `crates/tui/src/lib.rs` and in every app crate root. **`crates/tui-testing` must use `#![deny(unsafe_code)]`**, not `forbid` — `forbid` cannot be overridden, and that crate carries the documented `unsafe impl GlobalAlloc` (§16.6), which needs a local `#[expect(unsafe_code, reason = "counting allocator; see SAFETY")]` plus a `// SAFETY:` comment. Correspondingly, `[workspace.lints.rust] unsafe_code` is **`deny`**, and `crates/tui` tightens it to `forbid` at the crate root. Setting `forbid` at workspace level would make `tui-testing` uncompilable.
* **`#![deny(missing_docs)]`** in `crates/tui` (already required by §16.5) plus `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` in CI (goal §26).
* **`#![doc = include_str!("../README.md")]`** at the top of `crates/tui/src/lib.rs`, so `cargo test --workspace --doc` compile-checks every README example. This is the cheapest way to satisfy goal §24 + §25.5 "all examples compile" for the quick-start. Caveat: every README code fence must then be valid Rust or tagged ```` ```text ```` / ```` ```ignore ````.
* **Doctests hide setup only.** In `#`-hidden lines put fixture construction (`# let mut ui = junie_tui::testing::ui();`); never hide a `use` that a real downstream caller would need. A doctest that only compiles because a hidden line imports a private path is a false proof of §17's "external consumer" claim.
* **`#[non_exhaustive]` — apply narrowly.** Yes on types the *runtime produces and the caller only matches*, which will grow: `Diagnostic`, `Intent`, `Phase`, `FocusVia`, `LayerEvent`, `DismissReason`, `Status`, `ColorLevel`, `RegionKind`, plus the already-marked `LayerSpec` (§9.1) and `LayoutFacts` (§17.0 A2). **No** on `ColorTokens` (§17 example 2 states the reason explicitly: a new token must be a compile error for downstream themes, and `map_colors`'s exhaustive destructure depends on it), `StylePatch`, `RowDecor`, `CellDecor`, `Insets`, `Headroom` — all constructed by users with struct literals. Note the trade-off out loud: `#[non_exhaustive]` on an enum forces a wildcard arm downstream, which *destroys* the "adding a variant is a compile error" property; apply it only where downstream exhaustiveness is not wanted.
* **No process-global state in `crates/tui/src`.** `LazyLock`, `OnceLock`, `static mut`, `thread_local!` are forbidden (grep rule 18). The `Runtime` owns all state (§4); a global would break `Harness` isolation and `--test-threads=1`-free testing. This is a second, independent reason `ratatui-core/layout-cache` stays off — it *is* such a global.
* **`[lints] workspace = true`** in every member; `[workspace.lints]` defined once at the root, so `architecture::msrv_and_edition_are_unchanged` and the lint policy each read one place.

### 3.3 `[workspace.lints]` — deny level without noise

```toml
[workspace.lints.rust]
unsafe_code                   = "deny"      # crates/tui raises to forbid at the crate root
missing_docs                  = "warn"      # crates/tui raises to deny at the crate root
unreachable_pub               = "warn"
missing_debug_implementations = "warn"
unused_qualifications         = "warn"
rust_2018_idioms              = { level = "warn", priority = -1 }

[workspace.lints.clippy]
all      = { level = "deny",  priority = -1 }   # correctness + suspicious + style + complexity + perf
pedantic = { level = "warn",  priority = -1 }

# denied individually — each maps to a stated invariant, not to taste
panic                     = "deny"   # goal §10 "no panics during normal interaction"
indexing_slicing          = "deny"   # the structural fix for §1.3's ragged-row panics
unwrap_used               = "deny"
expect_used               = "deny"
todo                      = "deny"   # goal §29 "no material TODO / stub"
unimplemented             = "deny"
dbg_macro                 = "deny"
print_stdout              = "deny"   # a TUI library must never write to stdout
print_stderr              = "deny"
undocumented_unsafe_blocks= "deny"
mem_forget                = "deny"

# warned, not denied — real signal, too noisy to block on
arithmetic_side_effects   = "warn"   # the rule is saturating_* (R5); deny would flood
missing_panics_doc        = "warn"
missing_errors_doc        = "warn"

# allowed with reasons
must_use_candidate        = "allow"  # noise; #[must_use] is applied deliberately (Response)
module_name_repetitions   = "allow"
cast_possible_truncation  = "allow"  # u16 terminal coordinates — ratatui itself allows these four
cast_sign_loss            = "allow"  #   (ratatui-core-0.1.2/Cargo.toml:179-182); staying
cast_precision_loss       = "allow"  #   consistent with the ecosystem we live inside
cast_possible_wrap        = "allow"
```

Notes:
* `clippy::all` at **deny** is the workhorse; `pedantic` at **warn** because `-D warnings` in CI (goal §26) promotes it to a hard failure anyway while leaving local development usable.
* **`nursery` is rejected** as a group: unstable lint set, churns between toolchains, and would break a pinned-MSRV CI job on a compiler upgrade unrelated to our code.
* **`restriction` is never enabled as a group** — only the individual lints listed above.
* `indexing_slicing = "deny"` is aggressive and is *the point*: it makes `grid.rs:1458/505` and `table.rs:220/294/317/700`-class panics impossible to reintroduce. Relax it for tests only, at the crate root: `#![cfg_attr(test, allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic))]`.
* The four `cast_*` allows are justified **by citation**, not by fatigue: ratatui-core, ratatui, and ratatui-crossterm all allow exactly these four for exactly this reason (`ratatui-core-0.1.2/Cargo.toml:179-182`).

### 3.4 `cargo-semver-checks`

Adopt, but **not during the refactor**. It compares against a published or git baseline; during a total public-API rewrite every check fails by construction and the signal is zero. Plan: add `xtask semver` wrapping `cargo semver-checks --baseline-rev <tag>` at the end of Slice 8, tag `v0.1.0`, and wire it into CI as a **blocking** gate from `v0.1.1` onward. Record it in §16.5 as a deferred check, not a shipped one.

---

## 4. `smallvec` and `bitflags`

### 4.1 `bitflags 2.13` — **adopt**

**[F]** `bitflags` is **already in the dependency graph**: `ratatui-core` depends on `bitflags = "2.12"` (`ratatui-core-0.1.2/Cargo.toml:100-101`) and uses it for `Modifier` (`src/style.rs:87-115`); crossterm 0.29 uses it for `KeyModifiers`, `KeyEventState`, `KeyboardEnhancementFlags` (`crossterm-0.29.0/src/event.rs:284-311`, `:832-848`, `:911-932`). Requesting `"2.13"` unifies to a **single** 2.13.1 copy — **zero new nodes in the tree**.

Uses: `StateFlags` (16 bits, §6.1), `Caps: u32` (§16.2). Beyond `|`/`&`/`contains`, the architecture actually needs:
* `when.count_ones()` for the §11.3 specificity ordering of state rules;
* a *readable* `Debug` (`FOCUSED | HOVERED`) — G5's "readable in a debugger and a test failure" applies to state as much as to `Id`, and every §16.2 assertion prints it;
* `iter()` / `iter_names()` for the mono-fallback table walk (§11.4) and `state_flags_round_trip` (§16.1);
* `from_bits_truncate`, `all()`, `Sub`, `Not`, `BitXor` for `live & (FOCUSED | FOCUS_VISIBLE | HOVERED)` masking (`COMPONENT_ARCHITECTURE.md:2674`).

Hand-rolling all of that correctly is ~200 lines of tedious, test-hungry code to avoid a dependency that is already compiled. **Adopt.** Pin `"2.13"` (≥ ratatui-core's `2.12` floor).

### 4.2 `smallvec` — **reject**

Architecture uses: `PartRecipe.states: SmallVec<[StateRule; 8]>`, `Recipe.variants: SmallVec<[(Variant, PartMap<PartRecipe>); 6]>` (§11.3), `KeySet::{Only, AllExcept}(SmallVec<[ItemKey; 8]>)` (§17.0 A8).

**[F]** `smallvec` is **not** in the current graph — none of ratatui-core's dependencies is smallvec (`ratatui-core-0.1.2/Cargo.toml:96-160`). It would be a genuinely new node. **[F]** smallvec 2.0 is alpha and therefore ineligible (given fact).

Four reasons to reject:

1. **The stated bounds are not real bounds.** `RecipeEdit::when` is a *public* builder (§17.0 A5) and `apply_mono_fallbacks` appends one rule per family (§11.4) — `states` can exceed 8. `define_variant` is public — `variants` can exceed 6. `KeySet` must hold a 5,000-row multi-selection over a 100k grid. All three are unbounded; SmallVec's inline array is a heuristic, not a guarantee.
2. **It buys nothing on any hot path.** `Recipes` is built **once**, at theme construction — never per frame. §16.6's `style_resolve_10k_parts` requires "exactly 0 allocs" during *resolution*, and resolution only **reads** these containers: `Vec` is equally 0-alloc there. `KeySet` mutates on input events, not per frame; `AllExcept(Vec::new())` is 0 allocs, satisfying `list_100k_select_all`'s "< 100 allocs" (R7).
3. **It makes `Theme` bigger and `Theme::clone()` more expensive — a measurable regression.** `StateRule = { when: StateFlags(u16), patch: StylePatch }` and `StylePatch` has eight fields, so `StateRule` ≈ 40 bytes. Eight inline ⇒ ~320 bytes **per `PartRecipe`**, × ~34 parts × ~34 families ≈ **hundreds of KB of inline storage per `Theme`** — most of it empty, and all of it copied by `Theme::downgrade`, which does `let mut out = self.clone()` (`COMPONENT_ARCHITECTURE.md:913`). With `Vec`, the same data is a 24-byte header per `PartRecipe` plus a heap block sized to the 0–3 rules actually present. SmallVec makes the common case worse.
4. Goal §21: "Keep dependencies focused."

**Recommendation, and a better fix than either option.** Use `Vec<StateRule>` and `Vec<(Variant, PartMap<PartRecipe>)>`. For `KeySet`, use `Vec<ItemKey>` **kept sorted**, so `contains` is a binary search. This addresses a complexity problem SmallVec does not touch: with an unsorted container, a 100k grid with 5,000 selected rows costs O(5000) per visible row per frame — ~200,000 comparisons/frame — which would breach `list_100k_rows_render`'s "ns ≤ 1.5× `list_1k_rows_render`" (§16.6) regardless of where the bytes live. Sorted `Vec` makes it O(40 · log 5000).

Add to §16.1: `key_set_stays_sorted_after_insert_remove_toggle_retain`, `key_set_contains_is_binary_search` (assert comparison count, not just the answer).

**Consequences for the architecture text (record in `REFACTORING_STATE.md`):** §11.3, §16.5 and §17.0 A8 must be amended — `SmallVec` is removed from `PartRecipe`, `Recipe`, `KeySet`; §16.5's `architecture::library_has_no_application_dependency` row currently lists the expected deps as `ratatui, unicode-width, unicode-segmentation, bitflags, smallvec` (`COMPONENT_ARCHITECTURE.md:1517`) and becomes `ratatui-core, ratatui-crossterm, unicode-width, unicode-segmentation, bitflags`.

---

## 5. MSRV

### 5.1 Facts

* Repository declares `rust-version = "1.88"`, `edition = "2024"` (`Cargo.toml:4-5`). §16.5 pins it with `architecture::msrv_and_edition_are_unchanged`; goal §21 says preserve unless "strongly justified, documented, and verified".
* **[F] Every direct dependency declares `rust-version = "1.88.0"`**: `ratatui-core-0.1.2/Cargo.toml:14`, `ratatui-0.30.2/Cargo.toml:14`, `ratatui-crossterm-0.1.2/Cargo.toml:14`. 1.88 is the ecosystem floor; raising ours buys nothing in dependency terms.
* Local toolchain is 1.98 (given fact).

### 5.2 What a bump would buy — and why it is nothing

Every language capability the accepted architecture depends on is already available at 1.88:

| Need (with the section that needs it) | Stable since |
|---|---|
| edition 2024 (workspace-wide) | 1.85 |
| let-chains — used throughout the intent-drain loops and §12.2 reconcile | 1.88 |
| `#[expect(lint, reason)]` — the §3.2 suppression policy | 1.81 |
| `core::error::Error` for `FieldError` / `LayoutError` (§13) | 1.81 |
| RPIT in inherent methods — `FocusRing::reachable` (§8.1) | 1.75 |
| `const fn` over integers / `&'static str` — `Id`, `Part`, `ActionKey`, `LayerSpec`, `Dismiss`, `Overlay`, `StylePatch` (§7.1, §17.0) | ≤1.61 |
| `LazyLock` — deliberately *not* used in the library (§3.2) | 1.80 |

The only capability that would materially change the design is **const trait support / `const fn` in traits**, which would let `StylePatch::merge` and a fully `const` `Overlay` be evaluated at compile time (§11.3, §17.0 A5). That is **not stable in any released Rust up to and including the 1.98 toolchain in use**, so no bump reaches it.

### 5.3 Recommendation — **keep `rust-version = "1.88"`**, and actually verify it

"Latest dependency versions and their latest APIs" is fully satisfied at 1.88: the latest ratatui, the latest crossterm, edition 2024, let-chains, `#[expect]`, `core::error::Error`. Raising the MSRV would exceed every dependency's own floor and buy no feature the architecture uses — a change with cost and no benefit, which goal §21 forbids.

**But the current pin is a declaration, not a fact.** §16.5's `msrv_and_edition_are_unchanged` reads `cargo metadata`; it proves the *field*, not that the code compiles on 1.88. On a 1.98 toolchain a builder can use a 1.95 API and every gate passes. Close it:

* CI job `msrv`: `cargo +1.88.0 check --workspace --all-targets --all-features` (or `cargo-msrv verify`). Blocking.
* Optionally add `rust-toolchain.toml` with `channel = "1.88.0"` for a `--locked` verification profile — but keep the *primary* CI job on stable, so new compiler diagnostics are seen early.
* Record in `REFACTORING_STATE.md` that the MSRV was re-examined during the refactor and deliberately held, with this section as the justification — goal §21 requires the decision be documented, not merely defaulted to.

---

## 6. Binding rules for builders + the architecture check

### 6.1 Modern-API rules (binding; each names the exact path)

| # | Rule | Exact API |
|---|---|---|
| R‑1 | **One width function.** All display width goes through `crates/tui/src/text/measure.rs::width`, which delegates to `CellWidth::cell_width`. No file outside that one may import `unicode_width`. | `ratatui_core::buffer::CellWidth` (`ratatui-core-0.1.2/src/buffer/cell_width.rs:19-46`) |
| R‑2 | **Never pre-truncate for painting.** Use the clipping writer; it returns the end column. `fit`/`fit_right` are banned on every render path. | `Buffer::set_stringn` (`buffer.rs:336-370`) |
| R‑3 | **Multi-span rows use the line writer**, not a hand-rolled span cursor. | `Buffer::set_line` (`buffer.rs:373-392`) |
| R‑4 | **No nested `for y … for x` over a rect.** Iterate positions, or restyle wholesale. | `Rect::positions()` / `rows()` / `columns()` (`layout/rect.rs:6`, `:65-67`); `Buffer::set_style(area, style)` (`buffer.rs:405-413`) |
| R‑5 | **Cells are reached by `Option`, never by the deprecated accessors.** | `Buffer::cell` / `cell_mut` (`buffer.rs:179-214`); `Buffer::get`/`get_mut` are `#[deprecated]` (`:129`, `:149`) |
| R‑6 | **`Ui::paint_cell` must reset the cells shadowed by a wide grapheme**, matching the writer and the diff assumption. | `buffer.rs:359-368`, `:476-477` |
| R‑7 | **One cursor write per frame**, in `Runtime::draw`. Components call `ui.set_cursor(owner, pos)`. | `Frame::set_cursor_position` (`ratatui-core/src/terminal.rs:11-12`, `:369-372`) |
| R‑8 | **No style literal outside the theme.** Every style comes from `ui.style(family, variant, part, flags) -> Resolved`. `Stylize` shorthands are banned in library and app code. Spell `Style::new()`, never `Style::default()`. | `ratatui_core::style::{Style, Stylize}` (`src/style.rs:74-76`, `:131-139`) |
| R‑9 | **Layer a style with `patch`, never by field reassignment** — it is the only form with correct `sub_modifier` semantics. | `Style::patch` (used at `buffer.rs:385`) |
| R‑10 | **Colour literals are `Color::from_u32`, and only inside `theme/builtin/`.** | `Color::from_u32` (`style/color.rs:133-138`). There is no `Color::from_hsl`. |
| R‑11 | **Border and scrollbar glyph sets are ratatui symbol sets**, not bespoke structs. `BorderSet` is a type alias. | `symbols::border::{Set, PLAIN, ROUNDED, DOUBLE}` (`symbols/border.rs:3-71`); `symbols::scrollbar::{Set, VERTICAL}` (`symbols/scrollbar.rs:12-46`); `symbols::line` |
| R‑12 | **Reuse `Rect` geometry; do not re-derive it.** Centering, clamping, margins, emptiness, zero. | `Rect::{centered, centered_horizontally, centered_vertically, clamp, intersection, union, inner, outer, offset, is_empty, ZERO}` (`layout/rect.rs:38-67`, `:153`) |
| R‑13 | **No constraint solver inside components.** `Layout`/`Constraint`/`Flex` may not appear under `components/**`; use `layout::{rows, columns, action_row, inset}`. (This is what keeps `layout-cache` off.) | `ratatui_core::layout::{Layout, Constraint, Flex, Spacing}` (`src/layout.rs:324-333`) — vocabulary reference only |
| R‑14 | **Crossterm is reached only through the backend re-export**, never a direct `crossterm` dependency. | `ratatui_crossterm::crossterm` (`ratatui-crossterm-0.1.2/src/lib.rs:86-98`, rationale `:16-40`) |
| R‑15 | **Input normalisation matches `MouseEventKind` exhaustively** (no `_` arm) and carries `modifiers`. | `crossterm::event::{MouseEvent, MouseEventKind}` (`crossterm-0.29.0/src/event.rs:777-817`) |
| R‑16 | **Key press/repeat detection uses the accessors**, not hand-matched `kind`. | `Event::as_key_press_event`, `KeyEvent::is_press/is_repeat/is_release` (`crossterm-0.29.0/src/event.rs:666-689`, `:1009-1022`) |
| R‑17 | **No keyboard-enhancement flags.** They are a rejected design, not an oversight. | `PushKeyboardEnhancementFlags` / `PopKeyboardEnhancementFlags` / `KeyboardEnhancementFlags` (`crossterm-0.29.0/src/event.rs:284-311`, `:493`, `:522`) |
| R‑18 | **Terminal modes are typed commands, never raw escape strings.** Mouse capture and bracketed paste are ours to enable; raw mode, alt screen and the chained panic hook mirror `try_init`. | `crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, EnableLineWrap, DisableLineWrap}`, `event::{EnableMouseCapture, EnableBracketedPaste}` (`crossterm-0.29.0/src/event.rs:318`, `:421`); reference implementation `ratatui-0.30.2/src/init.rs:397-399`, `:182-197` |
| R‑19 | **`ratatui_core::text::Masked` is forbidden** — its `Debug` prints the raw secret. `Secret` is the only masking path. | `ratatui-core-0.1.2/src/text/masked.rs:50-56` |
| R‑20 | **Nothing from `ratatui`, `ratatui-widgets`, or `ratatui-macros`** enters the workspace. `Block`, `Padding`, `Clear`, `Fill`, `Scrollbar`, `ScrollbarState`, `Shadow`, `Dimmed`, `Paragraph`, `List`, `Table` are re-implemented on `Ui` or deliberately absent, with the reason recorded. | `ratatui-0.30.2/src/widgets.rs:668-691`; `ratatui-widgets-0.3.2/src/lib.rs:115-138` |

### 6.2 `architecture::no_deprecated_or_legacy_api_usage`

New test in `crates/tui/tests/architecture.rs`, driven from `xtask` (needs to read the whole workspace). Scans `crates/tui/src/**`, `crates/tui-testing/src/**`, `apps/**/src/**`. Allow-list file `crates/tui/tests/allow/legacy_api.txt`; every entry requires a justification comment on the same line, which the test **prints on failure and on success**, so a growing allow-list is visible in CI output.

| # | Forbidden pattern (regex) | Where allowed | Why |
|---|---|---|---|
| 1 | `Buffer::get\b\|Buffer::get_mut\b` | nowhere | deprecated (`buffer.rs:129`, `:149`) |
| 2 | `enable_raw_mode\|disable_raw_mode` | `crates/tui/src/runtime/session.rs` | R‑18 |
| 3 | `EnterAlternateScreen\|LeaveAlternateScreen\|Enable(Mouse\|Bracketed)\|Disable(Mouse\|Bracketed)` | same file | R‑18 |
| 4 | `\\x1b\[\|\\u\{1b\}\[` | nowhere | raw ANSI; replaces `runtime.rs:42`, `:91` |
| 5 | `KeyboardEnhancementFlags` | nowhere | R‑17 |
| 6 | `for\s+\w+\s+in\s+\w+\.top\(\)\.\.\|\.left\(\)\.\.` | `crates/tui/src/ui/paint.rs` | R‑4; kills `ctx.rs:136-137` |
| 7 | `Rect::new\(` | `crates/tui/src/{layout.rs,ui/**}`, tests | components receive rects; kills `list.rs:236`, `viewport.rs:645`, `button.rs:110` |
| 8 | `Style::default\(\)` | nowhere | R‑8, one spelling |
| 9 | `\.fg\(\|\.bg\(\|add_modifier\(\|remove_modifier\(\|underline_color\(` | `crates/tui/src/theme/**`, `crates/tui/src/ui/paint.rs` | R‑8; kills `button.rs:127-159`, `list.rs:253-296`, `viewport.rs:615-627` |
| 10 | `style::Stylize\|\.(red\|green\|blue\|yellow\|magenta\|cyan\|white\|black\|gray\|on_[a-z]+)\(\)` | `crates/tui-testing/**`, doctests | R‑8 |
| 11 | `Masked\b` | nowhere | R‑19 |
| 12 | `unicode_width::\|UnicodeWidth(Str\|Char)` | `crates/tui/src/text/measure.rs` | R‑1 |
| 13 | `unicode_segmentation::` | `crates/tui/src/text/**` | one segmentation site |
| 14 | `\bratatui::\|ratatui_widgets::\|ratatui_macros::` | nowhere | R‑20 |
| 15 | `Scrollbar\b\|ScrollbarState\|ScrollbarOrientation\|ScrollDirection` | nowhere | §2.13 |
| 16 | `Block::\|Paragraph::\|Padding::\|BorderType::\|Borders::\|Clear\b\|Fill::new\|Shadow::\|Dimmed\b` | nowhere | R‑20 |
| 17 | `#\[allow\(` | `crates/tui-testing/**` (documented) | `#[expect(…, reason=…)]` only (§3.2) |
| 18 | `LazyLock\|OnceLock\|static mut\|thread_local!` | `crates/tui-testing/**`, `xtask/**` | no process-global state (§3.2) |
| 19 | `\.unwrap\(\)\|\.expect\(\|panic!\|todo!\|unimplemented!` | `#[cfg(test)]`, `crates/tui-testing/**`, `xtask/**` | goal §10; belt-and-braces beside clippy |
| 20 | `fn render\(&mut self\|fn draw\(&mut self` | nowhere under `components/**` | G2; companion to `draw_takes_shared_self` |
| 21 | `bg:\s*(ratatui_core::style::)?Color` in any `pub fn` | `Role::Custom` only | §16.5 existing rule, restated |
| 22 | `Color::Rgb\(\|Color::from_u32\(\|#[0-9a-fA-F]{6}` | `crates/tui/src/theme/builtin/{junie,paper}.rs`, `tests/fixtures/**` | §16.5 existing rule + R‑10 (the `from_u32` arm is new and necessary) |
| 23 | `set_cursor_position` | `crates/tui/src/runtime.rs` | R‑7 |
| 24 | `Layout::\|Constraint::\|Flex::\|Spacing::` | nowhere under `components/**` | R‑13 |
| 25 | `\.child\(\|\.owns\(\|\.locate\|scrollbar::id_for\|WidgetId` | nowhere | §16.5 existing rule, extended to `WidgetId` |
| 26 | `SmallVec\|smallvec::` | nowhere | §4.2 |

**Companion check, `architecture::dependency_graph_is_exactly_the_declared_set`** (`xtask`, `cargo metadata`):

1. `junie-tui`'s direct normal dependencies are exactly `{ratatui-core, ratatui-crossterm, unicode-width, unicode-segmentation, bitflags}`.
2. `ratatui`, `ratatui-widgets`, `ratatui-macros`, `smallvec`, `crossterm` (as a *direct* dep), `critical-section`, `palette` are absent from the normal dependency closure of `junie-tui`.
3. Each app's direct normal dependencies are exactly `{junie-tui}`.
4. `unicode-width`, `unicode-segmentation` and `bitflags` each resolve to **one** version in the graph (a second `unicode-width` would mean two disagreeing width tables — R‑1's correctness rests on this).
5. Enabled features on `ratatui-core` are exactly `{std, underline-color}`; `layout-cache`, `palette`, `scrolling-regions`, `serde`, `anstyle`, `portable-atomic` are off.

**Companion CI gates** (beyond goal §26's list):

* `cargo check -p junie-tui --no-default-features` — proves the paint/theme/layout/collection core is backend-free (§1.2).
* `cargo +1.88.0 check --workspace --all-targets --all-features` — makes the declared MSRV a fact (§5.3).
* `cargo test --workspace --doc` with `#![doc = include_str!("../README.md")]` — makes the README a compiled artifact (§3.2).

---

## 7. Risks

1. **Losing `ratatui::init`/`restore`/`run` is a real cost of the `ratatui-core` decision.** `TerminalSession` must be maintained by hand and kept in sync with upstream's hook ordering (`ratatui-0.30.2/src/init.rs:196-197`). Mitigation: mirror `try_init` literally, cite it in the module docs, and add `runtime::panic_hook_restores_before_delegating` to §16.1.
2. **The `CellWidth` switch will move rendered output.** Any string containing U+FF9E/U+FF9F now measures one column wider than before; §16.3 baselines must be re-blessed **with a `docs/visual-changes.md` entry** under §20.10, not silently. Realistically zero cells change in the three apps' fixtures, which makes the diff a cheap confirmation rather than a review burden.
3. **`indexing_slicing = "deny"` will be loud on first application** to migrated code, especially the grid. That is the intended signal (§1.3), but it is front-loaded work in Slice 3–4; do not weaken it to `warn` under time pressure — the whole point is that it is a compile error.
4. **Removing `SmallVec` changes `ColorTokens`/`Recipes` sizes**, which shifts the perf baseline. Take the §16.6 pre-refactor baseline on the *unmodified* tree (Appendix A WP-0 already requires this) and record the `Vec` decision as one of the "after" explanations.
5. **The dev-dependency cycle `junie-tui` ⇄ `junie-tui-testing`** is legal but occasionally surprises tooling (`cargo tree`, some IDEs). If it becomes a problem, the fallback is to move the conformance driver into `crates/tui/tests/` with `#[cfg(test)]`-only helpers — at the cost of the `publish = false` isolation §16 wants. Prefer the cycle; document it.
6. **`#[non_exhaustive]` on `Diagnostic` and `Intent` forces wildcard arms downstream**, which weakens the "new variant is a compile error" property in app code. That is the deliberate trade (§3.2); apps must not rely on exhaustive matching of those enums, and `jackin`'s `Screen::update` intent loop already uses `_ => {}` (`COMPONENT_ARCHITECTURE.md:2661`).

## 8. Acceptance conditions (executable)

```bash
cargo +1.88.0 check --workspace --all-targets --all-features        # MSRV is a fact, not a field
cargo check -p junie-tui --no-default-features                      # core is backend-free
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test  --workspace --all-targets --all-features
cargo test  --workspace --doc                                       # README + §17 examples compile
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo test  --workspace --test architecture                         # incl. the two new checks below
cargo tree  -p showcase -e normal --depth 1                         # => junie-tui, and nothing else
cargo tree  -p junie-tui -e normal | grep -E 'ratatui-widgets|ratatui-macros|^ratatui |smallvec'  # => no matches
cargo test  -p junie-tui --lib text::width_matches_ratatui_cell_width
cargo test  --workspace --test architecture no_deprecated_or_legacy_api_usage
cargo test  --workspace --test architecture dependency_graph_is_exactly_the_declared_set
```

Pass condition for the two new architecture tests: `crates/tui/tests/allow/legacy_api.txt` is **empty**, and the five `dependency_graph_is_exactly_the_declared_set` assertions in §6.2 all hold.
