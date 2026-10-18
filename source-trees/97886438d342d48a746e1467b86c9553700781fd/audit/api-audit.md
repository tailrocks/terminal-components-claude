# Historical API Audit — reusable layer (`src/core`, `src/ui`, `src/theme.rs`, `src/runtime.rs`, `src/widgets/*`)

> Historical snapshot. This audit records the pre-migration reusable layer and
> preserves its findings and citations. The `src/...` paths below are historical
> evidence, not a current inventory. Current reusable code is under `crates/tui/`;
> current app code is under `apps/`, including the Jackin environment domain at
> `apps/jackin-preview/src/domain/workspace.rs`.

**Historical scope of this pass:** every file under `src/core/`, `src/ui/`, `src/theme.rs`, `src/runtime.rs`, `src/lib.rs`, and all 31 modules under `src/widgets/`, plus `README.md` and `DESIGN.md` (skim). Application directories (`src/bin/showcase`, `src/bin/tablepro`, `src/bin/jackin_preview`) were **not** scanned in this historical pass except for their test harness files; goal §7's "search the application directories" checklist and the "application-specific copies or variants" inventory column remain **open** in that snapshot and must be covered by a separate audit.

**Convention:** `[F]` = collected fact with citation. `[I]` = inference/judgement. Every `file:line` was read directly.

---

## 1. Foundation summary

### 1.1 `src/core/event.rs`

**Responsibility [F]** Normalise crossterm input into a widget-facing `Input`, and define the single consumption/redraw signal `Outcome`.

**Key public API [F]**
- `enum Outcome { Ignored, Consumed, Changed }` — `core/event.rs:14-22`; `Outcome::consumed()` `:25`; `Outcome::or()` `:30`.
- `enum Input { Key, Mouse, Resize(u16,u16), Paste(String), Tick }` — `:40-46`.
- `struct Key { pub code: KeyCode, pub mods: KeyModifiers }` — `:49-52`; predicates `ctrl/shift/alt/plain/is/is_char/ctrl_char` `:55-75`.
- `enum MouseKind { Move, Down, Up, Drag, Secondary, WheelUp, WheelDown, WheelLeft, WheelRight }` — `:79-90`.
- `struct Mouse { kind, pos }` — `:92-96`.
- `Input::from_crossterm(Event) -> Option<Input>` — `:99-134`.

**Invariants [F]**
- `Changed` dominates in `or()` (`:30-36`); only `Changed` triggers a redraw in the loop (`runtime.rs:126-131`).
- Only `KeyEventKind::Press | Repeat` become `Input::Key` (`:104`); other kinds and unmapped mouse kinds are dropped (`:123`).
- `Key::plain()` deliberately ignores SHIFT (`:65`), so `is_char('A')` matches a shifted `A`.

**Weaknesses [I]**
- `Outcome` conflates "did you consume it" with "must I repaint". There is no channel for a semantic action, so 20 of 31 widget modules invent a second return slot (see §3.1).
- Only left-button `Down/Up/Drag` are modelled; right-button up/drag and middle-button are silently dropped (`:115-122`), so pointer capture for a secondary drag is impossible.
- No modifier information survives on `Mouse` (`:92-96`), so Shift-click range selection cannot be expressed at the event layer — `DataGrid` fakes it via drag only (`grid.rs:1332-1353`).
- `Input::Paste(String)` carries an owned `String` per paste; no bracketed-paste target is identified, so every owner must guess which control is editing.

### 1.2 `src/core/focus.rs`

**Responsibility [F]** A per-frame ordered ring of focusable ids plus one global "current" id.

**Key public API [F]** `FocusRing { order: Vec<WidgetId>, barrier: Option<usize> }` `:10-13`; `register` `:16`; `push_barrier` `:20`; `reachable` `:24`; `contains` `:31`; `first` `:35`; `next` `:39`; `prev` `:50`. `Focus { current: Option<WidgetId> }` `:64-66`; `current` `:69`; `is` `:73`; `set` `:77`; `focus` `:81`; `next/prev` `:85/:89`; `ensure_valid` `:94`.

**Invariants [F]**
- Ring is rebuilt each frame in render order → Tab order == reading order (module doc `:1-6`).
- Exactly **one** barrier is supported: `push_barrier` overwrites (`:20-22`), and `reachable()` slices from it (`:24-29`).
- `ensure_valid` falls back to `first()` when the focused id vanished (`:94-98`).

**Weaknesses [I]**
- **One barrier only** — nested overlays (dialog → picker, menu inside dialog) cannot each trap focus. Goal §14/Scenario F is unimplementable on this primitive.
- No focus *scopes*, no roving-focus/inner-cursor concept: composite widgets fake an internal cursor by holding a separate `cursor: usize` field (`list.rs:49`, `chips.rs:38`, `tabs.rs:57`, `grid.rs:338`…).
- No focus **restoration** stack — the applications must remember the previous id themselves.
- `ensure_valid` is the only reconciliation, and it jumps to the *first* stop rather than the nearest survivor; goal §11 "disappearing focused elements must produce a defined focus transition" is only weakly met.
- `Focus` and `FocusRing` are fully public mutable state that widgets receive directly in one case (`dialog.rs:207-212`, `:324-329`), breaking the "widgets never read global state directly" rule stated in `ui/ctx.rs:1-5`.

### 1.3 `src/core/hit.rs`

**Responsibility [F]** Per-frame registry of rectangles → `WidgetId`, with a scroll-only flavour and one barrier.

**Key public API [F]** `HitRegion { id, area, scroll_only }` `:13-19`; `HitRegistry::register` `:30`; `register_scroll` `:41`; `push_barrier` `:53`; `hit` `:65`; `hit_scroll` `:75`; `len` `:83`; `is_empty` `:87`; `area_of` `:91`.

**Invariants [F]**
- Empty rects are ignored (`:31-33`, `:42-44`).
- Later registration wins (`.rev()` at `:68`, `:78`) — documented as "overlays naturally shadow" (`:1-6`).
- `hit()` skips `scroll_only`; `hit_scroll()` does not (`:69`, `:79`).

**Weaknesses [I]**
- Again **one barrier** (`:53-55` overwrites), so nested popups cannot each be a pointer barrier. `ui/popup.rs:74` and `Dialog::render` (`dialog.rs:373`) both push one; whichever renders last wins and silently unblocks the other.
- Linear reverse scan per event (`:66-71`); O(regions) per mouse move. Grid registers one region **per visible cell** (`grid.rs:1878`) plus a duplicate row region, so a 40×12 viewport registers ~500 regions/frame.
- `area_of` returns the *last* registration for an id (`:91-97`), which is how tests locate controls (`showcase/app_tests.rs:126-129`) — a public API whose meaning depends on render order.
- No notion of "which registration is the same logical control at a different z" and no pointer-capture concept; drag capture is reconstructed by the app from `Interaction::pressed`.

### 1.4 `src/core/id.rs`

**Responsibility [F]** 64-bit FNV-1a hash of a path string, with `child(index)` and `sub(name)` derivation.

**Key public API [F]** `WidgetId(u64)` `:10`; `of(&str)` `:27`; `child(usize)` `:32`; `sub(&str)` `:37`; `Debug` prints only the hex hash `:42-46`.

**Invariants [F]** Stable across frames for the same path; `child(0) != child(1) != parent` (test `:53-59`).

**Weaknesses [I]**
- **`Debug` is unreadable** — `WidgetId(3f9a…)`, no path retained (`:42-46`). Goal §12 "readable debugging" fails today.
- **No collision detection.** 64-bit FNV over a hand-written namespace with no registry; goal §12 requires "collision detection or convincing collision safety" — neither exists.
- **`child(index)` is positional.** Every collection derives child ids from the *display* index (`grid.rs:1192` uses `display`, not source row; `tree.rs:262` uses the flattened row index, not the node `Path`). Insert/remove/reorder/scroll re-labels every child. Goal §11 "dynamic item identity must not depend only on a shifting numeric index" and Scenario E are violated by construction.
- No parent linkage, so nothing can answer "which component owns this id" — hence 12 hand-written `owns()` methods (§3.5).

### 1.5 `src/core/scroll.rs`

**Responsibility [F]** Pure scroll model: `offset`, `content_len`, `viewport_len`, plus thumb geometry.

**Key public API [F]** `ScrollState` `:8-12` (all three fields **public**); `new` `:16`; `max_offset` `:23`; `overflows` `:27`; `set_viewport` `:31`; `set_content` `:36`; `clamp` `:41`; `scroll_by` `:45`; `scroll_to` `:52`; `page_up/down` `:56/:60`; `jump_start/end` `:64/:68`; `ensure_visible` `:73`; `visible_range` `:86`; `thumb` `:92`; `offset_for_track_pos` `:107`.

**Invariants [F]** `offset ≤ max_offset` after every mutator (`:41-43`); `overflows()` requires a non-zero viewport (`:27-29`); `thumb` is guarded against `track_len == 0` and non-overflow (`:93-95`).

**Weaknesses [I]**
- Public fields let callers write `hscroll.offset` directly, bypassing `clamp` — done in `grid.rs:846`, `:848`, `:850-853` and `table.rs:275`, `:277`, `:279-282`.
- Reused for a **column** axis (`grid.rs:345`, `table.rs:110`) where `viewport_len` means "columns that fit"; `thumb`/`offset_for_track_pos` are meaningless there. One type, two semantics.
- No pixel/sub-line scrolling and no "nested scroll bubbles to parent" concept, so goal §13 "nested scrolling" is handled ad hoc by whoever routes the wheel.

### 1.6 `src/core/text.rs`

**Responsibility [F]** Grapheme-aware editable text buffer shared by input/textarea/code/table/grid editors.

**Key public API [F]** `TextBuffer { text: String, cursor: usize, anchor: Option<usize>, multiline: bool }` `:11-18` (fields private); `CursorPos { line, col }` `:21-25`; `single/multi` `:28/:38`; `text()` `:48`; `select_range` `:66`; `selection_lines` `:73`; `insert_at` `:93`; `remove_range` `:106`; `set_text` `:126`; `selection` `:132`; `select_all` `:145`; movement `:233-313`; editing `:332-393`; `line_count` `:397`; `cursor_pos` `:402`; `pos_of` `:406`; `offset_at` `:415`; `width` `:435`.

**Invariants [F]** Byte offsets always land on grapheme boundaries (`prev_boundary`/`next_boundary` `:171-185`); `insert_char('\n')` is a no-op on single-line (`:333-335`); `insert_str` strips `\n\r` on single-line (`:343-347`); column arithmetic uses `UnicodeWidthStr` (`:410`, `:420-426`).

**Weaknesses [I]**
- **`#[derive(Debug, Clone, …)]` on a struct holding a raw `String`** (`:10-11`) — this is the root of the secret-exposure class (§5).
- `cursor()`, `has_selection()`, `selected_text()` are `#[cfg(test)]`-gated (`:56`, `:140`, `:154`) — production callers reimplement them (`input.rs:354-359`).
- Word motion is `is_alphanumeric`-based only (`:198-229`) — `_`, `-`, `.` split words; inconsistent with `TextViewport::select_word_at` which treats `_ - / .` as word chars (`viewport.rs:448-451`).
- `line_count()` is `split('\n').count()` — O(n) and called every render (`code.rs:485`, `textarea.rs:235`).

### 1.7 `src/runtime.rs`

**Responsibility [F]** Terminal lifecycle guard + event loop + tick scheduling.

**Key public API [F]** `trait Application { handle, render, should_quit, tick_interval }` `:22-28` — **the only trait in the library**. `TerminalSession::enter` `:47`; `terminal()` `:69`; `leave()` `:74`; `Drop` `:82-86`; `run` `:99`; `event_loop` `:106`; `drain_pending_input` `:158`.

**Invariants [F]** Terminal state is restored on normal exit, error and panic (panic hook installed `:58-62`; `Drop` `:82`); DECAWM re-enabled explicitly (`:42`, `:91`); input floods are coalesced by draining the queue before redrawing (`:124-137`); a final frame is drawn when a tick causes a quit (`:145-148`).

**Weaknesses [I]**
- `Application::render(&mut self, frame)` takes `&mut self` (`:25`) — the runtime *sanctions* render-time mutation at the top of the stack, which is why it is pervasive below (§4).
- `tick_interval()` is polled every loop iteration (`:121`) but there is no way to request a one-shot redraw; anything animating must return `Changed` from `Tick` forever.
- The panic hook is installed but never removed on `leave()` (`:58-66`), so a second `enter()` stacks hooks.
- No hook for resize-driven relayout, capability detection, or colour-level changes; `ColorLevel::detect()` lives in `theme.rs:30` and is called by apps.

### 1.8 `src/theme.rs`

**Responsibility [F]** Concrete token struct + component style resolvers + colour-capability downgrade.

**Key public API [F]** `ColorLevel {TrueColor, Ansi256, Ansi16, Mono}` `:22-27`, `detect()` `:30`, `label()` `:45`. `mod palette` (private) `:56-95`. `struct Theme` — 30 **public** `Color` fields `:99-142`. `Theme::junie()` `:145`; `for_level` `:183`. Base styles `:229-291`. `backdrop` `:297`. Component resolvers: `row` `:329`, `lift` `:362`, `gutter` `:376`, `button` `:387`, `field_style` `:454`, `placeholder` `:466`, `selection` `:474`, `scrollbar_track` `:478`, `scrollbar_thumb` `:482`, `tone` `:492`, `syntax` `:506`, `badge` `:521`. Enums `Tone` `:534`, `SyntaxTone` `:547`, `ButtonKind` `:559`, `BadgeKind` `:568`. `downgrade`/`nearest_256`/`nearest_16` `:572/:587/:604`.

**Invariants [F]**
- No widget spells an RGB literal — verified: no `Color::Rgb` or hex constant appears in any file under `src/widgets/`. All literals are confined to `theme.rs:67-94`.
- Hover is "exactly one plane up, never a colour" (`row` `:344-346` + `lift` `:362-372`).
- Selection tint only when the row also has keyboard focus (`:340-342`).
- Disabled ignores hover/pressed (`:330-332`, `:388-396`; test `:682-697`).

**Weaknesses [I]**
- **`lift()` and `backdrop()` dispatch on colour *equality*** (`:362-372`, `:298-319`). A custom theme where two roles share a value, or where a caller passes a `bg` that is not one of the six known tokens, silently falls through to `popover` (`:370`) / `surface_overlay` (`:302`). This is the single biggest obstacle to goal §15 "a clearly different custom theme works across the complete catalog".
- **No trait, no recipes, no parts, no scoping, no patches.** `Theme` is one flat `Copy` struct; there is no way to override one component family, one variant, one subtree, one instance, or one logical part. Goal §15 scenarios 4–9 are entirely unimplemented.
- **Layout/glyph/density tokens do not exist.** `DESIGN.md` declares a `spacing:` block (gutter, gap, card-inset, frame-inset, tree-indent, field-height, tabs-height, min-width 72, min-height 20) and glyph semantics, but `Theme` carries **only colours** (`:99-142`). Every spacing and glyph value is therefore a literal inside a widget (§6.2).
- `ButtonKind`, `BadgeKind` (one variant), `SyntaxTone`, `Tone` are **closed enums** — no custom variants (goal §16).
- `Theme::change_glyph(RowState)` is defined **inside `grid.rs`** (`grid.rs:2007-2018`), extending the theme with a database-row concept from a widget module.
- `Theme` is `Copy` with 30 public fields: any consumer can mutate one field of a copy and diverge silently; there is no "derive the rest safely" path (goal §15 scenario 3).
- `for_level` re-lists all 30 fields in a macro (`:189-223`) — adding a token requires editing two places, and nothing enforces it.

### 1.9 `src/ui/ctx.rs`

**Responsibility [F]** The library↔app seam: a per-frame snapshot of interaction plus registration sinks.

**Key public API [F]** `Interaction { focus, hover, pressed, flash, focus_hidden, hover_suppressed, tick }` `:17-29`; `focused/hovered/pressed` `:32-40`. `VisualState { focused, hovered, pressed, selected, disabled, error, editing, busy }` `:45-54`. `RenderCtx { theme, interaction, hits, ring, cursor, inert }` `:56-66`; `new` `:69`; `control` `:86`; `clickable` `:97`; `scrollable` `:104`; `state` `:110`; `set_cursor` `:119`; `begin_modal` `:126`. Free fn `fill` `:135`.

**Invariants [F]**
- `pressed` requires the pointer still to be over the control, or a keyboard `flash` (`:38-40`) — mouse-down-then-leave does not read as pressed.
- `hover_suppressed` kills hover after a keyboard action (`:35-37`), matching DESIGN's "keyboard beats hover".
- `inert` blocks hit/ring/cursor registration for background content (`:86-123`).

**Weaknesses [I]**
- **No surface/background context.** `RenderCtx` carries no notion of "what plane am I drawing on", which is precisely why 24 render signatures take a raw `bg: Color` (§3.6).
- **No theme scope stack**, no style-override stack, no part resolution — the ctx cannot carry a local override.
- `begin_modal()` (`:126-131`) mutates the *snapshot* mid-frame (`inert = false`, `focus_hidden = false`) and pushes both barriers. Two calls in one frame silently clobber each other (see `dialog.rs:373` and `picker.rs:260`).
- `ctx.hits` and `ctx.ring` are `pub` `&mut` (`:59-60`), so widgets bypass `control/clickable/scrollable` at will (`chips.rs:231`, `choice.rs:204`, `tabs.rs:489`, `dialog.rs:478-487`, `picker.rs:279`, `ui/popup.rs:74-77`) — the `inert` guard is not enforced on those paths, which is a real modal-leak vector.
- `cursor: Option<Position>` is last-writer-wins with no owner recorded; two editing controls in one frame silently fight.
- `fill()` is a free function on the ctx module rather than a ctx method, so it cannot honour a surface stack.

### 1.10 `src/ui/layout.rs`

**Responsibility [F]** Two-pane split model with minima, maximise, drag and nudge.

**Key public API [F]** `SplitDir` `:6-11`; `Maximized` `:14-18`; `Split { percent, min_first, min_second, maximized }` (all public) `:23-28`; `new` `:31`; `toggle_max` `:40`; `grow` `:48`; `layout` `:53`; `handle` `:62`; `drag_to` `:75`; `nudge` `:95`; `vertical` `:110`; `horizontal` `:131`.

**Invariants [F]** Percent clamped to 5–95 (`:49`, `:88`); when `usable < min_first + min_second` the whole area goes to **first** in `vertical` (`:117-119`) but to **second** in `horizontal` (`:137`) — an intentional-looking asymmetry that is undocumented.

**Weaknesses [I]**
- This is the *only* layout primitive in the library. There is no measure/min-size/preferred-size protocol, no padding/inset type, no action-row abstraction — so `button::row_layout` / `row_layout_right` (`button.rs:167`, `:179`) live in the button module and are re-used by `dialog.rs:473` and `grid.rs:1915`. Goal §17 "action rows, right-aligned button groups, framed surfaces, form fields and split regions should not each invent unrelated layout behavior" is unmet.
- The vertical/horizontal fallback asymmetry (`:117` vs `:137`) is a latent behaviour difference no test covers.

### 1.11 `src/ui/popup.rs`

**Responsibility [F]** Anchored non-modal popup placement + a shared elevated surface that claims a hit barrier.

**Key public API [F]** `Placement { Below, Center }` `:16-21`; `place(screen, anchor, w, h, placement) -> Rect` `:25`; `surface(area, buf, ctx, theme) -> Rect` `:61`.

**Invariants [F]** `place` never returns a rect outside the screen (`:26-27`, `:44-53`); `Center` sits in the upper third (`:31`, test `:104-109`); `Below` flips above then clamps (`:40-48`).

**Weaknesses [I]**
- **`surface()` registers a single global id `WidgetId::of("popup.surface")`** (`ui/popup.rs:76`). Two popups in one frame (a `Select` inside a form plus a `Completion`) register the same id at different rects — `area_of` returns the last, `hit` returns whichever is topmost, and no owner can tell them apart.
- `surface()` pushes a hit barrier (`:74`) but **not** a focus barrier, so keyboard focus still reaches the page behind a `Select`'s open list.
- **Two different `Placement` enums with the same name**: `ui::popup::Placement {Below, Center}` (`:16-21`) vs `widgets::menu::Placement {Below, Above, Right}` (`menu.rs:56-63`), with independent placement algorithms (`popup.rs:25-56` vs `menu.rs:143-171`). Dialogs use neither (`dialog.rs:376` uses `Rect::centered`).
- No stacking/z-order model, no nesting, no click-outside contract, no lifecycle events — goal §14's required definitions are absent.

### 1.12 `src/ui/text.rs`

**Responsibility [F]** Display-width text utilities and the fuzzy matcher.

**Key public API [F]** `width` `:6`; `truncate` `:11`; `truncate_middle` `:33`; `thousands` `:67`; `fit` `:80`; `fit_right` `:87`; `wrap` `:94`; `fuzzy(label, word) -> Option<(u32, Vec<usize>)>` `:145`.

**Invariants [F]** `truncate` never exceeds `max` cells and appends `…` (`:11-30`); `wrap` hard-wraps overlong words (`:107`, `:129-139`); `fuzzy` ranks prefix (0) < boundary substring (10) < substring (30) < subsequence (60+) (`:151-171`).

**Weaknesses [I]**
- Every helper **allocates a `String` per call**, and they are called per row per frame (`list.rs:284`, `tree.rs:556`, `grid.rs:1852-1856`, `table.rs:765-768`). Goal §25.6 "per-frame allocations / cloning of rows or strings" is a real finding here.
- `fuzzy` is byte-based over `to_lowercase()` output (`:160-170`) and returns byte offsets into the *lowercased* string, which are then used as indices into the *original* label by `completion.rs:211` and `picker.rs:458`. Any label whose lowercase form differs in byte length (e.g. `İ`, `SS`→`ss` is same length but Turkish dotted-I is not) mis-highlights or highlights the wrong bytes. **Latent correctness bug.**
- `truncate` at `max == 0` returns `""` but at `max == 1` returns just `…` — no way to opt out of the ellipsis.
- `thousands` hard-codes `,` (`:72`) — no locale hook.

