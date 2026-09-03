ected` `:92` | owned `Vec<String>` `:91` |
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

## 5. Secret / masked-input findings

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
13. `README.md:60-62` claims *"No secret ever reaches a frame — … plain-text keys live in transient edit state and render masked with a synthetic four-character tail."* The **synthesis** is an application behaviour; the library's `reveal_tail` reveals the *real* last N graphemes (`input.rs:135-143`). The safety property therefore lives in `src/bin/jackin_preview`, not in the reusable layer, and any other consumer of `TextInput::masked().reveal_tail(4)` leaks real characters.

**Inference**

- The library has **no** secret type, no redaction, no `Debug` suppression, and no zeroization. Every exposure path goal §19 lists — "rendering, captures, logs, cloning, or `Debug`" — is open.
- Minimum fixes: a `Secret<String>`-style newtype with a manual `Debug` writing `"[redacted]"`; a manual `Debug` impl on `TextInput`, `Dialog`, `AckInput`, `EditState` (both), `TextBuffer`; removal or re-specification of `reveal_tail` so the library synthesises the tail rather than revealing it; an explicit `expose()`/`with_value()` accessor instead of `text()`; and a conformance test asserting `format!("{:?}", masked_input)` contains neither the value nor the snapshot.

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