---

## 2. Per-widget inventory

### 2.1 Master matrix (machine-checkable columns)

Legend — **Ret**: return shape of the primary handlers. **Reg**: focus/hit registration mechanism. **bg**: render takes a raw `bg: Color`. **Rect**: exposes public frame-local geometry. **RT-mut**: render performs a *semantic* mutation (§4). **Scroll**: owns a `ScrollState`. **SB-h**: has an `on_scrollbar` handler. **owns/loc**: has `owns()` / `locate()`.

| # | Module | Ret | Reg | bg | Rect | RT‑mut | Scroll | SB‑h | owns/loc | Disposition |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `brand` | — (draw only) | `ctx.clickable` (`brand.rs:84`) | no (uses `t.accent`) | no | no | no | n/a | no | **Compose** into a `Badge`/`Chip` primitive |
| 2 | `button` | `(Outcome,bool)` / `bool` | `ctx.control` `:162` | **yes** `:106` | `pub area` `:23` | no | no | n/a | no | **Refactor** (reference control) |
| 3 | `chips` | `(Outcome,Option<ChipEvent>)` | raw `ctx.ring.register` `:231` | **yes** `:148` | `pub area` `:42` | no | no | n/a | `owns` `:141` | **Decompose** → Chip primitive + roving-focus group |
| 4 | `choice` | `Outcome` | `ctx.control` `:84`,`:293`; raw ring `:204` | **yes** `:52,153,254` | `pub area/areas` `:18,96,216` | no | no | n/a | no | **Decompose** → Checkbox/Radio/Switch on one field wrapper |
| 5 | `code` | `(Outcome,Option<EditorEvent>)` | `ctx.control`+`scrollable` `:616-617` | **yes** `:601` | `pub area` `:81` | **yes** `:611-614` | yes `:74` | `:566` | no | **Decompose** → editor core + gutter + find bar |
| 6 | `completion` | `(Outcome,Option<_>)` / `Option<_>` | `ctx.clickable` rows only `:229` | no (internal) | `pub area`,`pub anchor` `:37-39` | no | yes `:35` | **no** | both `:86,:90` | **Compose** onto the overlay/list primitives |
| 7 | `dialog` | `Outcome` (+ polled `result`) | raw `ctx.hits.register` `:478-487`; `begin_modal` `:373` | no (internal) | `pub area` `:52` | **yes** `:465-470` | no | n/a | no | **Rebuild** — closed body enum, focus params |
| 8 | `diff` | delegates to viewport | via `TextViewport` | **yes** `:284` | via viewport | **yes** `:285` (`layout` in render) | via viewport | `:280` | `owns` `:260` | **Move** to a `diff` presentation layer over a generic viewport |
| 9 | `empty` | — (free fn) | none | **yes** `:46` | no | no | no | n/a | no | **Retain**, restyle as a part |
| 10 | `field_common` | `EditAction` | none | n/a | no | no | no | n/a | no | **Refactor** → keymap/command table |
| 11 | `grid` | `(Outcome,Option<GridEvent>)` | `ctx.control`+`scrollable` `:1521-1522` | **yes** `:1509` | `pub area` `:355` | **yes** `:1510`,`:1518-1520` | yes ×2 `:344-345` | `:1364` | both `:1208,:1220,:1232,:1235` | **Split** — generic grid + TablePro adapter (goal §18) |
| 12 | `hintbar` | — (draw only) | none | no | no | no | no | n/a | no | **Retain**, wire to component hint metadata |
| 13 | `input` | `(Outcome,Option<InputEvent>)` | `ctx.control` `:424` | **yes** `:268` | `pub area` `:33` | **yes** `:282-286` | h-scroll (`usize` field `:32`) | n/a | no | **Decompose** → value control + field wrapper; secret handling |
| 14 | `keyhint` | — (free fns) | none | no | no | no | no | n/a | no | **Retain** |
| 15 | `list` | `Outcome` | `ctx.control`+`scrollable` `:218-219` | **yes** `:207` | `pub area` `:54` | no | yes `:53` | `:195` | both `:186,:191` | **Refactor** → generic collection with custom item renderer |
| 16 | `menu` | `(Outcome,Option<MenuEvent>)` / `Option<_>` | `ctx.clickable` `:260,:339`; `ctx.control` (bar) `:540` | **yes** (bar) `:533`; no (popover) | `pub area/anchor/areas/brand_area` `:77-79,:373-374` | **yes** `:243-248` | no | n/a | both `:118,:122,:414` | **Compose** onto overlay + list; own `Placement` must merge |
| 17 | `panel` | `Outcome` (ScrollPanel) | `ctx.control`+`scrollable` `:287-288` | **yes** `:80,:257` | `pub area` `:183` | viewport-only `:289-291` | yes `:180` | `:243` | no | **Split** — `Panel` (surface) retain; `ScrollPanel` **remove**, merge into `viewport` |
| 18 | `picker` | `(Outcome,Option<PickerEvent>)` / `Option<_>` | raw `ctx.hits.register` `:279`; `begin_modal` `:260` | no (internal) | `pub area` `:58` | `cursor_dirty` `:351-354` | yes `:52` | **no** | both `:120,:123` | **Compose** = overlay + query field + list |
| 19 | `progress` | — (draw only) | none | **yes** `:23,220,326,342,377` | no | no | no | n/a | no | **Decompose** → Spinner / Bar / Meter primitives |
| 20 | `props` | `(Outcome,Option<PropsEvent>)` + free `render` | `ctx.control`+`scrollable` `:203-204` | **yes** `:51,:193` | `pub area` `:105` | no | yes `:104` | `:181` | both `:129,:132` | **Refactor** → a `DescriptionList` variant of the list |
| 21 | `scrollbar` | — (free fns) | `ctx.clickable` `:43` | no | no | no | n/a | n/a | `id_for` `:12` | **Retain** as a part of a scroll container |
| 22 | `segments` | — (free fn) | `ctx.clickable` `:117` | **yes** `:50` | no | no | no | n/a | no | **Merge** with `statusbar` (duplicate priority-drop logic) |
| 23 | `select` | `(Outcome,Option<SelectEvent>)` | `ctx.control` `:199` + popup `:213` | **yes** `:153` | `pub area` `:24` | **yes** `:161-167` | **no** (10-row clip `:210`) | n/a | both `:65,:68` | **Rebuild** on overlay + list |
| 24 | `splitter` | `Outcome` | `ctx.clickable` `:52` | **yes** `:33` | `pub area` `:19` | no | no | n/a | no | **Retain**, pair with `Split` as one component |
| 25 | `statusbar` | — (draw only) | `ctx.clickable` `:302` | no (internal `surface_elevated` `:270`) | no (returns `Placed`) | no | no | n/a | no | **Retain**; unify priority-drop with `segments` |
| 26 | `steps` | `Outcome` | `ctx.control` (conditional) `:197`; `scrollable` `:199` | **yes** `:186` | `pub area` `:73` | no | yes `:72` | `:174` | both `:132,:136` | **Refactor** → list variant with a state column |
| 27 | `table` | `(Outcome,Option<TableEvent>)` | `ctx.control`+`scrollable` `:570-571` | **yes** `:557` | `pub area` `:114` | **yes** `:566-568` | yes ×2 `:109-110` | `:516` | both `:797,:811,:818` | **Merge** with `grid` — one generic table |
| 28 | `tabs` | `(Outcome,Option<TabEvent>)` | raw `ctx.ring.register` `:489` | **yes** `:258` | `pub areas` `:60` | strip scroll `:291-313` | no (own `first`/`fit`) | n/a | both `:122,:126` | **Refactor** — stable keys (Scenario E) |
| 29 | `textarea` | `(Outcome,Option<InputEvent>)` | `ctx.control`+`scrollable` `:222-223` | **yes** `:189` | `pub area` `:30` | **yes** `:202-205` | yes `:29` | **no** | no | **Refactor** onto the same editor core as `code` |
| 30 | `tree` | `(Outcome,Option<TreeEvent>)` | `ctx.control`+`scrollable` `:469-470` | **yes** `:460` | `pub area` `:114` | no | yes `:113` | `:448` | both `:432,:444` | **Refactor** — path-based identity, custom node renderer |
| 31 | `viewport` | `(Outcome,Option<ViewportEvent>)` | `ctx.control`+`scrollable` `:601-602` | **yes** `:580` | `pub area` `:119` | viewport-only `:598-600` | yes `:113` | `:519` | `owns` `:532` | **Retain** as *the* scrollable text primitive |

### 2.2 Per-widget detail

Fields below follow goal §7's list. "App copies/variants" is marked `[not audited]` for all 31 — the binaries were out of scope for this pass.

---

**1. `brand` (`Lockup`)**
- Responsibility: the single accent-filled identity pill (`brand.rs:1-4`).
- Constructors/methods: `new` `:24`, `compact` `:31`, `label` (private) `:38`, `width` `:46`, `style(&Theme)` (assoc.) `:50`, `render(x,y,buf,theme)` `:58`, `render_clickable(x,y,buf,ctx,id)` `:65`.
- Public mutable fields: `text: String`, `compact: bool` `:15-21`.
- Props/config: text + compact flag only.
- Owned app data: the mark text.
- Owned interaction state: none (id is passed per call).
- Frame-local geometry: none.
- Render signature: **two** — `render(x: u16, y: u16, …) -> u16` `:58` and `render_clickable(…, id) -> u16` `:65`. Both take `x,y` not a `Rect`, and both return a width. Unique in the library.
- Render-time mutation: none.
- Keyboard: none. Mouse: hover/pressed styling only `:76-81`.
- Paste/wheel/drag/tick: none.
- Event type: none — the owner must interpret the click.
- Focus: none (clickable-only, never enters the ring).
- Child identity: none (id supplied by caller).
- Hit registration: `ctx.clickable(id, Rect::new(x,y,w,1))` `:84`.
- owns/locate: none.
- Scroll/overlay: none.
- Theme deps: `text_on_accent`, `accent`, `accent_hover`, `accent_pressed`, `BOLD` `:50-55`, `:77-81`.
- Raw bg params: none.
- Hard-coded: one space of padding on each side `:41`.
- Domain assumptions: "only control that fills with the accent" (`:1-4`) — a Junie budget rule baked into a component.
- Tests: `brand.rs:96-131` (padding/width/style; clickable hover + hit).
- **Disposition — compose.** Fold into a themed `Badge`/`Chip` primitive with a normal `render(area, buf, ctx)` signature; keep "product mark" as a variant, not a hard rule.

---

**2. `button` (`Button`)**
- Responsibility: single-line ` label ` control with a focus gutter, four kinds + toggle + busy.
- Constructors/methods: `new(id,&str,ButtonKind)` `:27`; `primary/secondary/subtle/danger` `:39-50`; `toggle(id,label,on)` `:51`; `disabled(bool) -> Self` `:56`; `width()` `:62`; `can_activate()` `:67`; `on_key -> (Outcome,bool)` `:72`; `on_click -> bool` `:86`; `render(area,buf,ctx,bg)` `:101`. Free fns `row_layout` `:167`, `row_layout_right` `:179`.
- Public mutable fields: `id, label, kind, disabled, on: Option<bool>, busy, area` `:15-24`.
- Owned interaction state: `on` (uncontrolled toggle) — mutated inside `on_key`/`on_click` via `toggle_if_needed` `:95-99`.
- Frame-local geometry: `pub area` `:23`, written in render `:112`.
- Render-time mutation: none semantic.
- Keyboard: Enter, Space `:73` (hard-coded).
- Mouse: `on_click()` with no arguments `:86`.
- Event type: **`bool`** ("activated") — unique.
- Focus: `ctx.control(self.id, area, self.disabled)` `:162`.
- Child identity / hit-test: none beyond itself.
- Theme deps: `Theme::button` `:127`, `Theme::gutter` `:132`, `accent`/`text_muted` for the toggle marker `:149`, `text_secondary` for busy `:129`.
- Raw bg: `bg: ratatui::style::Color` `:106`.
- Hard-coded glyphs/dims: `"▎"` gutter `:143`; `●`/`○` toggle `:138-139`; 1-cell padding each side `:64`; height forced to 1 `:110`; spinner borrowed from `progress::spinner_frame` `:135`,`:157`.
- Domain assumptions: none.
- Tests: **none in-module.** Covered indirectly by `showcase/app_tests.rs:220-242`.
- **Disposition — refactor.** This is the least-damaged control and should become the reference shape for the new API: `Outcome`+action, no public `area`, no raw `bg`, variants via recipe.

---

**3. `chips` (`Chip`, `ChipBar`, `ChipEvent`)**
- Responsibility: horizontal row of removable/toggleable chips with an internal cursor, an optional lead label and an add button.
- Constructors/methods: `Chip::new` `:24`; `ChipBar::new(id)` `:56`; `chip_id/close_id/add_id/lead_id` `:67-78`; `stops` (private) `:81`; `on_key -> (Outcome,Option<ChipEvent>)` `:85`; `on_click(id) -> (Outcome,Option<ChipEvent>)` `:121`; `owns(id)` `:141`; `render(area,buf,ctx,bg)` `:148`.
- Public mutable fields: `Chip{label,enabled,removable,error}` `:16-21`; `ChipBar{id,chips,cursor,add_label,lead,area}` `:35-43`.
- Owned app data: `Vec<Chip>` (owned `String` labels).
- Owned interaction state: `cursor` `:38`.
- Frame-local geometry: `pub area` `:42`.
- Render-time mutation: `self.cursor` is clamped in `on_key` (`:90`), not render — render is clean semantically.
- Keyboard: ←/→/h/l, Enter, Space, Delete/Backspace/`x`, `+`, `X` `:91-118` — all hard-coded, including `X` = clear-all.
- Mouse: `on_click(WidgetId)` `:121`.
- Event: `ChipEvent{Activate,Toggle,Remove,Add,Lead,ClearAll}` `:46-53`.
- Focus: **raw** `ctx.ring.register(self.id)` at `:231` — bypasses `ctx.control`, so the bar never registers a hit region for itself.
- **Defect [F/I]:** when the chips overflow the row, render prints `…` and **`return`s at `:194`**, skipping both the add button (`:216-229`) and the ring registration (`:230-232`). An overflowing chip bar silently drops out of the Tab order.
- Hit registration: `ctx.clickable` for lead `:168`, chip `:209`/`:212`, close `:210`, add `:227`.
- owns/locate: `owns` `:141`; **no `locate`** — the owner must compare ids by hand or call `on_click(id)`.
- Scroll: none — overflow is truncation only.
- Theme deps: `Theme::button(ButtonKind::Toggle|Secondary|Subtle)` `:183`,`:220`; `gutter` `:198`,`:225`; `lift` `:163`; `text_faint`/`error`/`text_muted`/`text_primary` `:185-207`.
- Hard-coded: `"▎"` `:198`,`:225`; `"×"` `:208`; `"…"` `:193`; default add label `"+ Add filter"` `:61`; 1-cell gaps `:214`.
- Domain assumptions: the default `add_label` and the `lead` doc-comment example (`"match all ▾"`, `:40`) are TablePro filter-bar concepts.
- Tests: **none.**
- **Disposition — decompose.** A `Chip` presentation primitive plus a generic roving-focus horizontal group; the filter semantics move to TablePro.

---

**4. `choice` (`Checkbox`, `RadioGroup`, `Toggle`)**
- Responsibility: three unrelated form controls in one module (`choice.rs:1`).
- Constructors: `Checkbox::new(id,label,checked)` `:22`; `RadioGroup::new(id,label,&[&str],selected)` `:100`; `Toggle::new(id,label,on)` `:220`, `Toggle::disabled(bool)->Self` `:229`. `Checkbox` has **no** builder for `disabled` (field mutation only).
- Methods: `Checkbox::on_key/on_click -> Outcome` `:32/:44`; `RadioGroup::height()` `:112`, `on_key -> Outcome` `:116`, `on_click(index) -> Outcome` `:139`, `option_id(i)` `:149`; `Toggle::on_key/on_click -> Outcome` `:234/:246`.
- Public mutable fields: `Checkbox{id,label,checked,disabled,area}` `:13-19`; `RadioGroup{id,label,options,selected,cursor,disabled,areas}` `:89-97`; `Toggle{id,label,on,disabled,area}` `:211-217`.
- Owned app data: `options: Vec<String>` (owned copies of `&[&str]`, `:104`).
- Owned interaction state: `checked` / `selected`+`cursor` / `on` — all uncontrolled, mutated in handlers.
- **API inconsistency [F]:** `RadioGroup` moves *selection* with the arrow keys (`:121-130` set `self.selected = self.cursor`), so navigation is activation. `ListBox` does the opposite (`list.rs:124-131` move cursor only). Same physical gesture, different semantics.
- Frame-local geometry: `pub area` `:18`,`:216`; `pub areas: Vec<Rect>` `:96` (cleared and rebuilt in render `:166-173`).
- Render-time mutation: none semantic.
- Keyboard: Space/Enter (`:36`,`:238`), ↑↓/j/k + Space/Enter (`:121-134`).
- Mouse: `Checkbox::on_click()` (no args), `RadioGroup::on_click(index: usize)`, `Toggle::on_click()` — three shapes in one module.
- Event: none — `Outcome` only; the owner must re-read `checked`/`selected`/`on`.
- Focus: `ctx.control` for Checkbox `:84` and Toggle `:293`; RadioGroup uses **raw** `ctx.ring.register` `:204` and registers each option as `ctx.clickable` `:194` — the group itself has no hit region.
- Dead code [F]: `let whole = …;` `:196-201` followed by `let _ = whole;` `:206`.
- Theme deps: `row` `:66`,`:180`,`:267`; `gutter` `:68`,`:182`,`:269`; `accent`/`text_muted`/`disabled` for markers `:71-76`,`:185-191`,`:270-291`; `faint`/`label` `:160-164`,`:208`.
- Raw bg: all three `render(…, bg: Color)` `:52`,`:153`,`:254`.
- Hard-coded: `"▎"`; `[✓]`/`[ ]` `:69`; `(●)`/`( )` `:184`; `──●`/`○──` `:271-275`; label offsets `+5` `:78`,`:193`,`:278`; `"on"/"off"` suffix `:279`.
- Tests: **none.**
- **Disposition — decompose.** Split into three components sharing one field wrapper (label / required / help / error / disabled), emit semantic actions, and unify cursor-vs-selection with `list`.

---

**5. `code` (`CodeEditor`)**
- Responsibility: document editor with gutter (focus bar, block marker, line numbers, diagnostics), caller-supplied highlighting/segmentation, h-scroll, selection, inline find (`code.rs:1-7`).
- Constructors: `new(id, &str)` `:100`; builders `highlighter` `:127`, `segmenter` `:131`, `read_only` `:135`, `placeholder` `:139`.
- Methods: `text` `:144`, `set_text` `:148`, `is_empty` `:156`, `current_block` `:161`, `selection_or_block` `:173`, `blocks` `:182`, `jump_to` `:188`, `cursor_offset` `:195`, `cursor_cell` `:200`, `begin_edit` `:212`, `set_running` `:219`, `commit` `:223`, `open_find` `:230`, `on_key -> (Outcome,Option<EditorEvent>)` `:289`, `on_click(pos, was_focused) -> Outcome` `:530`, `on_drag(pos)` `:548`, `on_wheel(delta, horizontal)` `:557`, `on_scrollbar(pos)` `:566`, `on_paste(&str)` `:578`, `render(area,buf,ctx,bg)` `:601`.
- Public mutable fields: `id, buffer, editing, read_only, scroll, hscroll, indent, highlighter, segmenter, diagnostics, running, find, placeholder, tab_leaves, area` `:63-81`. Private: `text_area, gutter_w, drag_anchor, cached_spans, cached_for` `:82-87`.
- Config/props: `Highlighter = fn(&str)->Vec<(Range,SyntaxTone)>` `:26`, `Segmenter = fn(&str)->Vec<Range>` `:27` — **bare function pointers**.
- Owned app data: the document (`TextBuffer`), diagnostics, find state.
- Owned interaction state: `editing`, `scroll`, `hscroll`, `drag_anchor`, `find`.
- Frame-local caches: `cached_spans`/`cached_for` (hash-keyed) `:86-87`, `:589-599`; `text_area`, `gutter_w`.
- **Render-time semantic mutation [F]:** `code.rs:611-614` — `if !s.focused && self.editing { self.commit(); }`. Drawing an unfocused editor exits editing mode.
- Keyboard: nav mode `:398-481` (Enter/`i` edit, `a` append, ↑↓/k/j, ←→/h/l h-scroll, PgUp/PgDn, Home/`g`, End/`G`, `{`/`}` block jump, `/` find, `n`/`N` next/prev, Esc closes find). Edit mode delegates to `edit_key(key,true)` `:329` plus `PageUp/PageDown` `:378-392`. Find bar consumes everything `:291-325`. All hard-coded.
- Mouse: click places cursor / begins editing `:530-546`; drag selects `:548-555`; wheel v/h `:557-564`; scrollbar `:566-576`.
- Paste: `:578-585` (rejected unless editing and writable).
- Tick: spinner for `running` `:694`.
- Event: `EditorEvent{Changed, CursorMoved, Committed, Leave{backward}}` `:51-60`.
- Focus: `ctx.control(self.id, area, false)` `:616` — **`disabled` is hard-coded `false` even when `read_only`**, so a read-only editor still takes a Tab stop while `s.disabled` is set from `read_only` at `:610`.
- Child identity: none (single control).
- Hit registration: `control` + `scrollable` `:616-617`; scrollbar via `scrollbar::render_vertical` `:836`.
- owns/locate: **absent** — inconsistent with every other scroll container.
- Overlay: none (the completion popup is a separate widget the owner must anchor via `cursor_cell()` `:200`).
- Theme deps: `field_style` `:622`, `gutter` `:675`, `syntax` `:767`, `popover` (selection + current find match) `:771`,`:775`, `border_strong` (underlines) `:780`,`:796`,`:801`, `accent` (marker, find underline) `:695`,`:701`,`:849`, `error`/`warning` (diagnostics) `:709-713`,`:786-790`, text ladder throughout.
- Raw bg: `bg: Color` `:601`.
- Hard-coded: `"▎"` `:681`; `"›"` block marker `:700`; `"!"` diagnostic `:714`; `"…"` `:742`,`:760`; gutter layout `1+1+num_w+1+1` `:635`; h-scroll step `8` `:420`,`:424`; wheel h step `4` `:559`; scroll margin `4` `:488-491`; `indent` default `2` `:112`; find label `"find "` `:842`; position label format `:873`.
- Domain assumptions: `running: Option<Range>` and the block/statement model are SQL-runner concepts leaking into a generic editor (`:74-77`, `:161-186`).
- Tests: **none in-module.**
- **Disposition — decompose.** Editor core (shared with `textarea`), gutter as a part, find bar as a composed strip, `running`/blocks pushed to the owner via a generic "line decoration" hook, and closures instead of `fn` pointers.

---

**6. `completion` (`Completion`, `CompletionItem`, `CompletionEvent`)**
- Responsibility: anchored non-modal suggestion list where the owner keeps focus (`completion.rs:1-3`).
- Constructors/methods: `new(id)` `:49`; `open(items, anchor, replace_len)` `:62`; `is_open` `:70`; `close` `:74`; `current` `:78`; `row_id` `:82`; `locate` `:86`; `owns` `:90`; `on_key -> (Outcome,Option<CompletionEvent>)` `:94`; `on_click(id) -> Option<CompletionEvent>` `:136`; `on_wheel(delta) -> Outcome` `:142`; `render(screen,buf,ctx)` `:147`.
- Public mutable fields: `id, items, cursor, scroll, anchor, replace_len, max_rows, area` `:31-40`.
- Owned app data: `Vec<CompletionItem>` with owned `label`/`detail`/`insert` and `glyph: &'static str` `:19-27`.
- Owned interaction state: `cursor`, `scroll`.
- Frame-local: `pub area` `:39`; `pub anchor` `:37` is *caller-supplied* geometry.
- Render-time mutation: `scroll.set_content/set_viewport/ensure_visible` `:170-172` (viewport metadata only — acceptable per goal §11).
- Keyboard: ↓/Ctrl+n, ↑/Ctrl+p, PgUp/PgDn, Tab/Enter accept, Esc dismiss `:98-133`. The `Char('n')`/`Char('p')` guard is convoluted (`:99-102`, `:107-113`) and returns `Ignored` for plain `n`/`p`.
- Mouse: `on_click(WidgetId) -> Option<CompletionEvent>` — **no `Outcome`**, unique shape.
- Wheel: `on_wheel` always returns `Changed` `:142-145`, unlike `Picker::on_wheel` which returns `Consumed` at the boundary (`picker.rs:234-242`).
- Event: `CompletionEvent{Accept(usize), Dismiss}` `:43-46`.
- Focus: **none registered** — deliberate (owner keeps focus), but that means Esc/arrow routing is entirely the owner's job.
- Hit registration: rows via `ctx.clickable` `:229`; the popup surface via `ui::popup::surface` `:169` which pushes the hit barrier and registers the shared `"popup.surface"` id.
- **Renders a scrollbar (`:231-240`) but exposes no `on_scrollbar` handler** — the thumb is drawn and hit-registered (`scrollbar.rs:43`) yet unclickable.
- Theme deps: `surface_elevated` `:174`, `row` `:181`, `gutter` `:184`, `text_primary`/`text_muted` `:189-193`,`:226`.
- Raw bg: none (hard-codes `t.surface_elevated` `:174`).
- Hard-coded: `"▎"` `:184`; width clamp `24..=48` `:164`; `max_rows` default `8` `:57`; label column offset `+3` `:197`.
- Domain assumptions: `glyph: &'static str` documented as "T, V, C, K, F, S, A" `:22` — TablePro completion-kind letters in a generic type.
- Tests: **none.**
- **Disposition — compose.** Rebuild as `Overlay(anchored) + List(custom row renderer)`; `matched` highlighting becomes a row-render concern.

---

**7. `dialog` (`Dialog`, `DialogBody`, `AckInput`, `DialogResult`)**
- Responsibility: modal dialog owning its own focus scope (`dialog.rs:1-2`).
- Constructors: `confirm` `:58`, `destructive` `:75`, `prompt` `:92`, `facts` `:110`, plus `with_actions` `:146`.
- Methods: `armed()` `:139`, `is_editing()` `:152`, `input_mut()` (private) `:160`, `height(width)` `:168`, `finish` (private) `:191`, `on_key(key, focus: &mut Focus, ring: &FocusRing) -> Outcome` `:207`, `on_paste` `:316`, `on_click(id, pos, focus: &mut Focus) -> Outcome` `:324`, `on_click_outside() -> Outcome` `:350`, `render(screen,buf,ctx)` `:357`.
- Public mutable fields: `id, title, body, actions, cancel_index, width, area, result, initial_focus` `:44-55`.
- **Closed body enum [F]:** `DialogBody { Text(String), Input(TextInput), Facts{facts, code, ack} }` `:18-28`. Goal §14 explicitly forbids preserving this shape.
- Owned app data: title text, body text/facts/code, the embedded `TextInput` (which may hold a secret).
- Owned interaction state: `result: Option<DialogResult>` — a **polled** result field `:53`, unique in the library; every other widget returns its event.
- Frame-local: `pub area` `:52`.
- **Render-time semantic mutation [F]:** `dialog.rs:465-470` — render evaluates the acknowledgement token and writes `last.disabled = !armed` on the confirming button. Validation of a typed acknowledgement happens *because the dialog was drawn*.
- Keyboard: Esc `:263`, Tab/BackTab `:270-277`, ←/h and →/l between actions `:278-296`, `y`/`n` **only for `DialogBody::Text`** `:297-311`, everything else `Consumed` `:312`. All hard-coded.
- Mouse: `on_click(id, pos, focus)` `:324`; outside-click cancel `:350`.
- Paste: `:316-321`.
- Event: `DialogResult{Action(usize), Cancelled}` `:37-41` — delivered through the polled `result` field, not the return value.
- Focus: **the dialog mutates `Focus` directly**, taking `&mut Focus` and `&FocusRing` as handler parameters (`:207-212`, `:324-329`). This is the only widget that does, and it inverts the `RenderCtx` seam.
- Child identity: `id.sub("cancel")`, `id.sub("ok")`, `id.sub("ack")` `:59-60`,`:76-78`,`:93-94`,`:118-122`.
- Hit registration: **raw** `ctx.hits.register` for the surface `:478` and for each action/input `:480-487`, re-registering on top of what `Button::render`/`TextInput::render` already registered. `ctx.begin_modal()` `:373`.
- Overlay: dims the backdrop cell-by-cell via `Theme::backdrop` `:366-372`, deliberately leaving the last row live for the shared hint bar `:359-365`.
- Theme deps: `backdrop` `:368`, `surface_elevated` `:378`, `border(true)` `:384`, `title` `:395`, `secondary` `:407`,`:448`.
- Raw bg: none (hard-codes `t.surface_elevated`), but it *passes* `bg` down to `Button::render` `:475`, `TextInput::render` `:417`,`:460` and `props::render` `:426`.
- Hard-coded: widths `54` `:69`,`:86`,`:104` and `66` `:131`; inner margin `(3,2)` `:387`; height formula `2+1+1+1+body+1+1+1` `:188`; code preview cap `6` lines `:178`,`:431`; `y`/`n` keys.
- Domain assumptions: `DialogBody::Facts { code: Vec<String> }` is a SQL-preview slot; `AckInput` "type the table name" is TablePlus Safe Mode (`README.md:155-163`).
- Tests: **none in-module**; covered by `showcase/app_tests.rs:453-496`.
- **Disposition — rebuild.** Arbitrary composed content, convenience constructors on top of the same primitives, action results as a return value, focus handled by the overlay/focus-scope system rather than `&mut Focus` parameters.

---

**8. `diff` (`DiffView` + data model)**
- Responsibility: unified-diff model plus a viewer that renders one file in `Unified` or `Review` layout over a `TextViewport` (`diff.rs:1-6`).
- Data model: `DiffLineKind` `:19-23`, `DiffLine` (+`context/add/remove`) `:26-50`, `DiffHunk` (+`old_len/new_len/header`) `:53-83`, `DiffStatus` (+`marker/label/tone`) `:86-121`, `DiffFile` (+`additions/deletions/summary/header`) `:124-165`, `DiffMode` (+`label/toggled`) `:168-189`.
- Viewer: `DiffView { pub term: TextViewport, pub mode: DiffMode, file, laid_width, dirty }` `:192-198`; `new(id)` `:201`; `id()` `:211`; `file()` `:215`; `set_file` `:219`; `set_mode` `:228`; `toggle_mode` `:235`; `layout(width)` `:241`; `owns` `:260`; `on_key/on_wheel/on_click/on_drag/on_scrollbar` `:264-282` (all pure delegation); `render(area,buf,ctx,bg)` `:284`.
- Free fns: `unified_lines(&DiffFile) -> Vec<Line>` `:310`, `review_lines(&DiffFile, width) -> Vec<Line>` `:403`, plus private `gutter/num_width/changed_range/emphasised`.
- Public mutable fields: `term`, `mode` `:193-194`.
- **Render-time mutation [F]:** `render` calls `self.layout(area.width)` `:285`, which rebuilds every line, calls `self.term.set_lines(...)` `:253`, `set_follow(false)` `:254` and `scroll_to(offset)` `:255`. Content generation happens during draw. The module's own test asserts the offset survives (`:584-585`), which is the mitigation, not the fix.
- Event: `ViewportEvent` (borrowed from `viewport`) `:264`.
- Focus/hit: entirely via `TextViewport` `:601-602`.
- Theme deps: only through `Tone` (`Success/Error/Warning/Secondary/Faint/Muted`) `:113-120`, `:325-347`, `:416-423` — the cleanest theming in the library.
- Raw bg: `bg: Color` `:284`.
- Hard-coded: `@@ -a,b +c,d @@` header `:75-82`; `A/M/D/R` markers `:95-101`; ` │ ` separator `:416`; hunk rule `─`×`min(total,200)` `:429`; `+`/`−` summary `:148-153`; `"(no textual changes)"` `:353`,`:491`; min gutter width `3` `:305`.
- Domain assumptions: git-diff semantics throughout — legitimate for a diff component, but it is a *product* component, not a primitive.
- Tests: `diff.rs:530-592` — counts/headers, unified markers, review pairing + emphasis, render + wheel + mode toggle.
- **Disposition — move/compose.** Keep the data model; make the viewer an application-level composition over the generic viewport with a caller-supplied line builder, so `layout()` is an explicit lifecycle call, never a render side effect.

---

**9. `empty` (`EmptyState`, `EmptyKind`)**
- Responsibility: centred quiet empty/error state (`empty.rs:1`).
- Constructors: `new(&str)` `:25`, `error(&str)` `:32`, builder `hint(&str)` `:39`.
- Render: **free function** `render(area, buf, t: &Theme, e: &EmptyState, bg: Color) -> ()` `:46` — takes `&Theme`, **not** `&mut RenderCtx`. Unique signature shape shared only with `props::render` `:51` and `keyhint::*`.
- Public mutable fields: `title, hint, kind` `:18-22`.
- Interaction: none. Focus/hit: none. Events: none.
- Theme deps: `muted` `:71`, `error_fg` `:74`,`:83`, `faint` `:91`.
- Raw bg: `bg: Color` `:46`.
- Hard-coded: `"! "` prefix `:73`; centring math `:61`,`:67`; hint offset `+2` `:91`; wrap width `width-4` min 8 `:54`.
- Tests: **none.**
- **Disposition — retain**, but re-express as a themed part with a `RenderCtx` signature so it can inherit the surface.

---

**10. `field_common` (`EditAction`, `edit_key`)**
- Responsibility: the shared text-editing keymap (`field_common.rs:1`).
- API: `enum EditAction { Commit, Cancel, Tab{backward}, Apply(fn(&mut TextBuffer)), Insert(char), None }` `:8-15`; `edit_key(key: &Key, multiline: bool) -> EditAction` `:20`.
- **Extension mechanism [F]:** `Apply(fn(&mut TextBuffer))` — a bare function pointer, so every binding is a non-capturing closure literal (`:24`, `:29-31`, …). No caller can add or remap a binding.
- Bindings, all hard-coded `:22-110`: Esc, Enter (insert `\n` when multiline), Tab/BackTab, Ctrl/Alt+←→ word motion, ←→, ↑↓ (multiline), Ctrl+Home/End, Home/End, Ctrl/Alt+Backspace, Backspace, Delete, Ctrl+A/E/U/K/W/L, Alt+B/F, any other Ctrl/Alt char → `None`, printable → `Insert`.
- Semantics note [F]: `multiline` flips Esc from cancel to commit *at the call site*, not here — `edit_key` always returns `Cancel` for Esc `:23`; `code.rs:330-333` and `textarea.rs:121-124` treat `Cancel` as commit, while `input.rs:204-207` treats it as revert. The same `EditAction` means two opposite things.
- Tests: **none.**
- **Disposition — refactor.** Replace with a data-driven keymap over semantic edit *commands*, configurable per component and per application (goal §13).

---

**11. `grid` (`DataGrid` + the whole database-cell model)** — *the largest and most domain-entangled module (2192 lines).*
- Responsibility (as written): dense tabular data, cursor cell, row/range selection, typed cell rendering, pending-change queue with dirty/inserted/deleted/error states, sort/filter *requests* (`grid.rs:1-8`).
- **Database concepts in the reusable layer [F]:**
  - `CellValue { Null, Default, Text, Int, Num, Bool, Json }` `:32-41` — `Null`/`Default` are SQL.
  - `CellKind { Text, Id, Number, Bool, Timestamp, Json, Enum }` `:65-73`, with per-kind pixel widths `:76-86`.
  - `ColumnSpec { primary, nullable, read_only, references: Option<String>, enum_values, type_label, … }` `:93-106` — primary keys, nullability, foreign keys.
  - `RowTotal { Exact, Estimated, Unknown }` `:152-156`, `GridRows { rows, total, more }` `:159-164` — server-side paging.
  - `PendingChanges { cells, inserted, deleted }` `:187-191` and `UndoAction` `:169-182` — a commit queue.
  - `GridEvent` `:242-261` includes `SortRequested`, `FetchMore`, `CommitRequested`, `DiscardRequested`, **`PreviewSql`**, `FollowReference`, `FilterOnCell`, `OpenFilters`, `ClearFilters`.
  - `default_validator` `:267-322` parses UUIDs (`:306-312`), `YYYY-MM-DD` (`:313-319`), JSON object/array (`:291-298`), enum membership (`:299-305`), and emits `"{} is NOT NULL"` `:273`.
  - Pending-bar buttons literally labelled `"Preview SQL"`, `"Discard"`, `"Save"` `:400-404`.
  - `impl Theme { fn change_glyph(RowState) }` `:2007-2018` — a domain method grafted onto the theme from inside the grid.
- Constructors/methods (public): `new(id, Vec<ColumnSpec>)` `:367`; `editable(bool)->Self` `:408`; `len/is_empty/rows/source_row/value` `:415-435`; `set_rows` `:438`; `append_rows` `:462`; `set_loading` `:472`; `row_state` `:512`; `is_editing/edit_error` `:528-533`; `begin_edit` `:539`; `commit_edit` `:590`; `cancel_edit` `:623`; `record_cell` `:628`; `toggle_delete` `:649`; `insert_row` `:692`; `duplicate_row` `:724`; `undo() -> bool` `:748`; `apply_commit_result(Result<(),(usize,String)>)` `:778`; `discard` `:811`; `on_key` `:922`; `header_id/cell_id/rownum_id/more_id/left_id/right_id` `:1189-1206`; `owns` `:1208`; `locate/locate_header/locate_rownum` `:1220/:1232/:1235`; `on_click(id,pos)` `:1241`; `on_drag(pressed,pos)` `:1332`; `on_wheel(delta,horizontal)` `:1355`; `on_scrollbar` `:1364`; `on_paste` `:1376`; `position_label/rows_label/cols_label/pending_label` `:1389-1448`; `render(area,buf,ctx,bg)` `:1509`; `bar_ids()` `:1926`; `on_bar_key(id,key)` `:1931`. Free fns `cell_text` `:1969`.
- Public mutable fields (16): `columns, total, more, sort, local_sort, filtered_cols, cursor, selected_rows, pending, scroll, hscroll, edit, editable, read_only_reason, loading, cell_errors, row_errors, empty, validator, row_numbers, area` `:328-355`. Private: `rows, order, anchor, undo, body, widths, col_rects, show_bar, bar`.
- **Invariant coupling defect [I]:** `pub columns` `:329` is mutable but `widths` `:357` is private and only rebuilt by `set_rows`→`sample_widths` `:451`,`:480`. Pushing a column without calling `set_rows` makes `self.widths[i]` panic at `layout_columns` `:1458`.
- Owned app data: **all of it** — the rows, the pending edit queue, the undo stack, the per-cell and per-row error maps.
- Owned interaction state: cursor, anchor, selected_rows, scroll ×2, edit.
- Frame-local: `pub area` `:355`; private `body` `:356`, `widths` `:357`, `col_rects` `:358`.
- **Render-time semantic mutation [F]:**
  - `grid.rs:1510` — `self.fit_header_marks()` mutates `self.widths` during render.
  - `grid.rs:1518-1520` — `if !focused && self.edit.is_some() { self.commit_edit(); }`. `commit_edit` runs the validator and writes into `self.pending.cells` (`:590-621` → `record_cell` `:628-647`). **Drawing an unfocused grid commits a pending database edit.**
- Keyboard `:922-1172`: full edit-mode keymap plus ↑↓/k/K/j/J, Ctrl+←→ page columns, ←→/h/H/l/L, Ctrl+Home/End, Home/End, PgUp/PgDn, `g`/`G`, Enter/F2, Space select, Esc clear, Delete/Backspace null-or-delete, `-` delete row, `+` insert, Ctrl+D duplicate, `y`/`Y` copy, Ctrl+S commit, `s`/`S` sort, `f` filter-on-cell, `/` open filters, `F` clear filters, `r`/F5 refresh, `u` undo, `U` discard, `p` preview SQL, Ctrl+`]` follow reference. All hard-coded; several are application chords.
- Mouse: `on_click(id, pos)` `:1241` — dispatches bar buttons, more-row, left/right column steppers, scrollbar, header, row number, cell, and the trailing `→` reference affordance `:1307-1316`. `on_drag(pressed, pos)` `:1332` does range selection by scanning `col_rects`.
- Paste `:1376`; wheel v/h `:1355`; tick via spinner `:1688`.
- Event: `GridEvent` (18 variants) `:242-261`.
- Focus: `ctx.control(self.id, area, false)` `:1521` + `ctx.scrollable` `:1522`. The three pending-bar buttons are separate focus stops registered by `Button::render` `:1921`, and the owner must route their keys back through `on_bar_key` `:1931` — a bespoke second dispatch path.
- Child identity: `header_id = id.sub("header").child(col)` `:1190`; `cell_id = id.child(display).child(col)` `:1192` — **keyed on the display row, so scrolling or sorting re-labels every cell**; `rownum_id = id.sub("rownum").child(display)` `:1195`.
- Hit registration: per-cell `ctx.clickable` `:1878` (≈ rows×cols per frame), row-number gutter `:1762`, header `:1628`, more-row `:1697`, column steppers `:1578`,`:1645`.
- owns/locate: `owns` `:1208` (7 clauses incl. the bar buttons), `locate` `:1220` (O(visible rows × visible cols) scan), `locate_header` `:1232`, `locate_rownum` `:1235`.
- Scroll: two `ScrollState`s (`scroll` rows, `hscroll` **columns**) `:344-345`; `on_scrollbar` `:1364`.
- Overlay: none, but `GridEvent::OpenViewer` `:254` asks the owner to open one.
- Theme deps: `row` `:1712`, `gutter` `:1684`,`:1723`, `field_style` `:1771`, `popover` (range selection) `:1818`, `accent` `:1732`, `warning` (dirty) `:1741`,`:1822`,`:1894`, `error` `:1744`,`:1825`,`:1836`, `text_*` ladder, `lift` `:1574`,`:1641`, `canvas` as the reversed-cursor foreground `:1832`,`:1866`.
- Raw bg: `bg: Color` `:1509`, forwarded to `empty::render` `:1658`, `progress::render_spinner` `:1650`, `Button::render` `:1921`.
- Hard-coded glyphs/dims: `"▎"` `:1684`,`:1722`; `"✓"` `:1729`; `•`/`+`/`−`/`!` change glyphs `:1739-1745`,`:2011-2016`; `"▪ "` + `"⚷"` primary-key marks `:1610`,`:1625`; `" ∇"` filter mark `:1603`; `" ▴"`/`" ▾"` `:363-364`; `"→"` reference `:1865`; `"‹N"`/`"N›"` `:1570`,`:1636`; `"↓ … Enter fetches more"` `:1692`; column gap `2` `:1455`; gutter `3+num_w+1` `:1551`; sample cap 200 rows / p95 `:489-493`; header cap `24` `:496`; text-cell sanitiser cap `10_000` chars `:1989`.
- Tests: `grid.rs:2058-2190` — dirty-revert clears change, delete+undo, insert+undo, edit validation by kind, key navigation + sort requests + local sort keyed by source row, range selection + copy, position labels, fetch-more row, commit-result folding.
- **Disposition — split (goal §18).** A generic `Grid` (columns, rows, viewport, cursor, selection, editing/validation hooks, custom cell renderers, row decoration, sort/filter *requests*) plus a TablePro adapter owning `CellValue::{Null,Default}`, `ColumnSpec::{primary,nullable,references,enum_values}`, `PendingChanges`, `UndoAction`, `default_validator`, `apply_commit_result`, and every SQL-named `GridEvent` variant.

---

**12. `hintbar` (`HintLayer`, `HintBar`)**
- Responsibility: the one key-hint surface, resolved from a stack of layers (`hintbar.rs:1-6`).
- API: `HintLayer { hints: Vec<Hint>, badge: Option<(&'static str, BadgeKind)>, status: Option<(String,Tone)>, centered: bool }` `:14-20`; `new` `:23`, `centered` `:31`, `badge` `:35`, `status` `:39`; `HintBar::resolve(&[Option<HintLayer>]) -> HintLayer` `:50`; `HintBar::render(area,buf,t,&HintLayer) -> usize` `:56`.
- Render signature: `(&Theme, &HintLayer)` — no `RenderCtx`, no `bg`.
- Invariant [F]: first present layer wins `:51`; returns the number of hints that fit `:56`.
- Interaction/focus/hit: none.
- Hard-coded: delegates all layout to `keyhint::render_aligned` `:57`; badge type is `BadgeKind` which has exactly one variant (`theme.rs:568-570`).
- Domain assumptions: none.
- Tests: `hintbar.rs:74-113` — topmost layer wins + empty fallback; narrow rows drop from the right and mark with `…`.
- **Disposition — retain**, and connect to per-component hint metadata so screens stop hand-authoring the same layers (goal §13, last paragraph).

---

**13. `input` (`TextInput`, `InputEvent`)**
- Responsibility: single-line field with navigation vs editing modes (`input.rs:13-17`).
- Constructors/builders: `new(id,label)` `:59`; `placeholder` `:81`, `value` `:85`, `disabled` `:89`, `required` `:93`, `help` `:97`, `plain_label` `:101`, `validator(fn(&str)->Option<String>)` `:105`, `masked()` `:109`, `reveal_tail(u8)` `:113`.
- Methods: `clear()` `:119`; `text() -> &str` `:148`; `begin_edit` `:152`; `commit` `:161`; `cancel` `:167`; `validate() -> bool` `:174`; `const HEIGHT: u16 = 3` `:184`; `on_key -> (Outcome,Option<InputEvent>)` `:188`; `on_paste` `:230`; `on_click(pos, was_focused) -> Outcome` `:247`; `render(area,buf,ctx,bg)` `:263`.
- Public mutable fields: `id, label, placeholder, buffer, disabled, required, help, error, editing, area, validator, plain_label, masked, reveal_tail` `:20-44`. Private: `snapshot: String` `:30`, `scroll: usize` `:32`, `text_area: Rect` `:35`.
- Config: `validator: Option<fn(&str) -> Option<String>>` `:36` — **bare fn pointer**, cannot capture domain state.
- Owned app data: the value (uncontrolled). No external-sync API beyond `value()` builder and `buffer.set_text`.
- Owned interaction state: `editing`, `snapshot`, `scroll`, `error`.
- **Render-time semantic mutation [F]:** `input.rs:282-286` — on focus loss during render, `self.commit()`, which calls `self.validate()` `:164` and therefore runs the caller's validator and sets `self.error`. **Drawing runs validation.** Goal §11 forbids "run validation because focus changed" during render explicitly.
- Also mutates `self.scroll` `:363-371` and `self.text_area` `:346` in render (frame-local, acceptable).
- Keyboard: nav mode accepts only Enter and F2 `:193-196`; edit mode delegates to `edit_key(key,false)` `:199`, mapping `Cancel`→revert `:204-207` and `Tab`→commit+`CommittedTab` `:208-214`.
- Mouse: `on_click(pos, was_focused)` `:247` — the *caller* must tell the widget whether it was focused, which is how the "second click starts editing" rule is implemented (`:250-257`).
- Paste `:230-237`; no wheel/drag/tick.
- Event: `InputEvent { Committed, Cancelled, CommittedTab{backward}, Changed }` `:48-56`.
- Focus: `ctx.control(self.id, field, self.disabled)` `:424` — registers the *field row*, not the 3-row block.
- Hit/child identity: single control.
- Theme deps: `field_style` `:335`, `gutter` `:337`, `placeholder` `:350`, `selection` `:390`, `accent` (underline, required `*`) `:395`,`:318`, `error` `:421`,`:434`, `label`/`faint`/`muted` `:302-306`,`:325`,`:441`.
- Raw bg: `bg: Color` `:268`.
- Hard-coded: `"▎"` `:338`; `"*"` required `:298`,`:316`; `"  optional"` suffix `:300`,`:322`; `"!"` error `:420`; `"…"` overflow `:398`,`:409`; `HEIGHT = 3` `:184`; label x-offset `+2` `:307`; text x-offset `+2` `:341`; trailing reserve `2` when error `:339`.
- Domain assumptions: none, but the masked/reveal-tail behaviour encodes a product rule (see §5).
- Tests: **none in-module**; covered by `showcase/app_tests.rs:358-384`.
- **Disposition — decompose.** Value control + a shared field wrapper (label / required / optional / help / error), closure or trait validators, explicit `sync_value` for controlled use, and a secret-safe value type.

---

**14. `keyhint` (`Hint`, `hint`, `render`, `render_toned`, `render_aligned`)**
- Responsibility: `key Action` pairs on one row with priority-free right-to-left dropping (`keyhint.rs:1-2`).
- API: `struct Hint { key: &'static str, action: &'static str }` `:10-13`; `const fn hint(...)` `:15`; three render entry points `render` `:22`, `render_toned` `:43`, `render_aligned` `:56` — the first two are thin wrappers (`:30-37`, `:51`).
- Signature: `(area, buf, &Theme, &[Hint], badge, right, centered) -> usize` — `&Theme`, no ctx, no bg.
- Invariants [F]: status text always wins the right edge `:71-91`; two cells are reserved for the `…` cut marker `:106`,`:127`; an `Error`-toned status is prefixed with a bold `!` `:73-88`.
- Interaction/focus/hit: none.
- Hard-coded: `"…"` `:137`; `"! "` `:74`; leading x offset `+1` `:69`; inter-hint gap `2` `:101`,`:125`; badge padding `:93`.
- Constraint [I]: `&'static str` for both fields `:11-12` forbids runtime-built hints (e.g. "Ctrl+B m rename" with a user keymap).
- Tests: via `hintbar.rs:86-113`.
- **Disposition — retain**, widen to `Cow<'_, str>` and connect to component-supplied action descriptors.

---

**15. `list` (`ListItem`, `SelectMode`, `ListBox`)**
- Responsibility: scrollable single/multi-select list, one focus stop with an internal row cursor (`list.rs:43-44`).
- Constructors/builders: `ListItem::new/meta/disabled` `:20-34`; `ListBox::new(id, Vec<ListItem>, SelectMode)` `:61`; `empty_text` `:77`.
- Methods: `row_id` `:82`; `checked_count` `:86`; `move_cursor` (private) `:90`; `activate(i) -> Outcome` `:107`; `on_key -> Outcome` `:118`; `on_click(row: usize) -> Outcome` `:170`; `on_wheel` `:180`; `locate(id) -> Option<usize>` `:186`; `owns(id)` `:191`; `on_scrollbar(pos)` `:195`; `render(area,buf,ctx,bg)` `:207`.
- Public mutable fields: `id, items, cursor, mode, chosen, checked, scroll, area, empty_text` `:47-55`; private `anchor` `:57`.
- Owned app data: `Vec<ListItem>` with owned `label`/`meta` strings — Scenario D (borrowed domain rows + custom renderer) is impossible.
- Owned interaction state: `cursor`, `chosen`, `checked: Vec<bool>`, `anchor`, `scroll`.
- **Invariant coupling defect [I]:** `pub items` and `pub checked` are independent public vectors sized at construction (`:69`). Pushing to `items` without pushing to `checked` panics at `self.checked[li]` `:248` / `:113`.
- Render-time mutation: `scroll.set_content/set_viewport` `:215-216` only (viewport metadata).
- Keyboard `:118-167`: ↑↓/k/K/j/J, PgUp/PgDn, Home/`g`, End/`G`, Enter/Space activate, `a` select-all in multi mode. Shift extends only in multi mode `:92-99`.
- Mouse: **`on_click(row: usize)`** `:170` — takes an *index*, so the owner must call `locate()` first. Contrast `chips`/`menu`/`picker`/`select`/`tabs`/`grid` which take a `WidgetId`.
- Event: **none** — `Outcome` only; the owner polls `chosen`/`checked`.
- Focus: `ctx.control` `:218` + `ctx.scrollable` `:219`, registered *before* the rows so rows win hit-testing `:217`.
- Child identity: `row_id(i) = id.child(i)` `:82` — index-based.
- owns/locate: `locate` scans only `visible_range()` `:187`; `owns` `:191`.
- Theme deps: `row` `:253`, `gutter` `:255`, `accent`/`text_secondary` markers `:262-266`, `muted` `:228`,`:293`.
- Raw bg: `bg: Color` `:207`.
- Hard-coded: `"▎"` `:255`; `"›"`/`"✓"` markers `:256-260`; label offset `+3` `:284`; meta hidden when the label would drop below `12` cells `:278-282`; default empty text `"Nothing here yet"` `:72`.
- Tests: `list.rs:331-355` — wheel moves the viewport, render does not reset it, keyboard pulls the cursor back into view, wheel clamps.
- **Disposition — refactor** into the generic collection: borrowed items, stable keys, custom row renderer, cursor vs value separated, semantic activation events.

---

**16. `menu` (`MenuItem`, `Placement`, `ContextMenu`, `MenuBar`, `MenuEvent`, `MenuBarEvent`)**
- Responsibility: an anchored command popover plus a menu bar that opens the same popover (`menu.rs:1-5`).
- `ContextMenu` API: `new(id, Vec<MenuItem>)` `:85`; `anchor(rect, placement)` `:98`; `at(pos)` `:105`; `title` `:109`; `row_id` `:114`; `locate` `:118`; `owns` `:122`; `size() -> (u16,u16)` `:127`; `placed` (private) `:143`; `step` (private) `:173`; `on_key -> (Outcome,Option<MenuEvent>)` `:188`; `on_click(id) -> Option<MenuEvent>` `:219`; `on_click_outside() -> Option<MenuEvent>` `:231`; `render(screen,buf,ctx)` `:235`.
- `MenuBar` API: `new(id, Vec<(&str, Vec<MenuItem>)>)` `:378`; `brand(Lockup)` `:391`; `label_id` `:396`; `brand_id` `:400`; `is_open` `:404`; `open_index` `:408`; `owns` `:414`; `open_menu(i)` `:421`; `close` `:432`; `on_key -> (Outcome,Option<MenuBarEvent>)` `:436`; `on_click(id) -> (Outcome,Option<MenuBarEvent>)` `:486`; `on_hover(Option<WidgetId>) -> Outcome` `:517`; `render(area,buf,ctx,bg)` `:533`; `render_open(screen,buf,ctx)` `:598`.
- Public mutable fields: `ContextMenu{id,items,cursor,anchor,placement,area,title}` `:72-82`; `MenuBar{id,labels,menus,brand,cursor,open,areas,brand_area}` `:365-375`.
- **Render-time semantic mutation [F]:** `menu.rs:243-248` — render reads `ctx.interaction.hover`, locates the row, and writes `self.cursor = i`. **Drawing changes the selected command**; a subsequent Enter activates whatever the pointer last hovered.
- Keyboard: ↓/j, ↑/k, Home/`g`, End/`G`, Enter/Space, Esc `:189-215` (all other keys `Consumed` `:214`). Bar adds ←→/h/l `:440-449`,`:467-474` and Enter/↓/Space to open `:475`.
- Mouse: `ContextMenu::on_click(id) -> Option<MenuEvent>` (no `Outcome`) `:219`; `MenuBar::on_click(id) -> (Outcome, Option<MenuBarEvent>)` `:486`. Two shapes in one module.
- Hover: `MenuBar::on_hover` `:517` switches the open menu — an explicit hover handler no other widget has.
- Event: `MenuEvent{Chosen,Dismissed}` `:66-69`; `MenuBarEvent{Opened,Chosen(menu,item),Closed,Brand}` `:354-362`.
- Focus: `ContextMenu` registers **no** focus stop — it is drawn on top and the owner routes keys. `MenuBar` uses `ctx.control(self.id, area, false)` `:540`.
- Hit registration: popover surface `ctx.clickable(self.id, area)` `:260`; rows `:339`; bar labels `:584`; brand via `Lockup::render_clickable` `:544`. **No hit or focus barrier is pushed** — an open menu does not make the page beneath inert.
- Child identity: `row_id = id.child(i)` `:114`; `MenuBar` menu id = `label_id(i).sub("menu")` `:427`, recovered by scanning in `open_index()` `:408-412`.
- `size()` `:127` is the only intrinsic-size method on an overlay.
- Own `Placement { Below, Above, Right }` `:56-63` + own `placed()` clamping `:143-171`, duplicating `ui::popup::place`.
- Theme deps: `popover` (plane) `:251`, `highlight` / `highlight_danger` (cursor row) `:302-306`, `error_soft` (danger at rest) `:311`,`:320`, `lift` `:315`,`:570`, `border_subtle` `:256`,`:343`, `disabled` `:298`, `focus` (bar cursor bar) `:578`, `text_muted` (shortcut) `:334`.
- Raw bg: `MenuBar::render(…, bg: Color)` `:533`; `ContextMenu::render` hard-codes `t.popover` `:251`.
- Hard-coded: rounded border `:253-256`; separator `"─"` `:343`; label padding `" {label} "` `:552`; row inset `+2` `:327`; `"▎"` bar cursor `:576`; width formula `label_w.max(title_w).max(8) + 6` `:139`; brand gap `2` `:546`.
- Domain assumptions: `MenuItem::shortcut: Option<&'static str>` `:21` — a *rendered* shortcut string with no binding, so key handling and the displayed hint can drift.
- Tests: `menu.rs:630-748` — keyboard skips disabled/wraps/chooses; placement clamps and flips; click selects rows and outside dismisses (+ danger tone and right-aligned shortcut); hover moves the cursor; menu bar opens/switches/chooses/toggles/brand/Esc.
- **Disposition — compose.** Menu = overlay + list-of-commands; merge the two `Placement` enums and the two placement algorithms; move the hover→cursor rule out of render into an explicit pointer event.

---

**17. `panel` (`PanelKind`, `Panel<'a>`, `ScrollPanel`)**
- Responsibility: surface container (card / framed) plus an unrelated scrollable read-only text panel (`panel.rs:1-8`, `:174-175`).
- `Panel<'a>` API: `card(Option<&'a str>)` `:39`; `framed(Option<&'a str>)` `:49`; `focused(bool)` `:59`; `meta(&'a str)` `:63`; `bg(&Theme) -> Color` `:69`; `render(area, buf, &Theme) -> Rect` `:80` (**returns the inner area**, takes `&Theme` not ctx); private `title_row` `:127`.
- Public mutable fields: `title, kind, focused, meta, badge, bg_override` `:29-36`. `badge` has **no builder** — field mutation only (`:34`), unlike `meta` `:63`.
- `Panel` is the library's only *borrowing* component (`&'a str` throughout `:28-36`).
- `ScrollPanel` API: `new(id, Vec<String>)` `:188`; `wrap(bool)` `:199`; `push(String)` `:204`; `on_key -> Outcome` `:209`; `on_wheel` `:237`; `on_scrollbar` `:243`; `render(area, buf, ctx, bg, style_line: fn(&Theme,&str)->Style)` `:257`.
- **`style_line: fn(&Theme, &str) -> Style`** `:263` — a render-time bare function pointer; the only widget whose *render* takes an extension callback, and it cannot capture.
- Public mutable fields: `id, lines, scroll, follow, wrap, area` `:178-183`; private `wrapped_cache: (u16, Vec<String>)` `:184`.
- Render-time mutation: `scroll.set_content/set_viewport` `:285-286`, `wrapped_cache` rebuild `:275-281`, and `if self.follow { self.scroll.jump_end(); }` `:289-291` — viewport metadata, permitted by goal §11.
- Keyboard: ↑↓/k/j, PgUp/PgDn, Home/`g`, End/`G` (sets follow), `f` toggles follow `:211-230`.
- Event: none — `Outcome` only, even though `follow` changes state the owner may want to reflect. Contrast `TextViewport` which emits `ViewportEvent::FollowChanged` `:548`,`:557`.
- owns/locate: **absent** despite owning a scrollbar (`:299-302`) and an `on_scrollbar` (`:243`).
- Focus/hit: `ctx.control` `:287` + `ctx.scrollable` `:288`.
- Theme deps: `Panel` — `surface`/`canvas` `:74-76`, `border(focused)` `:111`, `focus` (title bar) `:92`, `title`/`secondary` `:133-137`, `faint` `:158`, `badge` `:166`. `ScrollPanel` — via the caller's `style_line`.
- Raw bg: `Panel::render` takes `&Theme` and derives bg itself `:85`, but exposes `pub bg_override: Option<Color>` `:36`; `ScrollPanel::render` takes `bg: Color` `:262`.
- Hard-coded: card margin `(2,1)` `:89`; framed margin `(1,1)` then `+2 / -3` `:116-122`; `"▎"` `:92`; rounded borders `:110`; framed title/meta wrapped in spaces `:141`,`:152`; text width `area.width - 2` `:272`.
- Tests: **none.**
- **Disposition — split.** `Panel` becomes the surface primitive (and the source of contextual background inheritance, removing most `bg:` params). `ScrollPanel` is a strict subset of `TextViewport` — **remove it** and migrate callers.

---

**18. `picker` (`PickerItem`, `PickerStatus`, `Picker`, `PickerEvent`)**
- Responsibility: centred modal with a query field and grouped ranked rows; ranking is the owner's job (`picker.rs:1-3`).
- API: `new(id, title)` `:83`; `set_items` `:103`; `set_cursor` `:111`; `row_id` `:117`; `locate` `:120`; `owns` `:123`; `step` (private) `:127`; `on_key -> (Outcome,Option<PickerEvent>)` `:147`; `on_click(id) -> Option<PickerEvent>` `:223`; `on_wheel -> Outcome` `:234`; `render(screen, buf, ctx, hints: &str)` `:244`.
- **Render signature takes a `hints: &str`** `:244` — a rendered hint row passed as a raw string, unique in the library, and documented as "owners with a shell-level hint bar pass an empty string" `:516`.
- Public mutable fields: `id, status, title, placeholder, query, items, cursor, scroll, width, max_rows, scope, empty_text, area, searchable` `:44-63`; private `cursor_dirty` `:64`.
- Owned app data: `Vec<PickerItem>` (owned `label`/`detail`, `&'static str` for `glyph`/`group`/`tag` `:19-28`), plus the query string.
- Owned interaction state: `cursor`, `scroll`, `cursor_dirty`, `query`.
- Render-time mutation: `scroll.set_content/set_viewport` `:348-349`, and the `cursor_dirty` handshake `:351-354` (a deliberate mechanism so a wheel scroll survives a render — see the tests `:574-592`). Also `ctx.begin_modal()` `:260` and the backdrop dim `:247-259`.
- Keyboard `:147-221`: Esc (clears query first, then cancels), Enter (+ Alt → `ChosenAlt`), ↑↓, Ctrl+n/j/p/k, PgUp/PgDn, Tab → `NextScope`, Delete → `Secondary`, Backspace (→ `Back` on empty query), Ctrl+U clear, any char appends to the query, `j`/`k` only when `!searchable`. Everything else `Consumed` `:219`.
- Mouse: `on_click(id) -> Option<PickerEvent>` `:223` — no `Outcome`.
- Wheel: returns `Consumed` at the boundary, `Changed` otherwise `:234-242` — the only widget with this refinement.
- Event: `PickerEvent{QueryChanged, Chosen, ChosenAlt, Secondary, NextScope, Cancelled, Back}` `:68-80`.
- Focus: **none** — raw `ctx.hits.register(self.id, area)` `:279` and `ctx.begin_modal()` `:260`; the query field cursor is placed via `ctx.set_cursor` `:336-339`. The owner must route all keys.
- Child identity: `row_id = id.child(i)` `:118`.
- **Renders a scrollbar (`:506-515`) but has no `on_scrollbar`** — same defect as `completion` and `textarea`.
- Overlay: dims the backdrop identically to `Dialog` (`:247-259` vs `dialog.rs:366-372`) — duplicated code, and both leave the last row live.
- Theme deps: `backdrop` `:255`, `surface_elevated` `:272`, `border(true)` `:277`, `title` `:289`, `field_style` (with a synthetic `VisualState{focused:true, editing:true}`) `:308-312`, `focus` `:318`, `accent` (query underline) `:333`, `row` `:435`, `gutter` `:438`, text ladder `:446-499`, `faint` (hints) `:371`,`:388`,`:522`.
- Raw bg: none (internal `t.surface_elevated`), but forwards `bg` to `progress::render_spinner` `:362` and `empty::render` `:381`,`:399`.
- Hard-coded: width `64` `:93`, `max_rows` `12` `:94`; height `2+1+query_rows+rows+2` `:267`; margin `(2,1)` `:280`; `"▎"` `:317`,`:438`; label column clamped to 45 % of the row `:414`; column offsets `+3`, `+2` `:454`,`:491`; placeholder `"Type to search…"` `:88`; `empty_text` `"No matches"` `:96`.
- Domain assumptions: `PickerItem::tag`/`group` as `&'static str` `:23`,`:22` forces compile-time strings for what is naturally runtime data.
- Tests: `picker.rs:573-620` — wheel scrolls and survives re-render, keyboard pulls the cursor into view, boundary wheel is `Consumed` not `Changed`.
- **Disposition — compose.** Picker = modal overlay + query field + generic list. Drop `hints: &str`; deliver hints as component metadata to the shared `HintBar`.

---

**19. `progress` (spinner, bar, meter, indeterminate)**
- Responsibility: four unrelated progress presentations plus the shared spinner frames (`progress.rs`).
- API: `const SPINNER: [&str;10]` `:8`; `spinner_frame(tick) -> &'static str` `:10`; `ProgressStatus{Active,Done,Error,Paused}` `:15-20`; `render_bar(area,buf,ctx,label,ratio,status,bg)` `:23`; `METER_LOW_MAX`/`METER_MEDIUM_MAX` consts `:86-87`; `MeterLevel{Low,Medium,High}` + `of(pct)` `:90-106`; `MeterVisual{Line,Block}` `:110-117`; `MeterTone{Normal,Level,Warning,Exhausted,Stale,Refreshing,Error,Unknown}` `:123-141` + `level()` `:145`; `Meter{used_pct,value,tone,visual}` `:164-169` with builders `:180-191`, private `palette` `:194`, `render(area,buf,ctx,bg)` `:220`; `render_meter(...)` `:326`; `render_indeterminate(...)` `:342`; `render_spinner(...)` `:377`.
- Public mutable fields: `Meter{used_pct, value, tone, visual}` `:165-168`.
- Interaction/focus/hit: **none** — all four are pure draw calls; there is no busy/progress *control*.
- Render-time mutation: none.
- Tick: `spinner_frame(ctx.interaction.tick)` `:228`, `:364`, `:386`.
- Theme deps: `text_secondary`/`success`/`error`/`text_muted`/`warning` for fills `:62-67`,`:196-217`; `border_subtle` for the empty track `:72`,`:267`; `accent` for the indeterminate sweep `:368` and the spinner `:387`; `lift(bg)` for the block remainder `:289`; `canvas` as on-fill text `:293`.
- Raw bg: every entry point takes `bg: Color` `:30`,`:224`,`:333`,`:347`,`:381`.
- Hard-coded: braille spinner frames `:8`; `━`/`─` track glyphs `:70-73`,`:265-267`; suffix glyphs `✓ ! ‖` `:49-52` and `▲ !` `:211-215`; `pct_w = 5` `:46`; minimum track `6` `:55`,`:253`; block min `4` `:279`; segment `track/5` clamped 2..8 `:362`; label gap `2` `:44`,`:359`,`:389`; `"—"` for unknown `:229`; `STATUS_METER_TRACK = 10` lives in `statusbar.rs:42`, not here.
- Domain assumptions: `MeterTone::{Warning, Exhausted, Stale, Refreshing}` `:129-140` are Jackin quota-lifecycle states in a generic component; the doc says "green is never used: a quota is not a completion" `:156-157` — a Junie budget rule.
- Tests: `progress.rs:416-497` — level thresholds, line-mode run colours, block-mode fill, and each domain marker state.
- **Disposition — decompose.** `Spinner`, `ProgressBar` (determinate + indeterminate as a variant), `Meter` as three components with one status/tone vocabulary; move quota lifecycle states to Jackin.

---

**20. `props` (`Prop`, free `render`, `PropsList`, `PropsEvent`)**
- Responsibility: label/value fact rows, in a static and an interactive flavour (`props.rs:1-2`, `:97-98`).
- API: `Prop::new(label, value)` `:27`, builders `copyable` `:36`, `tone` `:40`, `wrap` `:44`; free `render(area, buf, &Theme, &[Prop], bg) -> u16` `:51` (returns rows used); `PropsList::new(id, Vec<Prop>)` `:109`; `set_props` `:120`; `row_id` `:126`; `locate` `:129`; `owns` `:132`; `set_cursor` (private) `:136`; `on_key -> (Outcome,Option<PropsEvent>)` `:141`; `on_click(row: usize) -> (Outcome,Option<PropsEvent>)` `:168`; `on_wheel` `:176`; `on_scrollbar` `:181`; `render(area,buf,ctx,bg)` `:193`.
- **Two render paths for the same content [F]:** the free `render` (`&Theme`, no ctx, no interaction, `:51`) and `PropsList::render` (`&mut RenderCtx`, `:193`) with independent layout maths (`:56-61` vs `:207-213`). `Dialog` uses the free one `:421-427`.
- Public mutable fields: `Prop{label,value,tone,wrap,copyable}` `:17-24`; `PropsList{id,props,cursor,scroll,area}` `:101-105`.
- Owned app data: owned `label`/`value` strings.
- Render-time mutation: `scroll.set_content/set_viewport` `:201-202` only.
- Keyboard: ↑↓/k/j, PgUp/PgDn, Home/`g`, End/`G`, Enter → `Activate`, `y` → `Copy` (only when `copyable`) `:145-163`.
- Mouse: `on_click(row: usize)` — index-based, like `list`/`tree`/`steps`.
- Event: `PropsEvent{Copy(usize), Activate(usize)}` `:92-95`.
- Focus: `ctx.control` `:203` + `ctx.scrollable` `:204`.
- owns/locate: `:129`, `:132`.
- Theme deps: `muted` `:67`, `tone` `:69`,`:237`, `row` `:221`, `gutter` `:223`, `text_muted` `:228`, `text_faint` `:244`.
- Raw bg: both render paths `:51`, `:193`.
- Hard-coded: label column = max label width + 2 `:56-61`,`:207-213`; `"▎"` `:223`; value offset `+2 + label_w` `:230`; the literal hint string `"y copy"` and its reserved 8 cells `:231`,`:241-245`.
- Tests: **none.**
- **Disposition — refactor.** One `DescriptionList` with a static (non-interactive) mode; `wrap` becomes a row-render option; the `"y copy"` affordance becomes hint metadata, not painted text.

---

**21. `scrollbar` (free functions)**
- Responsibility: draw a 1-column vertical scrollbar, register it, and map track positions to offsets.
- API: `const TRACK: &str = "│"` `:8`; `const THUMB: &str = "┃"` `:9`; `id_for(container) -> WidgetId` `:12`; `render_vertical(area, buf, ctx, container, &ScrollState, focused)` `:18`; `offset_for_click(track, pos, &ScrollState) -> usize` `:47`; `position_label(&ScrollState) -> String` `:53`.
- Invariants [F]: draws nothing when the content fits `:27`; the id is always `container.sub("scrollbar")` `:13`, which is why every `owns()` checks `scrollbar::id_for(self.id)`.
- Interaction: `ctx.clickable(sb_id, area)` `:43`; hovered = hover **or** pressed `:32` (so the thumb stays lit through a drag).
- Theme deps: `scrollbar_track` `:40`, `scrollbar_thumb(focused, hovered)` `:38`.
- Raw bg: none — **it never sets a background**, so the track/thumb inherit whatever was painted underneath.
- Hard-coded: the two glyphs `:8-9`; vertical only (no horizontal scrollbar exists anywhere in the library); `position_label` format `"a–b of n"` `:58`.
- Tests: **none in-module**; exercised by `showcase/app_tests.rs:514-532`.
- **Disposition — retain as a part.** Make it a part of a scroll-container primitive that also owns the drag handling, so the three widgets that draw it without an `on_scrollbar` handler (`completion`, `picker`, `textarea`) stop being broken.

---

**22. `segments` (`Segment`, free `render`)**
- Responsibility: one line of labelled facts with priority-based dropping (`segments.rs:1-4`).
- API: `Segment::new(text, tone)` `:25`, builders `bold` `:34`, `clickable(id)` `:38`, `priority(u8)` `:42`; `render(area, buf, ctx, left: &[Segment], right: &[Segment], bg)` `:50`.
- Public mutable fields: `text, tone, bold, id, priority` `:15-22`.
- **Duplication [F/I]:** the priority-drop loop `:83-101` is a second, incompatible implementation of `statusbar::StatusBar::layout` `:136-262` (which drops center→right→left and protects the strongest left item). Two components, two algorithms, same problem.
- Interaction: `ctx.clickable` for segments with an id `:117-122`; hover lifts `:105-107`.
- Focus: none. Events: none.
- Theme deps: `tone` `:104`, `lift` `:106`, `text_primary` `:106`.
- Raw bg: `bg: Color` `:56`.
- Hard-coded: separator width `2` `:63`; clickable segments padded with one space each side `:111-115`; leading `+1` inset `:124`.
- Tests: **none.**
- **Disposition — merge with `statusbar`.** One "priority strip" with left/center/right groups, one drop algorithm.

---

**23. `select` (`Select`, `SelectEvent`)**
- Responsibility: closed dropdown field with a trailing `▾`; open state shows an anchored popup list (`select.rs:1-2`).
- API: `const HEIGHT: u16 = 3` `:33`; `new(id, label, &[&str], selected)` `:35`; `help` `:48`; `disabled` `:52`; `value() -> &str` `:56`; `option_id` `:62`; `locate` `:65`; `owns` `:68`; `on_key -> (Outcome,Option<SelectEvent>)` `:72`; `on_click(id) -> (Outcome,Option<SelectEvent>)` `:125`; `dismiss() -> Outcome` `:144`; `render(area,buf,ctx,bg)` `:153`.
- Public mutable fields: `id, label, options, selected, cursor, open, disabled, help, area` `:16-24`.
- Owned app data: `options: Vec<String>` (owned copies) — no borrowed or custom-rendered options.
- Owned interaction state: `selected`, `cursor`, `open`.
- **Render-time semantic mutation [F]:** `select.rs:161-167` —
  - `if self.disabled { … self.open = false; }`
  - `if !s.focused { self.open = false; }`
  
  **Drawing closes the overlay.** Goal §11 lists "close an overlay" as a forbidden render side effect verbatim.
- Keyboard: open state ↑↓/k/j, Enter/Space commit, Esc revert `:77-101`; closed state Enter/Space open, ↑/← and ↓/→ change the value *without opening* `:103-122`.
- Mouse: `on_click(id)` toggles the field or picks an option `:125-141`; `dismiss()` for click-outside `:144`.
- Event: `SelectEvent::Changed(usize)` `:28-30` — emitted only when the index actually changes `:92`,`:113`,`:119`,`:138`.
- Focus: `ctx.control(self.id, field, self.disabled)` `:199` — one stop; the open list is not in the ring.
- Child identity: `option_id(i) = id.child(i)` `:62`.
- Overlay: `ui::popup::place` + `ui::popup::surface` `:212-213` — inherits the shared `"popup.surface"` id collision and the hit-only barrier.
- **No scrolling** — the popup height is `min(options.len()+2, 10)` `:210` and rows past `inner.bottom()` are simply dropped `:216-218`. An 11-option select silently hides options.
- Theme deps: `field_style` `:179`, `gutter` `:185`, `label`/`faint` `:168-172`, `text_secondary`/`disabled` (chevron) `:193-197`, `row` `:223`, `accent` (`›` marker) `:233`, `muted` (help) `:205`.
- Raw bg: `bg: Color` `:153`.
- Hard-coded: `"▎"` `:184`,`:229`; `"▴"`/`"▾"` `:192`; `"›"` `:233`; `HEIGHT = 3` `:33`; popup width `field.width.clamp(12,40)` `:211`; row offsets `+2`/`+3` `:188`,`:237`. Dead code: `let _ = Constraint::Length(0);` `:270` in `picker.rs` and an unused `Constraint` import chain here.
- Tests: **none.**
- **Disposition — rebuild** on the overlay + list primitives, with scrolling, an explicit `close()` lifecycle call, and no render-time state change.

---

**24. `splitter` (`Splitter`)**
- Responsibility: the mouse affordance for a `Split` gap strip (`splitter.rs:1-5`).
- API: `const fn new(id, dir)` `:23`; `render(area, buf, ctx, bg)` `:33`; `on_drag(&self, split: &mut Split, container, gap, pos) -> Outcome` `:56`.
- Public mutable fields: `id, dir, area` `:17-19`.
- **The only handler that takes the model it mutates as a parameter** (`&mut Split` `:56`) — the splitter is stateless w.r.t. the ratio, which is arguably the right design and is *unlike* every other widget.
- Interaction: `ctx.clickable(self.id, area)` `:52`; hover/pressed change the border weight and glyph `:40-48`.
- Focus: **none** — "keyboard resize stays a chord on the owning pane" `:2-3`. So a keyboard-only user cannot resize through this component.
- Theme deps: `border(hovered || pressed)` `:42`.
- Raw bg: `bg: Color` `:33`.
- Hard-coded: glyph table `│ ┃ ─ ━` `:43-48`.
- Tests: **none in-module**; `Split` itself is tested at `ui/layout.rs:155-188`.
- **Disposition — retain**, but pair `Split` + `Splitter` into one `SplitPane` component that also owns the keyboard resize action.

---

**25. `statusbar` (`Emphasis`, `StatusItem`, `Group`, `Placed`, `StatusBar`)**
- Responsibility: a full-width status row with left/center/right groups and priority-based dropping (`statusbar.rs:1-6`).
- API: `Emphasis{Plain,Strong,Chip}` `:19-27`; `StatusItem::new(text, tone)` `:45`, builders `meter` `:56`, `priority` `:60`, `clickable` `:64`, `strong` `:68`, `chip` `:72`, `width()` `:78`; `const STATUS_METER_TRACK: u16 = 10` `:42`; `Group{Left,Center,Right}` `:94-98`; `Placed{group,index,x,width,text}` `:102-109`; `StatusBar{left,center,right}` (all public `Vec`s) `:112-116`; `new()` `:123`; `layout(area) -> Vec<Placed>` `:136`; `render(&self, area, buf, ctx)` `:264`.
- **`render(&self, …)`** `:264` — the only widget whose render takes `&self`. This is the one component with *no* render-time mutation, because layout is a pure function returning `Placed` (`:136-262`) and is separately testable.
- Public mutable fields: the three `Vec<StatusItem>` `:113-115`.
- Interaction: `ctx.clickable(id, …)` for items with an id `:301-303`; hover lifts `:281-283`.
- Focus: none.
- Composition: embeds a `Meter` inline `:290-300`.
- Theme deps: `surface_elevated` (its own plane) `:270`, `tone` `:275`, `surface_overlay` (chips) `:278`, `lift` `:282`.
- Raw bg: **none** — hard-codes `t.surface_elevated` `:270`. This makes it un-composable onto another plane.
- Hard-coded: `GAP = 3` `:119`, `EDGE = 1` `:120`, `STATUS_METER_TRACK = 10` `:42`, meter reservation `+1+track+7` `:86`; chip padding `" text "` `:284-287`.
- Tests: `statusbar.rs:344-410` — group ordering and sides, narrow-row drop order with the name preserved and truncated, render fills the plane and registers hover on the chip.
- **Disposition — retain** as the model for how every component should separate `layout()` from `render()`; merge the drop algorithm with `segments`.

---

**26. `steps` (`StepState`, `Step`, `StepRail`)**
- Responsibility: ordered lifecycle rail, optionally a focus stop with a cursor (`steps.rs:1-4`).
- API: `StepState{Queued,Running,Done,Skipped,Failed,Blocked}` + `label()` `:29`, `terminal()` `:40`; `Step::new(&str)` `:57`; `StepRail::new(id, Vec<Step>)` `:79`; `selectable(bool)` `:92`; `set_state(i, state)` `:97`; `set_meta(i, Option<String>)` `:103`; `frontier()` `:110`; `counts()` `:115`; `failed()` `:124`; `row_id` `:128`; `locate` `:132`; `owns` `:136`; `set_cursor` (private) `:140`; `on_key -> Outcome` `:145`; `on_click(row: usize) -> Outcome` `:161`; `on_wheel` `:169`; `on_scrollbar` `:174`; `render(area,buf,ctx,bg)` `:186`.
- Public mutable fields: `id, steps, selectable, cursor, scroll, area, numbered` `:67-76`.
- Event: **none** — `Outcome` only, even for `on_click`, so the owner polls `cursor`.
- Dead code [F]: `frontier` is computed at `:202` and discarded via `let _ = frontier;` `:288`; `let _ = Style::new();` `:294`.
- Render-time mutation: `scroll.set_content/set_viewport` `:194-195` only.
- Keyboard: ↑↓/k/j, Home/`g`, End/`G` — only when `selectable` `:146-158`.
- Focus: `ctx.control` **conditionally** `:196-198`; `ctx.scrollable` unconditionally `:199` — so a non-selectable rail is a wheel target but not a Tab stop.
- Child identity: `row_id = id.child(i)` `:128`.
- Theme deps: `row` `:213`, `gutter` `:215`, `accent` (running spinner) `:220`, `success` `:222`, `error` `:223`,`:247`,`:264`, `warning` (blocked meta) `:265`, text ladder `:233-266`.
- Raw bg: `bg: Color` `:186`.
- Hard-coded: `"▎"` `:215`; `"✓"` `:222`, `"!"` `:223`; ordinal format `{:02}` `:228`; column offsets `+1`, `+3`, `+3` `:215`,`:226`,`:240`; meta hidden unless `avail >= mw + 12` `:274`; default meta strings `"queued"/"skipped"/"blocked"` `:256-260`.
- Tests: `steps.rs:302-316` — frontier and counts.
- **Disposition — refactor** into a list variant with a state column and a per-row status glyph; emit an activation event.

---

**27. `table` (`SortDir`, `Align`, `Column`, `Cell`, `EditState`, `DataTable`, `TableEvent`)**
- Responsibility: data table with header sorting, row/cell navigation, hover and in-place cell editing (`table.rs:93-94`).
- **Duplication [F]:** this is a second, independent implementation of everything `grid` does — its own `EditState` (`table.rs:86-91` vs `grid.rs:233-239`), its own sort cycling, its own `layout_columns`, its own edit-commit-tab loop, its own `locate`/`owns`. `grid` even imports `SortDir` from here (`grid.rs:27`).
- API: `SortDir{Asc,Desc}` `:15-18`; `Align{Left,Right}` `:21-24`; `Column::new(title, Constraint)` `:36`, `right` `:45`, `editable` `:49`, `min_width` `:53`; `Cell::new` `:72`, `tone` `:79`; `pub use crate::theme::Tone;` `:69` (a re-export leak); `DataTable::new(id, cols, rows)` `:135`, `cell_nav` `:160`, `numeric(&[usize])` `:164`, `validator(fn(usize,&str)->Option<String>)` `:172`, `empty_text` `:176`; `len/is_empty/is_editing` `:181-189`; `source_row` `:192`; `header_id/row_id/cell_id` `:196-204`; `set_rows` `:206`; `apply_sort` (private) `:216`; `sort_by(col) -> Outcome` `:237`; `begin_edit -> Outcome` `:285`; `commit_edit -> Option<TableEvent>` `:304`; `cancel_edit -> Option<TableEvent>` `:334`; `on_key -> (Outcome,Option<TableEvent>)` `:338`; `on_paste` `:445`; `on_click_header(col) -> Outcome` `:455`; `on_click_cell(display, col, pos) -> (Outcome,Option<TableEvent>)` `:464`; `on_wheel_h` `:501`; `on_wheel` `:511`; `on_scrollbar` `:516`; `render(area,buf,ctx,bg)` `:557`; `locate -> Option<(usize,Option<usize>)>` `:797`; `owns` `:811`; `locate_header` `:818`; `edit_error` `:822`.
- Public mutable fields: `id, columns, rows, sort, cursor_row, cursor_col, cell_nav, selected, scroll, hscroll, edit, validator, empty_text, area, numeric` `:97-119`; private `order`, `body`, `col_rects`.
- **Invariant coupling defect [I]:** `pub rows: Vec<Vec<Cell>>` and `pub columns` are independently mutable, but render indexes `self.rows[src][ci]` `:700` and sorting indexes `rows[a][col]` `:220-221`. A ragged row panics.
- Owned app data: **the rows themselves** — `commit_edit` writes `self.rows[src][e.col].text = text` `:317`.
- **Render-time semantic mutation [F]:** `table.rs:566-568` — `if !focused && self.edit.is_some() { self.commit_edit(); }`. `commit_edit` runs the validator (`:308`), **writes into `self.rows`** (`:317-318`), and may re-sort and move the cursor (`:319-327`). **Drawing an unfocused table mutates application data.** This is the most severe instance of the class.
- `sort_by` (`:237`) mutates `order`, `sort`, `cursor_row` and `scroll` — and is reachable from `on_click_header` `:455`, which also silently `commit_edit()`s first `:456-458`.
- Keyboard `:338-443`: edit-mode via `edit_key(key,false)` plus ↑↓/k/j, ←→/h/l (cell nav or h-scroll depending on `cell_nav`), PgUp/PgDn, Home/`g`, End/`G`, `s` sort, Enter/F2 edit, Enter/Space activate. Hard-coded.
- Mouse: `on_click_header(col: usize)` and `on_click_cell(display, col, pos)` — **index-based**, so the owner must run `locate()`/`locate_header()` first (`showcase` and `tablepro` do exactly the `owns → locate → dispatch` chain goal §12 wants removed).
- Event: `TableEvent{Committed{row,col}, Cancelled, Activated(usize), LeaveForward, LeaveBackward}` `:122-132`.
- Focus: `ctx.control` `:570` + `ctx.scrollable` `:571`.
- Child identity: `row_id = id.child(display)` `:199`, `cell_id = id.child(display).child(col)` `:202` — display-index keyed, so sorting re-labels every row.
- Hit registration: cells `:779`, row `:781`, then **cells again** `:783-788` so cell hover beats row hover — a manual z-ordering hack.
- Theme deps: `row` `:679`, `gutter` `:685`, `accent` (`›` marker, edit underline) `:689`,`:733`, `field_style` `:718`, `canvas`/`text_primary` (reversed cell cursor) `:750-753`, `error` `:755`,`:757`,`:735`, `border_strong` (hover underline) `:763`, `tone` `:711`, `muted`/`faint`/`primary` for headers `:602-608`,`:635`,`:655`.
- Raw bg: `bg: Color` `:557`.
- Hard-coded: `"▎"` `:684`; `"›"` `:687`; `" ▴"`/`" ▾"` `:611-613`; `"…"` overflow marks `:635`,`:642`,`:729`; `"!"` `:743`,`:773`; gutter width `3` `:576`; column gap `2` `:533`,`:550`; right reserve `5` `:579`.
- Tests: `table.rs:877-955` — sort cycles asc/desc/none, numeric sort, cursor keeps its source row, edit commit/cancel, validation blocks commit, Tab moves to the next editable cell and leaves at the end.
- **Disposition — merge with `grid`.** One generic table with pluggable cell rendering and editing; `DataTable` and `DataGrid` should not both exist.

---

**28. `tabs` (`TabItem`, `Tabs`, `TabEvent`)**
- Responsibility: horizontal tab strip with state glyphs, close affordances and `‹ ›` overflow scrolling (`tabs.rs:1-7`).
- API: `TabItem::new(&str)` `:32`, builders `closable` `:38`, `prefix` `:42`, `suffix` `:46`; `Tabs::new(id, &[&str])` `:78` **and** `Tabs::with_items(id, Vec<TabItem>)` `:92` (two constructors for the same thing); `tab_id/close_id/new_id/left_id/right_id` `:106-120`; `locate` `:122`; `owns` `:126`; `hidden() -> (usize,usize)` `:136`; `set_active(i)` `:141`; `remove(i)` `:152`; `ensure_visible` (private) `:165`; `on_key -> (Outcome,Option<TabEvent>)` `:173`; `on_click(id) -> (Outcome,Option<TabEvent>)` `:216`; `tab_width` (private) `:240`; `render(area,buf,ctx,bg)` `:258`.
- Public mutable fields: `id, items, active, cursor, first, areas, allow_new, quiet` `:53-67`; private `fit` `:67`.
- **Identity [F/I]:** `tab_id(i) = id.child(i)` `:107`, `close_id(i) = id.child(i).sub("close")` `:110`. `remove(i)` `:152-163` shifts every subsequent tab's identity. Scenario E ("close actions and pending edits remain associated with the logical item") **fails by construction**.
- Owned interaction state: `active`, `cursor`, `first` (strip scroll), `fit`.
- Frame-local: `pub areas: Vec<Rect>` `:60`, rebuilt each render `:265`.
- Render-time mutation: `self.areas` `:265`, `self.first` `:291`,`:307-309`, `self.fit` `:313` — strip scrolling is computed during draw. Frame-local/viewport metadata, but `first` is *durable* state that persists across frames and is mutated only here and in `on_click` `:220-227`.
- Keyboard `:173-213`: ←→/h/l, digits `1`–`9`, Enter/Space, `x`/Delete close, `n` new. Hard-coded.
- Mouse: `on_click(WidgetId)` `:216`.
- Event: `TabEvent{Activated(usize), Close(usize), New}` `:70-75` — indices, so the owner must re-map after any removal.
- Focus: **raw** `ctx.ring.register(self.id)` `:488-490` — no `ctx.control`, so the strip registers no hit region of its own; only individual tab rects are clickable `:440`.
- Hit registration: tab `:440`, close `:427` and again `:443` (double registration so close wins), new `:486`, left/right steppers `:338`,`:473`.
- Theme deps: `lift`/`lift(lift())` for planes `:359-363`, `accent` vs `border_strong` for the active rule `:430-434`, `border_subtle` baseline `:273`, `text_primary/secondary/muted/faint` `:368-372`,`:383`,`:394-398`,`:424`, `warning` (dirty) `:415`, `error` `:412`, spinner `:407`.
- Raw bg: `bg: Color` `:258`.
- Hard-coded: `"─"` baseline and `"━"` active rule `:271`,`:437`; `"×"` close `:426`; `"•"` dirty `:415`, `"!"` error `:412`; `" + "` new `:485`; `"‹{:<3}"` / `"{:>3}›"` overflow labels `:333`,`:459`; overflow reserve `4`+`4`, new-tab reserve `4` `:278`,`:285-286`; tab padding and inter-tab gap `1` `:241`,`:445`; two-row height assumption `:267`,`:429`.
- Tests: `tabs.rs:518-599` — active plane + the only accent underline + no gutter glyph; hover vs cursor vs active planes; suffix glyph placement.
- **Disposition — refactor** with caller-supplied stable keys, key-based events, and `ctx.control` registration.

---

**29. `textarea` (`TextArea`)**
- Responsibility: multi-line editor with the same two modes as `TextInput`, but Esc *commits* (`textarea.rs:16-18`).
- API: `new(id, label, rows)` `:37`; builders `value` `:54`, `placeholder` `:59`, `disabled` `:63`, `error(Option<&str>)` `:67`, `help` `:71`; `height() -> u16` `:76`; `begin_edit` `:80`; `commit` `:86`; `on_key -> (Outcome, Option<InputEvent>)` `:91`; `on_paste` `:158`; `on_click(pos, was_focused)` `:166`; `on_wheel` `:184`; `render(area,buf,ctx,bg)` `:189`.
- **Reuses `InputEvent` from `input`** `:13`, `:91` — so a textarea emits `InputEvent::Committed` and `CommittedTab`, but never `Cancelled` (Esc maps to commit `:121-124`). One event enum, two meanings.
- Public mutable fields: `id, label, placeholder, buffer, disabled, error, help, editing, scroll, area, rows` `:21-33`; private `text_area` `:31`.
- **Render-time semantic mutation [F]:** `textarea.rs:202-205` — `if !s.focused && self.editing { self.commit(); }`.
- Keyboard: nav mode Enter/F2 edit, ↑↓/k/j scroll, PgUp/PgDn `:96-118`; edit mode via `edit_key(key,true)` `:120`, with `Commit | Cancel` folded to the same arm `:121-124`.
- Mouse: `on_click(pos, was_focused)` `:166` (same contract as `input`); `on_wheel` `:184`. **No `on_scrollbar`**, yet it draws a scrollbar unconditionally `:296-297` (not even guarded by `overflows()` at the call site — `render_vertical` guards internally at `scrollbar.rs:27`).
- Focus: `ctx.control(self.id, body, self.disabled)` `:222` + `ctx.scrollable` `:223`.
- owns/locate: **absent**.
- No validator (unlike `input`), no `required`, no `plain_label`, no masking.
- Theme deps: `field_style` `:224`, `gutter` `:226`, `placeholder` `:243`, `selection` `:269`, `border_strong` (cursor-line underline) `:281`, `error_fg` `:303`,`:328`, `muted` `:261`,`:334`, `faint` `:342`.
- Raw bg: `bg: Color` `:189`.
- Hard-coded: `"▎"` per body row `:228`; `"…"` `:260`; `"!"` `:302`; `height() = rows + 2` `:77`; inner inset `+2 / -4` `:230`; label offset `+2` `:213`; position label format `"ln a/b"` `:311`.
- Tests: **none in-module**; covered by `showcase/app_tests.rs:386-401`.
- **Disposition — refactor** onto the same editor core as `code`, with the field wrapper shared with `input`; distinct event enum or a shared `EditEvent` with explicit `Committed{via}`.

---

**30. `tree` (`TreeNode`, `Path`, `FlatRow`, `TreeView`, `TreeEvent`)**
- Responsibility: lazily-loaded, filterable tree flattened to rows (`tree.rs:12-13`).
- API: `TreeNode::leaf/leaf_meta/dir/lazy/note` `:30-74`, builders `glyph` `:64`, `meta` `:76`; `type Path = Vec<usize>` `:83`; `FlatRow{path,depth,label,meta,has_children,expanded,glyph,busy,note}` `:86-96`; `TreeView::new(id, Vec<TreeNode>)` `:121`; `rows()` `:141`; `flatten()` `:145`; `node/node_mut` `:207/:217`; `set_children(path, children)` `:227`; `set_busy(path, bool)` `:237`; `set_filter(Option<&str>)` `:244`; `reveal(path)` `:252`; `row_id(i)` `:262`; `toggle_id(i)` `:266`; `set_cursor` (private) `:270`; `toggle(i) -> (Outcome, Option<TreeEvent>)` `:275`; `expand_all/collapse_all` `:302/:317`; `on_key -> (Outcome, Option<TreeEvent>)` `:322`; `on_click_row(i)` `:403`; `on_click_toggle(i)` `:421`; `on_wheel` `:426`; `locate(id) -> Option<(usize,bool)>` `:432`; `owns` `:444`; `on_scrollbar` `:448`; `render(area,buf,ctx,bg)` `:460`.
- Public mutable fields: `id, nodes, expanded: HashSet<Path>, cursor, selected: Option<Path>, scroll, area, filter` `:107-118`; private `rows: Vec<FlatRow>` `:115`.
- **Identity split [F]:** `selected` and `expanded` are keyed by `Path` (stable across reorder), but `row_id(i) = id.child(i)` `:262` and `toggle_id(i) = id.child(i).sub("toggle")` `:266` are keyed by the **flattened row index**, which changes on every expand/collapse/filter. So hit identity and selection identity disagree.
- Owned app data: the whole `Vec<TreeNode>` with owned labels/meta; `glyph: Option<&'static str>` `:21` forces compile-time glyphs.
- Owned interaction state: `expanded`, `cursor`, `selected`, `scroll`, `filter`.
- Cached derivation: `rows` rebuilt by `flatten()` `:145-205`, called from `set_children` `:234`, `set_busy` `:241`, `set_filter` `:246`, `reveal` `:255`, `toggle` `:288`,`:292`,`:298`, `expand_all` `:314`, `collapse_all` `:319` — **explicitly, never from render.** This is the correct pattern and the only widget that gets it right.
- Render-time mutation: `scroll.set_viewport` `:468` only. **Clean.**
- Keyboard `:322-401`: ↑↓/k/j, PgUp/PgDn, Home/`g`, End/`G`, →/l expand-or-descend, ←/h collapse-or-ascend, Enter/Space toggle-or-activate, `*` expand-all, `-` collapse-all. Hard-coded.
- Mouse: `on_click_row(i)` / `on_click_toggle(i)` — **index-based**, so the owner runs `locate()` `:432` and branches on the `bool`.
- Event: `TreeEvent{Expand(Path), Activate(Path)}` `:99-104` — **path-keyed, the only correctly-identified events in the library.**
- Focus: `ctx.control` `:469` + `ctx.scrollable` `:470`.
- Hit registration: row `:570`, toggle registered twice (`:523` and `:573`) so it wins over the row.
- Theme deps: `row` `:499`, `gutter` `:501`, `accent` (busy spinner, selected label) `:515`,`:539`, `text_secondary` (fold glyph) `:517`, `text_muted` (glyph, meta, note) `:549`,`:541`,`:567`.
- Raw bg: `bg: Color` `:460`.
- Hard-coded: `"▎"` `:501`; `"▾"`/`"▸"` `:510`; indent `depth * 2` `:502`; toggle hit width `2` `:523`; meta hidden unless `avail - (meta+2) >= 10` `:533`; first level auto-expanded in `new()` `:134-136`.
- Tests: `tree.rs:608-628` — wheel moves the viewport, render does not reset it, keyboard pulls the cursor into view.
- **Disposition — refactor.** Keep the `Path` identity and the explicit `flatten()` lifecycle; move hit identity onto `Path`; add borrowed nodes and a custom node renderer.

---

**31. `viewport` (`Span`, `Line`, `CellPos`, `TextViewport`, `ViewportEvent`)**
- Responsibility: selectable read-only styled text with bounded retention, tail-follow, wrapping, drag/word selection, copy and an optional caret (`viewport.rs:1-5`).
- API: `Span::new/plain/muted` + `bold/italic/underline/reversed` `:33-65`; `type Line = Vec<Span>` `:68`; `line_text(&[Span]) -> String` `:70`; `CellPos{line,col}` `:75-79`; `TextViewport::new(id)` `:129`, `with_lines` `:149`, `wrap(bool)` `:155`, `max_lines(n)` `:160`; `push(Line)` `:165`; `set_lines` `:183`; `replace_last` `:190`; `clear` `:199`; `len/is_empty` `:207-213`; `selection()` `:228`; `set_area(area)` `:239`; `has_anchor` `:255`; `has_selection` `:259`; `clear_selection` `:263`; `is_at_tail` `:271`; `scrollback_depth` `:276`; `set_follow(bool)` `:280`; `pos_at(Position)` `:393`; `selected_text()` `:409`; `select_word_at(Position)` `:437`; `on_click(pos)` `:477`; `on_drag(pos)` `:490`; `on_wheel` `:513`; `on_scrollbar` `:519`; `owns` `:532`; `on_key -> (Outcome, Option<ViewportEvent>)` `:537`; `render(area,buf,ctx,bg)` `:580`.
- Public mutable fields: `id, lines, max_lines, scroll, follow, wrap, caret, caret_visible, area` `:110-119`; private `selection, drag_anchor, cells, visual, layout_width, dirty` `:120-125`.
- **`set_area()`** `:239-252` — an explicit "lay out without drawing" entry point so an owner that renders a copy can route events against the same geometry. This is the only component that acknowledges the stale-geometry problem head-on.
- Render-time mutation: `ensure_layout` (cache, keyed on `dirty`+`layout_width`) `:590`,`:595`, `scroll.set_viewport` `:591`,`:596`, `if self.follow { self.scroll.jump_end(); }` `:598-600`. All viewport metadata — permitted.
- Keyboard `:537-578`: ↑↓/k/j, PgUp/PgDn, Home/`g`, End/`G` (sets follow), `f` toggle follow, `y` copy selection, Esc clear selection.
- Mouse: `on_click(pos)` anchors a drag `:477-486`; `on_drag` extends with edge auto-scroll `:490-511`; `select_word_at` for double-click `:437-474` (the *owner* must detect the double click); `on_wheel` `:513`; `on_scrollbar` `:519`.
- Event: `ViewportEvent{Copy(String), SelectionChanged, FollowChanged(bool)}` `:82-87`.
- Focus: `ctx.control` `:601` + `ctx.scrollable` `:602`; caret exposed via `ctx.set_cursor` only when following + focused + visible `:653-670`.
- owns: `:532` (no `locate` — there are no child ids).
- Theme deps: `tone` `:615`, `selection` `:633`,`:646`, `canvas`/`text_primary` for `reversed` spans `:626`.
- Raw bg: `bg: Color` `:580`.
- Hard-coded: tab expands to 4 spaces `:302-315`; word chars `alnum _ - / .` `:448-451`; trailing-selection gap width 1 `:645`.
- Tests: `viewport.rs:698-743` — tail follow + wheel leaves it + End restores; drag select + copy + word select; wrap + bounded retention.
- **Disposition — retain** as *the* scrollable-text primitive; absorb `ScrollPanel`; formalise `set_area()` into the general "measure/layout then render" contract.

---

## 3. API-inconsistency matrix (goal §7)

### 3.1 `Outcome` vs `(Outcome, Option<Event>)` vs `bool` vs `Option<Event>` vs polled field

| Shape | Occurrences (file:line) |
|---|---|
| `Outcome` only | `choice.rs:32,44,116,139,234,246`; `list.rs:118,170,180,195`; `panel.rs:209,237,243`; `steps.rs:145,161,169,174`; `select.rs:144`; `splitter.rs:56`; `table.rs:237,285,455,501,511,516`; `tree.rs:426,448`; `code.rs:530,548,557,566,578`; `input.rs:230,247`; `textarea.rs:158,166,184`; `grid.rs:1332,1355,1364,1376`; `dialog.rs:207,316,324,350`; `menu.rs:517`; `viewport.rs:263,437,477,490,513,519`; `props.rs:176,181`; `completion.rs:142`; `picker.rs:234` |
| `(Outcome, Option<Event>)` | `chips.rs:85,121`; `code.rs:289`; `completion.rs:94`; `grid.rs:922,1241,1931`; `menu.rs:188,436,486`; `picker.rs:147`; `props.rs:141,168`; `select.rs:72,125`; `table.rs:338,464`; `tabs.rs:173,216`; `textarea.rs:91`; `tree.rs:275,322,403,421`; `viewport.rs:537`; `input.rs:188` |
| `(Outcome, bool)` | `button.rs:72` |
| `bool` | `button.rs:86`; `grid.rs:748` |
| `Option<Event>` (no `Outcome`) | `completion.rs:136`; `menu.rs:219,231`; `picker.rs:223`; `grid.rs:590,649,692,724`; `table.rs:304,334` |
| polled result field | `dialog.rs:53` (`pub result: Option<DialogResult>`), read after `on_key`/`on_click` |
| `()` (draw-only, owner infers) | `brand.rs:58,65`; `empty.rs:46`; `keyhint.rs:22,43,56`; `hintbar.rs:56`; `progress.rs:23,220,326,342,377`; `props.rs:51`; `segments.rs:50`; `statusbar.rs:264` |

Also inconsistent: **wheel boundary semantics.** `picker.rs:234-242` and `table.rs:501-509` return `Consumed` when the offset did not move; `list.rs:180`, `tree.rs:426`, `code.rs:557`, `grid.rs:1355`, `viewport.rs:513`, `props.rs:176`, `steps.rs:169`, `panel.rs:237`, `completion.rs:142`, `textarea.rs:184` always return `Changed`.

### 3.2 Constructors

| Pattern | Sites |
|---|---|
| `new(id, …)` | 23 of 31 modules |
| Named variant constructors | `button.rs:39-55`; `brand.rs:24,31`; `dialog.rs:58,75,92,110`; `empty.rs:25,32`; `tree.rs:30,42,49,57,69`; `diff.rs:32,38,44`; `viewport.rs:34,44,47`; `panel.rs:39,49` |
| Two constructors for one type | `tabs.rs:78` (`new(&[&str])`) vs `:92` (`with_items(Vec<TabItem>)`); `viewport.rs:129` vs `:149` |
| Chained builders (`self`) | `button.rs:56`; `chips` (none); `code.rs:127-142`; `input.rs:81-116`; `list.rs:77`; `menu.rs:37-52,98,105,109,391`; `panel.rs:59,63`; `picker` (none); `progress.rs:180-191`; `props.rs:36-46`; `segments.rs:34-45`; `select.rs:48,52`; `statusbar.rs:56-75`; `steps.rs:92`; `table.rs:160-179`; `tabs.rs:38-49`; `textarea.rs:54-74`; `tree.rs:64,76`; `viewport.rs:155,160`; `choice.rs:229` (`Toggle` only — `Checkbox` has none) |
| Direct public-field mutation as the primary path | `chips.rs:35-43`; `grid.rs:328-355` (16 pub fields); `picker.rs:44-63`; `tabs.rs:53-67`; `tree.rs:107-118`; `statusbar.rs:112-116`; `panel.rs:34` (`badge`, no builder) |
| Setter methods | `code.rs:148,219`; `grid.rs:438,462,472`; `props.rs:120`; `picker.rs:103,111`; `steps.rs:97,103`; `table.rs:206`; `tabs.rs:141,152`; `tree.rs:227,237,244,252`; `viewport.rs:183,190,199,239,280`; `diff.rs:219,228` |

### 3.3 Internal vs caller-owned selection/data

| Widget | Selection ownership | Data ownership |
|---|---|---|
| `list` | internal `chosen`/`checked` `:51-52` | owned `Vec<ListItem>` `:48` |
| `tree` | internal `selected: Option<Path>` `:112` | owned `Vec<TreeNode>` `:109` |
| `table` | internal `selected: Option<usize>` `:108` | owned `Vec<Vec<Cell>>` `:99`, **mutated by the widget** `:317` |
| `grid` | internal `selected_rows` + `cursor` + `anchor` `:338-342` | private rows + a full pending-change queue `:330,:342` |
| `tabs` | internal `active` `:55` | owned `Vec<TabItem>` `:54` |
| `select` | internal `selected` `:19` | owned `Vec<String>` `:18` |
| `choice::RadioGroup` | internal `selected` `:92` | owned `Vec<String>` `:91` |
| `picker` | internal `cursor` `:51` | owned `Vec<PickerItem>` `:50` |
| `completion` | internal `cursor` `:33` | owned `Vec<CompletionItem>` `:32` |
| `props` | internal `cursor` `:103` | owned `Vec<Prop>` `:102` |
| `steps` | internal `cursor` `:71` | owned `Vec<Step>` `:69` |
| `chips` | internal `cursor` `:38` | owned `Vec<Chip>` `:37` |
| `menu` | internal `cursor` `:75`, **also written during render** `:247` | owned `Vec<MenuItem>` `:74` |
| `viewport` | internal `selection: Option<(CellPos,CellPos)>` `:120` | owned `Vec<Line>` `:111` |

**No component supports a caller-owned (controlled) value or borrowed data.** Every collection requires owned `String`s rebuilt by the application, and no component accepts a custom row/cell renderer. Goal §11 "controlled values and selections" and Scenario D are unimplemented across the board.

### 3.4 Cursor-vs-selection semantics conflict

- `list.rs:124-131` — arrows move the cursor only; Enter/Space activates `:151`.
- `choice.rs:121-130` — arrows move the cursor **and** set `selected` (navigation *is* activation).
- `select.rs:109-120` — arrows change the value while the popup is **closed**, without opening it.
- `tabs.rs:183-189` — arrows call `set_active` and emit `TabEvent::Activated` (navigation *is* activation).
- `menu.rs:190-196` — arrows move the cursor only.
- `grid.rs:990-1015` / `table.rs:393-416` — arrows move a cell cursor only.

Four different meanings for ←/↑/→/↓ across seven components.

### 3.5 Component-specific `owns` / `locate` / `row_id` / `close_id` / scrollbar handling

| Widget | `owns` | `locate` signature | id helpers | `on_scrollbar` | draws a scrollbar |
|---|---|---|---|---|---|
| `chips` | `:141` | **none** | `chip_id` `:67`, `close_id` `:70`, `add_id` `:73`, `lead_id` `:76` | — | no |
| `completion` | `:90` | `-> Option<usize>` `:86` | `row_id` `:82` | **missing** | **yes** `:231` |
| `diff` | `:260` (delegates) | — | — | `:280` | via viewport |
| `grid` | `:1208` | `-> Option<(usize,usize)>` `:1220` | `header_id` `:1189`, `cell_id` `:1192`, `rownum_id` `:1195`, `more_id` `:1198`, `left_id` `:1201`, `right_id` `:1204`, `bar_ids` `:1926` | `:1364` | yes `:1883` |
| `list` | `:191` | `-> Option<usize>` `:186` | `row_id` `:82` | `:195` | yes `:301` |
| `menu` | `:122` / `:414` | `-> Option<usize>` `:118` | `row_id` `:114`, `label_id` `:396`, `brand_id` `:400` | — | no |
| `picker` | `:123` | `-> Option<usize>` `:120` | `row_id` `:117` | **missing** | **yes** `:507` |
| `props` | `:132` | `-> Option<usize>` `:129` | `row_id` `:126` | `:181` | yes `:251` |
| `select` | `:68` | `-> Option<usize>` `:65` | `option_id` `:62` | — | no (clips at 10) |
| `steps` | `:136` | `-> Option<usize>` `:132` | `row_id` `:128` | `:174` | yes `:292` |
| `table` | `:811` | `-> Option<(usize, Option<usize>)>` `:797` + `locate_header` `:818` | `header_id` `:196`, `row_id` `:199`, `cell_id` `:202` | `:516` | yes `:792` |
| `tabs` | `:126` | `-> Option<usize>` `:122` | `tab_id` `:106`, `close_id` `:109`, `new_id` `:112`, `left_id` `:115`, `right_id` `:118` | — | no (own `first`/`fit`) |
| `tree` | `:444` | `-> Option<(usize, bool)>` `:432` | `row_id` `:262`, `toggle_id` `:266` | `:448` | yes `:578` |
| `viewport` | `:532` | — | — | `:519` | yes `:651` |
| `code` | **missing** | **missing** | — | `:566` | yes `:836` |
| `panel::ScrollPanel` | **missing** | **missing** | — | `:243` | yes `:301` |
| `textarea` | **missing** | **missing** | — | **missing** | **yes** `:296` |

Four distinct `locate` return types; three widgets draw an interactive scrollbar with no handler; three scroll containers have no `owns`.

### 3.6 Render methods requiring a raw background colour

`bg: Color` in the render signature — 24 sites:
`button.rs:106`, `chips.rs:148`, `choice.rs:52`, `choice.rs:153`, `choice.rs:254`, `code.rs:601`, `diff.rs:284`, `empty.rs:46`, `grid.rs:1509`, `list.rs:207`, `menu.rs:533`, `panel.rs:262`, `progress.rs:30`, `progress.rs:224`, `progress.rs:333`, `progress.rs:347`, `progress.rs:381`, `props.rs:51`, `props.rs:193`, `segments.rs:56`, `select.rs:153`, `splitter.rs:33`, `steps.rs:186`, `table.rs:557`, `tabs.rs:258`, `textarea.rs:189`, `tree.rs:460`, `viewport.rs:580`.

Not taking `bg` (each hard-codes its own plane instead, which is the mirror-image problem): `completion.rs:174` (`t.surface_elevated`), `dialog.rs:378` (`t.surface_elevated`), `picker.rs:272` (`t.surface_elevated`), `menu.rs:251` (`t.popover`), `statusbar.rs:270` (`t.surface_elevated`), `ui/popup.rs:66` (`t.surface_elevated`), `panel.rs:74-76` (derives from `PanelKind`, with a `pub bg_override` escape hatch at `panel.rs:36`).

`Panel::bg(&Theme) -> Color` (`panel.rs:69`) exists purely so callers can extract the value and hand it back to every child render — the manual surface-inheritance protocol that goal §15 says must become contextual.

### 3.7 Render methods that perform semantic state transitions

See §4 — 8 confirmed sites.

### 3.8 Components that expose internal rectangles for later routing

`pub area: Rect` — `button.rs:23`, `chips.rs:42`, `choice.rs:18`, `choice.rs:216`, `code.rs:81`, `completion.rs:39`, `dialog.rs:52`, `grid.rs:355`, `input.rs:33`, `list.rs:54`, `menu.rs:79`, `panel.rs:183`, `picker.rs:58`, `props.rs:105`, `select.rs:24`, `splitter.rs:19`, `steps.rs:73`, `table.rs:114`, `textarea.rs:30`, `tree.rs:114`, `viewport.rs:119`.

`pub areas: Vec<Rect>` — `choice.rs:96`, `menu.rs:373`, `tabs.rs:60`. Plus `menu.rs:374` `pub brand_area`, `completion.rs:37` `pub anchor`, `menu.rs:77` `pub anchor`.

All are written during render and read during event routing — so any frame in which a widget was not drawn (early return on an empty/tiny rect: `grid.rs:1512`, `table.rs:559`, `code.rs:604`, `list.rs:210`, …) leaves **stale geometry** that the next click will use.

### 3.9 Components that hard-code keyboard bindings

All 18 interactive modules. Representative: `button.rs:73` (Enter/Space); `chips.rs:91-118` (`x`, `+`, `X`); `choice.rs:36,121-134`; `code.rs:400-479` (`i`, `a`, `g`, `G`, `{`, `}`, `/`, `n`, `N`); `completion.rs:98-133` (Ctrl+n/p); `dialog.rs:263-311` (`y`/`n`, `h`/`l`); `field_common.rs:22-110` (the whole edit keymap); `grid.rs:989-1171` (`s`, `S`, `f`, `F`, `p`, `u`, `U`, `y`, `Y`, `+`, `-`, Ctrl+D, Ctrl+S, Ctrl+`]`); `list.rs:124-165` (`a` select-all); `menu.rs:189-215`; `panel.rs:211-230` (`f` follow); `picker.rs:148-220` (Tab = next scope, Delete = secondary); `props.rs:145-163` (`y` copy); `select.rs:77-122`; `steps.rs:149-158`; `table.rs:392-441` (`s` sort); `tabs.rs:182-212` (digits 1–9, `x`, `n`); `textarea.rs:96-155`; `tree.rs:326-398` (`*`, `-`); `viewport.rs:539-572` (`f`, `y`).

There is no keymap, command table, or action-descriptor indirection anywhere. `MenuItem::shortcut: Option<&'static str>` (`menu.rs:21`) is a *rendered label only*, with no binding behind it — display and behaviour can silently diverge.

### 3.10 Closed body/content enums

- `dialog.rs:18-28` — `DialogBody { Text, Input, Facts }`. Named explicitly in goal §14 as unacceptable.
- `grid.rs:32-41` — `CellValue` (7 variants, closed); `grid.rs:65-73` — `CellKind` (7 variants, closed, with hard-coded widths at `:76-86`).
- `theme.rs:559-565` — `ButtonKind` (5, closed); `theme.rs:534-543` — `Tone` (7, closed); `theme.rs:547-556` — `SyntaxTone` (8, closed); `theme.rs:568-570` — `BadgeKind` (**1 variant**).
- `progress.rs:15-20` `ProgressStatus`; `:110-117` `MeterVisual`; `:123-141` `MeterTone`; `:90-94` `MeterLevel`.
- `panel.rs:22-26` — `PanelKind { Card, Framed }`.
- `steps.rs:18-26` — `StepState` (6, closed).
- `list.rs:37-41` — `SelectMode { Single, Multi }` (no range/none).
- `statusbar.rs:19-27` — `Emphasis { Plain, Strong, Chip }`.
- `diff.rs:168-174` — `DiffMode { Unified, Review }`.
- `table.rs:21-24` — `Align { Left, Right }` (no centre).

None of these has a custom-variant escape hatch (goal §16: "do not force every possible custom design into a closed enum").

### 3.11 Components combining generic presentation with application-domain behaviour

| Site | Domain leak |
|---|---|
| `grid.rs:32-41,65-106,152-164,169-222,242-261,267-322,400-404,2007-2018` | SQL nulls/defaults, primary keys, nullability, foreign-key references, enum columns, server row totals, pending-change queue, undo stack, `PreviewSql`/`CommitRequested`/`DiscardRequested`/`FollowReference`/`FilterOnCell`/`OpenFilters`/`ClearFilters`, a UUID/timestamp/JSON validator, and buttons literally labelled "Preview SQL"/"Save" |
| `code.rs:74-77,161-186,219-221` | `running: Option<Range>` and the statement/block model — a SQL-runner concept |
| `completion.rs:22` | `glyph` documented as "T, V, C, K, F, S, A" — TablePro completion kinds |
| `chips.rs:40,61` | `lead` example `"match all ▾"` and default `"+ Add filter"` — TablePro filter bar |
| `progress.rs:123-141,156-157` | `MeterTone::{Warning,Exhausted,Stale,Refreshing}` — Jackin quota lifecycle; "green is never used: a quota is not a completion" is a Junie budget rule |
| `dialog.rs:24-27,31-34,110-136` | `Facts { code: Vec<String>, ack: Option<AckInput> }` — SQL preview + TablePlus Safe Mode typed acknowledgement |
| `brand.rs:1-4` | "the only control that fills with the accent" — a Junie green-budget rule encoded as a component |
| `tabs.rs:63-65` | `quiet` = "so one screen keeps one accent underline" — a Junie budget rule as a component flag |
| `statusbar.rs:1-6`, `segments.rs:1-4` | priority-drop ordering tuned to Jackin's status row |

### 3.12 Extension mechanisms restricted to bare function pointers

- `grid.rs:265` — `pub type Validator = fn(&ColumnSpec, &str) -> Result<CellValue, String>` (field at `:353`).
- `table.rs:112` — `pub validator: Option<fn(col: usize, &str) -> Option<String>>`.
- `input.rs:36` — `pub validator: Option<fn(&str) -> Option<String>>`.
- `code.rs:26` — `pub type Highlighter = fn(&str) -> Vec<(Range<usize>, SyntaxTone)>`; `:27` — `pub type Segmenter = fn(&str) -> Vec<Range<usize>>`.
- `panel.rs:263` — `style_line: fn(&Theme, &str) -> Style`, passed **into render**.
- `field_common.rs:12` — `Apply(fn(&mut TextBuffer))`.

None can capture application state (a connection, a schema, a locale, a config). Goal §19 names this explicitly.

### 3.13 Masked / secret-bearing controls

See §5.

### 3.14 Cross-cutting duplication

- **Two table implementations**: `table::DataTable` and `grid::DataGrid`, each with its own `EditState` (`table.rs:86-91`, `grid.rs:233-239`), sort cycling (`table.rs:243-247`, `grid.rs:1132-1136`), `layout_columns` (`table.rs:529-555`, `grid.rs:1452-1486`), Tab-to-next-editable-cell loop (`table.rs:346-373`, `grid.rs:933-960`), and `locate`. `grid` imports `SortDir` from `table` (`grid.rs:27`).
- **Two scrollable-text implementations**: `panel::ScrollPanel` (`panel.rs:177-304`) and `viewport::TextViewport` (`viewport.rs:109-671`).
- **Two priority-drop strips**: `segments::render` (`segments.rs:83-101`) and `StatusBar::layout` (`statusbar.rs:165-187`).
- **Two `Placement` enums + two placement algorithms**: `ui/popup.rs:16-56` and `menu.rs:56-171`; `Dialog` uses neither (`dialog.rs:376`).
- **Two backdrop-dim loops**: `dialog.rs:359-372` and `picker.rs:247-259`, byte-identical logic.
- **Two props render paths**: `props::render` free fn (`props.rs:51`) and `PropsList::render` (`props.rs:193`).
- **Two `EditState` + two `TextBuffer`-in-cell editors**, plus a third inline editor idiom in `input`/`textarea`/`code`.
- `row_layout` / `row_layout_right` live in `button.rs:167,179` but are the generic action-row primitive, used by `dialog.rs:473` and `grid.rs:1915`.

### 3.15 Measurement

No shared measurement protocol. Nine unrelated conventions: `TextInput::HEIGHT` const `input.rs:184`; `Select::HEIGHT` const `select.rs:33`; `TextArea::height()` `textarea.rs:76`; `RadioGroup::height()` `choice.rs:112`; `Button::width()` `button.rs:62`; `Lockup::width()` `brand.rs:46`; `StatusItem::width()` `statusbar.rs:78`; `ContextMenu::size() -> (u16,u16)` `menu.rs:127`; `Dialog::height(width) -> u16` `dialog.rs:168`. Everything else has none.

---

## 4. Render-time semantic-mutation findings

Goal §11 forbids render from committing an edit, cancelling an edit, running validation because focus changed, mutating application data, changing a selected value, closing an overlay, or altering focus. All eight confirmed violations:

| # | Site | What render does | Forbidden category |
|---|---|---|---|
| 1 | `input.rs:282-286` | `if !s.focused && self.editing { self.commit(); }` → `commit()` (`:161-165`) calls `validate()` (`:174-181`), which runs the caller's `fn` validator and writes `self.error` | **commit an edit** + **run validation because focus changed** |
| 2 | `textarea.rs:202-205` | `if !s.focused && self.editing { self.commit(); }` | **commit an edit** |
| 3 | `code.rs:611-614` | `if !s.focused && self.editing { self.commit(); s.editing = false; }` | **commit/cancel an edit** |
| 4 | `table.rs:566-568` | `if !focused && self.edit.is_some() { self.commit_edit(); }` → `commit_edit` (`:304-332`) runs the validator (`:308`), **writes `self.rows[src][e.col].text`** (`:317`), clears the cell error (`:318`), may re-sort (`:322`) and move the cursor + scroll (`:324-326`) | **mutate application data**, **reorder data**, **run validation**, **alter scroll/cursor** |
| 5 | `grid.rs:1518-1520` | `if !focused && self.edit.is_some() { self.commit_edit(); }` → `commit_edit` (`:590-621`) runs `(self.validator)` (`:603`) and `record_cell` (`:628-647`) writes into `self.pending.cells` and pushes an `UndoAction` | **commit an edit**, **mutate the pending-change queue**, **run validation** |
| 6 | `grid.rs:1510` | `self.fit_header_marks()` (`:1489-1507`) mutates `self.widths` before layout | durable layout state written during draw |
| 7 | `dialog.rs:465-470` | render compares `a.input.text().trim() == a.token` and writes `self.actions.last_mut().disabled = !armed` | **run validation**, **change a control's enabled state** |
| 8 | `menu.rs:243-248` | `if let Some(h) = ctx.interaction.hover { … self.cursor = i; }` — render moves the command cursor to whatever the pointer hovers | **change a selected value** |
| 9 | `select.rs:161-167` | `if self.disabled { self.open = false; }` and `if !s.focused { self.open = false; }` | **close an overlay** |

Additional render-time mutation that is *permitted* by goal §11 ("update non-semantic viewport metadata required by the current frame") but should be recorded because it is currently indistinguishable from the above:

- Viewport/scroll metadata: `list.rs:215-216`; `tree.rs:468`; `props.rs:201-202`; `steps.rs:194-195`; `table.rs:573-574`; `grid.rs:1542-1543`; `code.rs:631-632`; `textarea.rs:235-236`; `completion.rs:170-172`; `picker.rs:348-349`; `panel.rs:285-286`; `viewport.rs:591,596`.
- Follow-to-tail: `panel.rs:289-291`; `viewport.rs:598-600`.
- Cursor-into-view during editing: `code.rs:647-649`; `textarea.rs:238-240`; `picker.rs:351-354` (guarded by `cursor_dirty`).
- Frame-local geometry writes: `button.rs:112`; `chips.rs:153`; `choice.rs:55,166-173,257`; `code.rs:606,644`; `completion.rs:168`; `dialog.rs:377`; `grid.rs:1515,1541,1470-1485`; `input.rs:334,346`; `list.rs:212`; `menu.rs:238,545,583`; `panel.rs:269`; `picker.rs:271`; `props.rs:198`; `select.rs:178`; `splitter.rs:36`; `steps.rs:191`; `table.rs:562,585,551-554`; `tabs.rs:265,291,313`; `textarea.rs:221,231`; `tree.rs:465`; `viewport.rs:585`.
- Cache rebuilds: `code.rs:589-599` (highlight spans, hash-keyed); `panel.rs:275-281` (wrap cache); `viewport.rs:287-367` (cell/visual layout); `diff.rs:241-258` (**line generation — this one also calls `set_follow(false)` and `scroll_to`, so it is closer to category 6**).
- Barrier/modal establishment during draw: `dialog.rs:373`; `picker.rs:260`; `ui/popup.rs:74`.

**Structural root cause [I]:** `runtime::Application::render(&mut self, …)` (`runtime.rs:25`) and every widget `render(&mut self, …)` give render write access to durable state, and there is no separate "focus changed" or "layout" lifecycle hook. Focus loss is only *observable* at render time because `RenderCtx` is the only place a widget learns whether it is focused (`ui/ctx.rs:110-117`). Fixing sites 1–5 requires an explicit focus-transition callback, not a local patch.

---

## 5. Historical secret / masked-input findings

The facts and inferences in this section were collected from the historical
snapshot. Keep them as audit evidence; do not apply them wholesale to current
Jackin behavior.

**Collected facts**

1. `core/text.rs:10` — `#[derive(Debug, Clone, Default, PartialEq, Eq)] pub struct TextBuffer { text: String, … }`. `Debug` prints the raw text; `Clone` duplicates it; `PartialEq` compares it (a timing-observable comparison, though not attacker-controlled here).
2. `input.rs:18-19` — `#[derive(Debug, Clone)] pub struct TextInput`, with `pub buffer: TextBuffer` `:23` and private `snapshot: String` `:30`. Both appear in `Debug` output. `#[derive(Debug)]` does **not** honour the `masked` flag `:41`.
3. `input.rs:148` — `pub fn text(&self) -> &str` returns the raw value regardless of `masked`. There is no `SecretString`, no redacting wrapper, no `expose()` ceremony.
4. `input.rs:41` documents the hazard rather than preventing it: *"`text()` still returns the raw value, which is transient edit state only: never log or render it."* — a comment, not an invariant.
5. `input.rs:44,113,127-146` — `reveal_tail: u8` plus `display_graphemes`: while **not editing**, the last `reveal_tail` graphemes are drawn **in clear** (`:135-138`). Those cells reach the Ratatui buffer, therefore any `TestBackend` snapshot, `tools/capture.sh` PNG, or baseline digest.
6. `showcase/app_tests.rs:624-659` — `showcase_visual_baseline` hashes `cell.symbol()` for every cell into `tests/showcase_baseline.txt`. Any revealed tail is folded into a committed fixture (as a hash, but the *rendered* capture pipeline in `tools/` writes the glyphs verbatim).
7. `dialog.rs:43-44` — `#[derive(Debug, Clone)] pub struct Dialog` containing `DialogBody::Input(TextInput)` (`:20`) and `AckInput{ input: TextInput, token: String }` (`:30-34`). A `{:?}` on a prompt dialog prints the typed value **and** the acknowledgement token.
8. `input.rs:167-172` — `cancel()` does `let snap = self.snapshot.clone(); self.buffer.set_text(snap);` — a second heap copy of the old value with no zeroization; `set_text` (`core/text.rs:126-130`) drops the previous `String` without clearing it.
9. `input.rs:119-123` — `clear()` calls `buffer.set_text("")` and `snapshot.clear()`. `String::clear` truncates without zeroing; `set_text` replaces the allocation. Neither overwrites the bytes.
10. `grid.rs:233-239` and `table.rs:86-91` — `#[derive(Debug, Clone)] pub struct EditState { … buffer: TextBuffer … }`; `DataGrid` and `DataTable` both derive `Debug`/`Clone` (`grid.rs:326`, `table.rs:95`), so an in-flight cell edit is printable and cloneable.
11. `grid.rs:31` — `#[derive(Debug, Clone, PartialEq)] pub enum CellValue { …, Text(String), Json(String) }`; `DataGrid` holds every row, so `{:?}` on a grid dumps the entire result set.
12. `viewport.rs:22,89,108` — `Span`, `Cell` and `TextViewport` all derive `Debug`/`Clone` and hold arbitrary text (log/terminal content).
13. `README.md:60-62` claims *"No secret ever reaches a frame — … plain-text keys live in transient edit state and render masked with a synthetic four-character tail."* The **synthesis** is an application behaviour; the library's `reveal_tail` reveals the *real* last N graphemes (`input.rs:135-143`). In this historical snapshot, the safety property therefore lived in `src/bin/jackin_preview`, not in the reusable layer, and any other consumer of `TextInput::masked().reveal_tail(4)` leaked real characters.

**Historical inference**

- The library has **no** secret type, no redaction, no `Debug` suppression, and no zeroization. Every exposure path goal §19 lists — "rendering, captures, logs, cloning, or `Debug`" — is open.
- Minimum fixes: a `Secret<String>`-style newtype with a manual `Debug` writing `"[redacted]"`; a manual `Debug` impl on `TextInput`, `Dialog`, `AckInput`, `EditState` (both), `TextBuffer`; removal or re-specification of `reveal_tail` so the library synthesises the tail rather than revealing it; an explicit `expose()`/`with_value()` accessor instead of `text()`; and a conformance test asserting `format!("{:?}", masked_input)` contains neither the value nor the snapshot.

### Current Jackin behavior

The current environment implementation is `apps/jackin-preview/src/domain/workspace.rs`.
`EnvValue::Plain` is redacted by its debug formatter, while persisted plain
values are rendered through `mask`. For API-key-shaped values, that stored mask
intentionally retains the final four characters, so a masked tail may reach a
frame; this is not evidence that raw secret material is absent everywhere.
The editor keeps transient plain environment input masked and adds it to the
pending workspace only after `env_key_error` accepts the key and Save is
activated. These current facts are separate from the historical library
findings above.

---

## 6. Literal-palette and Junie-specific colour assumptions inside widgets

### 6.1 Literal colours — clean

**[F]** No file under `src/widgets/`, `src/ui/`, or `src/core/` contains a `Color::Rgb(...)` construction or a hex colour constant. Every literal is confined to `theme.rs:59-95` (private `mod palette`). The README's rule (`README.md:204`, `README.md:307`) holds today.

### 6.2 Junie-specific *structural* assumptions — not clean

| Assumption | Sites |
|---|---|
| **Plane arithmetic by colour equality.** `Theme::lift` (`theme.rs:362-372`) and `Theme::backdrop` (`theme.rs:297-321`) branch on `bg == self.canvas`, `== self.surface`, `== self.field`, … A theme where two roles share a value, or a caller passing any other colour, silently lands on `popover`/`surface_overlay`. Every hover in the library goes through `lift`. | `theme.rs:362-372`, `:297-321`; callers: `chips.rs:163`, `grid.rs:1574,1641`, `list.rs` (via `row`), `menu.rs:315,570`, `progress.rs:289`, `segments.rs:106`, `statusbar.rs:282`, `tabs.rs:359-363,422`, `tree.rs`, `table.rs` (via `row`) |
| **"Dark canvas" assumed as the reversed-text foreground.** `fg(t.canvas)` on a light fill only reads if `canvas` is dark. | `theme.rs:354-356` (pressed row), `:420` (pressed secondary button), `:433` (pressed subtle); `grid.rs:1832,1866`; `table.rs:751`; `viewport.rs:626`; `progress.rs:293` |
| **Accent-budget rules encoded in components.** "the only control that fills with the accent"; "one screen keeps one accent underline"; "green is never used: a quota is not a completion"; "the accent tint appears only on the row that also has keyboard focus". | `brand.rs:1-4`; `tabs.rs:63-65,430-434`; `progress.rs:61-62,156-157`; `theme.rs:340-342` |
| **Menu hue reserved outside the semantic set.** `highlight` (`#2f5aa8`) and `highlight_danger` (`#7a2a2a`) exist solely so menus do not use the accent (`theme.rs:109-114`). A custom theme must supply a "cool blue that does not compete" to keep menus legible. | `theme.rs:74-75,109-114,155-157`; `menu.rs:302-306` |
| **`error_soft` exists only for destructive menu rows at rest.** | `theme.rs:89-91,115-117,158`; `menu.rs:311,320` |
| **Every glyph and every spacing value is a widget literal**, because `Theme` carries no layout/glyph tokens even though `DESIGN.md:35-48` declares them. `▎` appears in 17 modules (`button.rs:143`, `chips.rs:198,225`, `choice.rs:68,182,269`, `code.rs:681`, `completion.rs:184`, `grid.rs:1684,1722`, `input.rs:338`, `list.rs:255`, `menu.rs:576`, `panel.rs:92`, `picker.rs:317,438`, `props.rs:223`, `select.rs:184,229`, `steps.rs:215`, `table.rs:684`, `textarea.rs:228`, `tree.rs:501`). Markers `›` / `✓`: `list.rs:257-258`, `table.rs:687`, `grid.rs:1729`, `select.rs:233`, `code.rs:700`, `choice.rs:69`, `steps.rs:222`. Rounded borders: `menu.rs:255`, `panel.rs:110`, `dialog.rs:383`, `picker.rs:276`, `ui/popup.rs:69`. Scrollbar glyphs: `scrollbar.rs:8-9`. Spinner: `progress.rs:8`. Progress track: `progress.rs:70-73,265-267`. | as listed |
| **`Theme` extended from a widget module.** `impl Theme { fn change_glyph(RowState) }` returns a `(&'static str, Color)` pair for database row states. | `grid.rs:2007-2018` |
| **Colour-capability downgrade is Junie-shaped.** `nearest_16` (`theme.rs:604-641`) has a special case `if g > 120 && b < 80 → Yellow` tuned to amber `#f59e09`; `Theme::for_level` re-lists all 30 fields by name in a macro (`theme.rs:189-223`), so a user-supplied theme cannot be downgraded through a generic path. | `theme.rs:572-641`, `:183-225` |

**[I]** Goal §15 "reusable components must not contain literal palette colors" is already satisfied; goal §15 "components [must not depend] directly on Junie-specific palette assumptions" is **not** — the dependency has moved from literals into token *identity* comparisons (`lift`, `backdrop`), reserved-hue tokens (`highlight`, `error_soft`), and hard-coded glyph/spacing constants.

---

## 7. Panic / underflow risks with tiny or empty rects

Goal §17: *"Components must behave safely when given empty or very small rectangles. They must not panic, underflow, write outside their area, or leave stale hit regions."*

### 7.1 Confirmed arithmetic underflow (debug-build panic; wrapping in release)

| Site | Expression | Trigger |
|---|---|---|
| `input.rs:433` | `area.width as usize - 2` | `area.width == 1` (non-empty, passes the `:271` guard) and `area.height >= 3`. `usize` subtraction → panic. |
| `input.rs:440` | `area.width as usize - 2` | same |
| `input.rs:418` | `field.right() - 2` | `self.error.is_some()` and `area.x + area.width < 2` (i.e. `x==0, width==1`) |
| `input.rs:412` | `(cursor_col - self.scroll) as u16` | the scroll-fixup block is guarded by `if w > 0` (`:362`), but the cursor placement at `:411-414` is **not**. With `inner.width == 0` and a stale non-zero `self.scroll`, `cursor_col < scroll` → underflow. `inner.width = field.width.saturating_sub(3 + trailing)` (`:340-345`), so any field narrower than 4 (or 6 with an error) reaches this. |
| `textarea.rs:299` | `body.right() - 2` | `self.error.is_some()` and `area.x + area.width < 2` |
| `grid.rs:1458` | `self.widths[i]` | `self.columns` (public, `:329`) grown without calling `set_rows`/`sample_widths` → index out of bounds |
| `grid.rs:505-506` | `rows[a][col]` in `apply_local_sort` | a row shorter than `columns.len()` passed to `set_rows` |
| `grid.rs:1766,1801` | `&self.columns[ci]`, `self.value(src, ci)` | ragged rows are tolerated by `value()` (`:428-435` falls back to `Null`) but `self.columns[ci]` assumes `col_rects` and `columns` agree |
| `table.rs:700` | `&self.rows[src][ci]` | ragged rows (`pub rows`, `:99`) — no guard anywhere |
| `table.rs:220-221` | `rows[a][col]` in `apply_sort` | same |
| `table.rs:294` | `self.rows[src][col].text.clone()` in `begin_edit` | same |
| `table.rs:317-318` | `self.rows[src][e.col]` in `commit_edit` | same — and this path is reachable **from render** (`:566-568`) |

### 7.2 Early return before geometry is recorded → stale hit regions

Each of these returns without updating `self.area`/`self.col_rects`, so the *previous* frame's rectangles remain and subsequent `on_scrollbar`/`on_drag`/`on_click_cell` calls compute against stale coordinates:

`grid.rs:1512` (`area.is_empty() || area.height < 2`); `table.rs:559` (same); `code.rs:604`; `list.rs:210`; `tree.rs:463`; `props.rs:196`; `steps.rs:189`; `viewport.rs:583`; `panel.rs:267`; `textarea.rs:192` and `:218` (`rows == 0`); `input.rs:272` and `:331` (`area.height < 2` — returns *after* the label is drawn but *before* `self.area` is set); `select.rs:156` and `:175`; `chips.rs:151`; `choice.rs:57,156,259`; `button.rs:113`; `dialog.rs:389` (`inner.is_empty()` — returns after `self.area = area` at `:377`, so the dialog rect is recorded but no actions are); `picker.rs:282`.

Worst case [I]: `dialog.rs:389` returns after `ctx.begin_modal()` (`:373`) and after registering nothing — the frame is fully modal with **zero reachable focus stops and zero hit regions**, so `Focus::ensure_valid` (`focus.rs:94`) sets focus to `None` and the dialog can only be dismissed by Esc reaching the app's own handler.

### 7.3 Writes outside the area (clipped by `Buffer`, but geometrically wrong)

- `grid.rs:1637` — `let x = cols_area.right() + 1;` then `buf.set_string(x, …)` and `ctx.clickable(right_id, Rect::new(x, …))`. When `cols_area` ends at the buffer edge, the hit region is registered outside the grid.
- `table.rs:639` — `cols_area.right() + 1` for the right-overflow `…`.
- `panel.rs:117-122` — framed inner is `Rect::new(inner.x + 2, inner.y, inner.width.saturating_sub(3), inner.height)`. For `area.width <= 4` the `x` moves past `area.right()` while width saturates to 0 — the returned "inner" is outside the panel. Children then draw wherever that lands.
- `keyhint.rs:80-88` — guarded by `if area.width > w + 2` `:80`, correct.
- `chips.rs:193` — draws `…` at `x` which is already `> area.right()` in the overflow case (the check at `:192` is `x + w > area.right()`, but `x` itself can equal `area.right()`).

### 7.4 Correctly guarded (verified, for the record)

`scroll.rs:93-102` (`track_len == 0`, non-overflow); `scroll.rs:108-115`; `ui/popup.rs:26-27` (`.max(1)`); `ui/text.rs:12-18` (`max == 0`); `ui/text.rs:95` (`w.max(1)`); `progress.rs:54-58`, `:252-260`, `:278-286` (narrow fallbacks); `hit.rs:31,42` (empty rects rejected); `layout.rs:83-92` (`usable == 0`); `menu.rs:239`; `statusbar.rs:266`; `grid.rs:1776-1791` and `table.rs:724-738` (`cur - off` is safe because `off = cur.saturating_sub(cw-1)`).

---

## 8. Existing test inventory

### 8.1 Library unit tests (in-module `#[cfg(test)]`)

**`src/core/`** — 4 of 6 modules tested.
- `event.rs:137-164` — `outcome_combines_with_changed_dominating`, `key_helpers`.
- `focus.rs:101-152` — `tab_cycles_forward_and_backward`, `barrier_traps_focus_and_restores`, `ensure_valid_falls_back_to_first`.
- `hit.rs:100-136` — `topmost_wins`, `barrier_shadows_lower_regions`, `scroll_only_regions_ignore_hover`.
- `id.rs:48-60` — `ids_are_stable_and_distinct`.
- `scroll.rs:118-169` — `clamps_offset_to_content`, `ensure_visible_moves_minimally`, `thumb_covers_track_proportionally`, `track_position_round_trips`.
- `text.rs:440-526` — `insert_and_move_by_grapheme`, `selection_replaces_on_insert`, `word_motion_and_deletion`, `multiline_vertical_motion_keeps_column`, `single_line_rejects_newline`, `wide_characters_count_as_two_columns`.

**`src/ui/`** — 3 of 4 modules tested.
- `layout.rs:151-188` — `splits_respect_minimums_and_maximize`, `drag_moves_the_seam_and_respects_minima`.
- `popup.rs:80-109` — `places_below_then_flips_then_clamps`, `centers_in_upper_third`.
- `text.rs:174-203` — `truncates_with_ellipsis`, `middle_truncation_and_thousands`, `wraps_words_and_hard_wraps_long_tokens`.
- **`ui/ctx.rs` has no tests** — `Interaction::focused/hovered/pressed`, `RenderCtx::control/clickable/scrollable`, `begin_modal` and `fill` are untested.

**`src/theme.rs:643-698`** — `accent_survives_downgrade`, `hover_and_focus_are_distinct_styles`, `disabled_button_ignores_hover`. No test for `lift`, `backdrop`, `row` selection/error/busy paths, `field_style`, `gutter`, `syntax`, `tone`, `nearest_256`, `Mono`.

**`src/runtime.rs`** — **no tests** (the event loop, coalescing, tick scheduling and terminal restoration are unverified).

**`src/widgets/` — 10 of 31 modules have tests; 21 have none.**

| Module | Tests | Coverage |
|---|---|---|
| `brand` | `:89-132` (2) | padding/width/accent/bold; clickable hover + hit registration |
| `diff` | `:496-593` (4) | counts/headers; unified markers; review pairing + emphasis + hunk separator; render + wheel + "render must not undo the wheel" + mode toggle |
| `grid` | `:2020-2191` (9) | dirty-revert clears; delete+undo; insert+undo; edit validation by kind; key nav + sort requests + local sort keyed by source row; range selection + copy; position-label variants; fetch-more row; commit-result folding |
| `hintbar` | `:69-114` (2) | topmost layer wins + empty fallback; narrow rows drop right and mark `…` |
| `list` | `:306-356` (1) | wheel moves viewport, render preserves it, keyboard pulls the cursor back, clamp |
| `menu` | `:605-749` (5) | keyboard skips disabled/wraps/chooses; placement clamps + flips; click selects/dismisses + danger tone + shortcut; hover moves the cursor; menu bar open/switch/choose/toggle/brand/Esc |
| `picker` | `:529-621` (3) | wheel scrolls and survives re-render; keyboard pulls the cursor into view; boundary wheel is `Consumed` |
| `progress` | `:392-498` (4) | level thresholds; line-mode run colours; block-mode fill; each domain marker state |
| `statusbar` | `:308-411` (3) | group ordering/sides; narrow-row drop order keeping the truncated name; render fills the plane + hover registration |
| `steps` | `:298-317` (1) | frontier and counts |
| `table` | `:858-956` (6) | sort cycles asc/desc/none; numeric sort; cursor keeps its source row; edit commit/cancel; validation blocks commit; Tab to next editable cell + leave |
| `tabs` | `:494-600` (3) | active plane + only-accent underline + no gutter; hover vs cursor vs active planes; suffix glyph |
| `tree` | `:583-630` (1) | wheel moves viewport, render preserves it, keyboard pulls the cursor into view |
| `viewport` | `:674-744` (3) | tail follow + wheel leaves + End restores; drag select + copy + word select; wrap + bounded retention |

**Untested widget modules (17):** `button`, `chips`, `choice`, `code`, `completion`, `dialog`, `empty`, `field_common`, `input`, `keyhint`, `panel`, `props`, `scrollbar`, `segments`, `select`, `splitter`, `textarea`.

**[I]** The gap is concentrated exactly on the components with the worst findings: `dialog` (closed enum, render-time arming, `&mut Focus`), `input` (secrets, render-time validation), `select` (render closes the overlay), `code` and `textarea` (render commits), `chips` (drops out of the focus ring on overflow), `field_common` (the shared keymap).

### 8.2 Application integration tests

All three live **inside the binaries** as `#[cfg(test)] mod app_tests`, not under `tests/`. Each drives the real `App` through a `TestBackend` so the hit registry and focus ring are the production ones.

- **`src/bin/showcase/app_tests.rs`** — harness at `:14-130` (`draw`, `key`, `key_mod`, `type_str`, `mouse`, `click`, `text`, `row`, `find_row`, `find` (grapheme-accurate), `focus_bar_x`, `count`, `focus_area`). Tests: `launches_and_renders_shell` `:137`; `every_page_renders_at_representative_sizes_without_panic` `:147` (six sizes × every `NAV_ENTRIES` page, walking the focus ring twice); `below_minimum_size_shows_reduced_state` `:171`; `resize_recovers_from_too_small` `:180`; `tab_traversal_is_deterministic_and_wraps` `:189`; `disabled_buttons_are_skipped_and_cannot_activate` `:220`; `mouse_click_activates_and_keyboard_enter_activates` `:234`; `hover_and_focus_render_differently` `:245`; `hit_testing_prefers_rows_over_their_container` `:266`; `table_sorts_both_directions_and_clears` `:278`; `header_click_sorts` `:308`; `editable_table_commit_cancel_and_validation` `:322`; `input_editing_commit_and_revert` `:359`; `textarea_scrolls_with_wheel_and_keys` `:387`; `list_scrolling_and_selection` `:404`; `tree_expand_collapse_and_focus_bar_column_is_stable` `:431`; `modal_traps_focus_and_restores_it` `:453`; `prompt_dialog_validates_and_returns_value` `:479`; `form_validation_blocks_submit_and_focuses_first_error` `:499`; `scrollbar_click_and_drag_move_the_view` `:514`; `keyboard_navigation_between_pages` `:535`; `quit_keys` `:551`; `settings_screen_remove_member_flow` `:565`; `task_runner_animates_and_can_be_cancelled` `:586`; `color_downgrade_still_renders` `:607`; `showcase_visual_baseline` `:624`.
- **`src/bin/tablepro/app_tests.rs`** — harness `H` at `:15-40+`, with `H::new(w,h)` and `H::connected(w,h)` (connects to the "Production" fixture `:28-39`). Imports `Modal`, `Screen`, `WorkTab` `:12-13`.
- **`src/bin/jackin_preview/app_tests.rs`** — harness `H` at `:14-60+`, constructed as `H::new(scenario, motion, frame, w, h)` via `App::for_scenario` `:21`, with `key`, `ctrl`, `type_str`, `ticks(n)` (virtual clock `:51-56`) and `mouse`. Determinism is the contract (`README.md:57-62`).

### 8.3 Baseline / snapshot tests and fixtures

- **The only snapshot test is `showcase_visual_baseline`** (`src/bin/showcase/app_tests.rs:624`). It is a **digest**, not a golden image: for each of `(120,40)` and `(80,24)` × every `NAV_ENTRIES` page, it focuses the first control (`:631`), FNV-1a-hashes `symbol|fg|bg|modifier` for every cell (`:635-653`), **excluding the navigation sidebar** (`sidebar_area()`, `:633`,`:639`), and writes one line `"{w}x{h} {label} {hash:016x}"` (`:654`).
- **Fixture location:** `concat!(env!("CARGO_MANIFEST_DIR"), "/tests/showcase_baseline.txt")` — `app_tests.rs:657`. Regenerated with `UPDATE_BASELINE=1` (`:658-659`), documented at `README.md:325-328`.
- `tests/` is a directory that holds **fixtures only** — it contains no `.rs` integration-test targets; every test is an in-crate `#[cfg(test)]` module.
- **No baseline exists for TablePro or Jackin.** Their visual evidence is the manually produced PNG captures in `shots/` (`t_*`, `j_*`, `f_*`, `s_*`) generated by `tools/capture.sh` + `tools/ansi2png.py` (`README.md:316-323`) — reviewed by eye, not asserted by any test.

### 8.4 What the current suite does **not** prove

**[I]** Mapped against goal §25:
- No conformance/contract tests of any kind (§25.2) — nothing asserts "rendering does not commit an edit", which is precisely why the eight §4 violations are all green today.
- No test renders twice with unchanged inputs and asserts semantic stability, except three ad-hoc scroll cases (`list.rs:344`, `tree.rs:620`, `picker.rs:585`, `diff.rs:584`).
- No no-colour/monochrome assertions; the only downgrade test is `color_downgrade_still_renders` (`app_tests.rs:607`) at `Ansi16`, checking that *some* cell is `LightGreen`. `ColorLevel::Mono` is never rendered.
- No custom-theme test — every test constructs `Theme::junie()` or `Theme::for_level(...)`.
- No empty/tiny-rect fuzz. `every_page_renders_at_representative_sizes_without_panic` (`:147`) starts at 72×20, which is the documented minimum; nothing renders a widget into a 1×1 or 0-width rect, so every §7 finding is untested.
- No secret-redaction test.
- No overlay-stacking or nested-overlay test (`modal_traps_focus_and_restores_it` `:453` covers one level only).
- No identity-stability test under reorder/insert/remove.
- No architecture checks (§25.5) and no performance measurements (§25.6).
- No CI configuration was found in the repository root.

---

## 9. Summary of decisions this audit forces

**[I]** Ranked by how much of the goal is blocked:

1. **Split the event/action model.** `Outcome` must stop carrying "what happened"; one semantic-action channel replaces 6 return shapes, `bool`, and `Dialog`'s polled `result` field. Blocks §9.3, §12, §23-A.
2. **Introduce a focus-transition lifecycle.** Without it the eight §4 render-time commits cannot be removed, only relocated. Blocks §11, §19, §25.2.
3. **Make identity path-based, not index-based.** `WidgetId::child(usize)` keyed on display position is the single root cause of Scenario E's failure and of the `owns`/`locate` proliferation. `tree`'s `Path`-keyed events (`tree.rs:99-104`) are the existing proof it can be done. Blocks §12, §18.
4. **Give `RenderCtx` a surface stack.** 26 `bg: Color` parameters and `Theme::lift`'s equality dispatch both disappear once background is contextual. Blocks §15's "avoid forcing callers to pass raw background colors".
5. **Replace the flat `Theme` with tokens + recipes + parts + scoped patches**, and move spacing/glyph/density out of widget literals into design tokens (`DESIGN.md:35-48` already specifies the values). Blocks §15 scenarios 3–9 and §16 entirely.
6. **Support multiple barriers / a real overlay stack.** One `Option<usize>` in each of `FocusRing` (`focus.rs:12`) and `HitRegistry` (`hit.rs:26`) makes Scenario F impossible. Blocks §14, §23-F.
7. **Separate the generic grid from TablePro's database model**, and merge `DataTable` into it. Blocks §18, §23-H.
8. **Replace `fn`-pointer extension points with closures/traits** (6 sites, §3.12) and add borrowed data + custom row/cell renderers to every collection. Blocks §18, §23-D, §23-G.
9. **Add a secret type and manual `Debug` impls**; re-specify or remove `reveal_tail`. Blocks §19, §29.
10. **Rebuild `Dialog` on open composition**, delete `DialogBody`, and stop passing `&mut Focus` into handlers. Blocks §14 explicitly.

**Open work not covered by this pass:** the goal §7 application-side checklist — direct `Focus`/`FocusRing` and `HitRegistry` use, `.owns(...)`/`.locate(...)` chains, manually derived child ids, manual pressed/hover routing, raw Ratatui styles, hand-built dialogs/menus/tabs/forms/sidebars/status bars/scrollbars, duplicated key handling, and reusable interaction logic embedded in screens — across `src/bin/showcase/`, `src/bin/tablepro/` and `src/bin/jackin_preview/`, plus the "application-specific copies or variants" column of the §2 inventory.

**Relevant paths**
- `/Users/donbeave/Projects/terminal-components-claude/src/core/{event,focus,hit,id,scroll,text}.rs`
- `/Users/donbeave/Projects/terminal-components-claude/src/ui/{ctx,layout,popup,text}.rs`
- `/Users/donbeave/Projects/terminal-components-claude/src/{theme,runtime,lib}.rs`
- `/Users/donbeave/Projects/terminal-components-claude/src/widgets/` (31 modules)
- `/Users/donbeave/Projects/terminal-components-claude/src/bin/showcase/app_tests.rs`
- `/Users/donbeave/Projects/terminal-components-claude/src/bin/tablepro/app_tests.rs`
- `/Users/donbeave/Projects/terminal-components-claude/src/bin/jackin_preview/app_tests.rs`
- `/Users/donbeave/Projects/terminal-components-claude/tests/showcase_baseline.txt`
- `/Users/donbeave/Projects/terminal-components-claude/{README,DESIGN,REFACTORING_GOAL}.md`
- `/Users/donbeave/Projects/terminal-components-claude/Cargo.toml` (single package `junie-tui`, edition 2024, MSRV 1.88, deps: ratatui 0.30 + unicode-width 0.2 + unicode-segmentation 1)
