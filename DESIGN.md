---
version: alpha
name: Junie TUI
description: Canonical terminal-native design system for junie-tui, extracted from the approved Ratatui implementation.
omitted:
  - section: typography
    reason: Ratatui does not control terminal font family, font size, or line height; semantic text roles and modifiers are specified in Markdown.
  - section: rounded
    reason: Terminals have no radius dimension; rounded Unicode border glyphs are specified in Markdown.
colors:
  primary: "#48e054"
  canvas: "#000000"
  surface: "#111111"
  surface-elevated: "#18181b"
  field: "#1e1e22"
  field-hover: "#232328"
  surface-overlay: "#27272a"
  popover: "#3f3f46"
  border-subtle: "#262626"
  border-strong: "#4d4d4d"
  text-primary: "#ffffff"
  text-secondary: "#b3b3b3"
  text-muted: "#808080"
  text-faint: "#4d4d4d"
  text-ghost: "#262626"
  text-on-accent: "#19191c"
  accent: "{colors.primary}"
  accent-hover: "#3ab343"
  accent-pressed: "#2b8632"
  accent-selection: "#0f2e13"
  focus: "{colors.primary}"
  disabled: "{colors.text-faint}"
  error: "#e44545"
  warning: "#f59e09"
  success: "{colors.primary}"
spacing:
  none: 0
  inline: 1
  control: 2
  dialog-horizontal: 3
  form-columns: 4
components:
  row-default:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.text-primary}"
  row-hover-on-canvas:
    backgroundColor: "{colors.surface-elevated}"
    textColor: "{colors.text-primary}"
  row-selected-focused:
    backgroundColor: "{colors.accent-selection}"
    textColor: "{colors.text-primary}"
  row-pressed:
    backgroundColor: "{colors.text-primary}"
    textColor: "{colors.canvas}"
  row-disabled:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.disabled}"
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.text-on-accent}"
  button-primary-hover:
    backgroundColor: "{colors.accent-hover}"
    textColor: "{colors.text-on-accent}"
  button-primary-pressed:
    backgroundColor: "{colors.accent-pressed}"
    textColor: "{colors.text-on-accent}"
  button-secondary:
    backgroundColor: "{colors.surface-overlay}"
    textColor: "{colors.text-primary}"
  button-secondary-hover:
    backgroundColor: "{colors.popover}"
    textColor: "{colors.text-primary}"
  button-danger:
    backgroundColor: "{colors.surface-overlay}"
    textColor: "{colors.error}"
  field-default:
    backgroundColor: "{colors.field}"
    textColor: "{colors.text-primary}"
  field-hover:
    backgroundColor: "{colors.field-hover}"
    textColor: "{colors.text-primary}"
  field-placeholder:
    backgroundColor: "{colors.field}"
    textColor: "{colors.text-muted}"
  panel-card:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-primary}"
  frame-border-unfocused:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.border-subtle}"
  frame-border-focused:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.border-strong}"
  dialog:
    backgroundColor: "{colors.surface-elevated}"
    textColor: "{colors.text-primary}"
  scrollbar-track:
    textColor: "{colors.border-subtle}"
  scrollbar-thumb:
    textColor: "{colors.text-muted}"
  scrollbar-thumb-hover:
    textColor: "{colors.text-secondary}"
  scrollbar-thumb-focused:
    textColor: "{colors.text-primary}"
  modal-backdrop-ghost:
    textColor: "{colors.text-ghost}"
  validation-error:
    textColor: "{colors.error}"
  warning-status:
    textColor: "{colors.warning}"
  success-status:
    textColor: "{colors.success}"
  focus-gutter:
    textColor: "{colors.focus}"
---

# Junie TUI Design System

This file is normative for new screens and reusable components. It records the approved implementation; it does not propose a redesign. When prose and code differ, rendered behavior in `src/theme.rs`, `src/widgets/`, `src/ui/`, and the application shells is authoritative. Update this file with any intentional design change.

Format follows Google's current DESIGN.md `alpha` model: normative YAML tokens plus operational rationale in canonical section order. Terminal-only concepts remain Markdown because browser dimensions, shadows, and radii would be false abstractions.

## Overview

### Identity

Junie TUI translates Junie's visual language into a modern character grid. Atmosphere: calm, precise, professional, focused, technical, restrained, contemporary. Near-black planes carry crisp white content; a disciplined white-opacity ladder creates hierarchy; one deliberate green identifies focus, primary action, current choice, edit mode, or live completion. Density can be high, but chrome stays quiet.

System character comes from five constraints:

1. **One interaction accent.** Green is accent, never general paint. Do not use it for body copy, ordinary metadata, every heading, large ambient fills, or arbitrary status categories.
2. **State uses geometry plus tone.** Focus uses a gutter, weight, or stronger frame; current/selected uses component-specific marker, shape, weight, or tint; hover lifts one neutral plane; editing exposes hardware cursor plus underline; errors add `!` and a message. Color alone should not carry required meaning.
3. **Three working planes.** Canvas, surface, elevated surface. Overlay, field, and popover are targeted state/overlay planes, not a license to add cards everywhere.
4. **Whitespace is structure.** Empty rows, two-cell insets, and restrained gaps establish groups before borders do.
5. **Borders carry structure.** Frames identify panes and overlays. They are not default decoration.

Primary truecolor target uses modern terminal mouse support. ANSI-256, ANSI-16, and monochrome fallbacks preserve semantics through glyphs, modifiers, and relative lightness.

### Source and extension boundary

- `src/theme.rs`: raw palette, semantic tokens, text roles, state resolvers, color fallback.
- `src/core/`: widget IDs, deterministic focus ring, hit registry, text buffer, scroll state, event model.
- `src/runtime.rs`: terminal lifecycle, mouse/paste capture, tick scheduling, and the application event loop.
- `src/ui/`: render context, interaction snapshot, split layout, text fitting, popup placement.
- `src/widgets/`: reusable components. Primitives remain domain-neutral; `DataGrid` is the explicit database-domain exception.
- `src/bin/showcase/`: approved laboratory, composed examples, responsive shell, interaction baseline.
- `src/bin/tablepro/`: evidence for dense workspaces and database workflows, not source of application-specific design tokens.
- `tests/showcase_baseline.txt`: visual digest of every showcase page at `120×40` and `80×24`.
- `shots/`: headless captures for human visual review.

### Design composition order

For a new screen:

1. State primary task and one primary working area.
2. Place identity/context in compact segment strip; keep action hints contextual.
3. Group related content with whitespace. Add card only for contained content; add frame only for a real pane boundary.
4. Register focus stops in visual reading order. Define current, selection, editing, dirty, loading, disabled, and error separately.
5. Allocate responsive priority before shrinking: remove optional metadata, stack/collapse secondary regions, turn navigation into drawer, then truncate.
6. Add mouse hit regions after container regions so specific rows/cells win.
7. Validate at `72×20`, `80×24`, `100×30`, `120×40`, and `160×50`; compare to approved frames.

## Colors

### Canonical truecolor tokens

YAML values are normative. This table defines role, use, and prohibition.

| Token | Exact RGB | Role and approved use | Do not use for |
|---|---:|---|---|
| `canvas` | `#000000` | Application foundation; ambient page and unfilled pane background. | Hover, raised content, modal body. |
| `surface` | `#111111` | Chrome and filled card plane. | Every pane; selected rows. |
| `surface-elevated` | `#18181b` | Cards above canvas; dialogs/popups; canvas-hover lift. | General canvas replacement. |
| `field` | `#1e1e22` | Text-entry body in default/focused/editing state. | Card or row background. |
| `field-hover` | `#232328` | Field hover only when not editing. | Focus or editing indicator. |
| `surface-overlay` | `#27272a` | Secondary/toggle button fill; lift for rows on surface/elevated; neutralized colored fills behind modal. | Default card plane. |
| `popover` | `#3f3f46` | Strongest neutral lift; text selection and range-selection background; secondary-button hover. | Large regions or normal rows. |
| `border-subtle` | `#262626` | Unfocused rounded frame, separator, progress track, scrollbar track. | Focus or text. |
| `border-strong` | `#4d4d4d` | Focused frame and quiet underline. Same RGB as `text-faint`. | Accent substitute. |
| `text-primary` | `#ffffff` | Main content, headings, focused labels, key names. | Low-priority metadata. |
| `text-secondary` | `#b3b3b3` | Supporting copy, active progress, chosen markers outside focus. | Disabled or ghosted content. |
| `text-muted` | `#808080` | Metadata, placeholders, helper text, key actions, idle scrollbar thumb. | Required primary values. |
| `text-faint` | `#4d4d4d` | Lowest live text tier and disabled content. | Body copy or safety warnings. |
| `text-ghost` | `#262626` | Deepest modal-backdrop tier only. | Live content. |
| `text-on-accent` | `#19191c` | Primary-button and `EDIT` badge text. | Text on neutral surfaces. |
| `accent`, `primary`, `focus`, `success` | `#48e054` | Focus gutter, primary action, chosen marker at focus, active tab underline, edit badge, spinner/indeterminate sweep, completed progress. | Body copy, generic backgrounds, all statuses, decoration. |
| `accent-hover` | `#3ab343` | Primary-button hover background only. | Neutral hover. |
| `accent-pressed` | `#2b8632` | Primary-button pressed background only. | Selection or success. |
| `accent-selection` | `#0f2e13` | Focused selected row tint before hover precedence. | Standalone focus or broad fill. |
| `error` | `#e44545` | Invalid fields/cells, error glyph/message, danger pressed fill, failed progress. | Routine destructive action at rest beyond danger text. |
| `warning` | `#f59e09` | Dirty values, pending changes, warnings, production-sensitive status, paused/attention semantics where implemented. | Every notice. |

### Defined but noncanonical palette entries

Do not promote implementation reserves without a demonstrated recurring role:

- `accent_bg_subtle` / `#0a1c0c` and `error_bg` / `#2e0f0f` are constructed and downgraded but unused by current styles.
- `info` / `#8787ff` appears in the overview swatch only and is absent from semantic `Tone`; it is not an approved general accent.

### Color relationships and fallback

- YAML names describe design roles, not Rust method names: YAML `primary`/`accent` map to `Theme.accent`; YAML `accent-selection` maps to `Theme.accent_bg`. `Theme::primary()` is the white primary-text style, not the green YAML token.
- White hierarchy is explicit sRGB, not terminal alpha: 100% `#ffffff`, 70% `#b3b3b3`, 50% `#808080`, 30% `#4d4d4d`, 15% `#262626`.
- `border-strong`, `text-faint`, and `disabled` intentionally alias `#4d4d4d`. `border-subtle` and `text-ghost` alias `#262626`. `accent`, `focus`, and `success` alias `#48e054`.
- Hover lifts exactly one neutral plane: `canvas` to `surface-elevated`; `surface` or `surface-elevated` to `surface-overlay`; `field` to `field-hover`; other backgrounds to `popover`.
- Error and warning are safety/status hues. Green remains interaction/completion accent; phrase “one hue” means one general interaction accent, not absence of semantic risk hues.
- `NO_COLOR` forces monochrome. `COLORTERM=truecolor|24bit` selects truecolor. `TERM` containing `256color`, `ghostty`, or `kitty` selects ANSI-256; otherwise ANSI-16.
- Downgrade applies to every semantic token. ANSI-256 selects nearest color cube/gray. ANSI-16 selects named colors by luminance/hue; tested anchors are accent `LightGreen`, error `LightRed`, canvas `Black`. Mono bins average RGB into `Black`, `DarkGray`, `Gray`, or `White`.

## Typography

Terminal owns font family, size, cell metrics, ligatures, and exact weight rendering. Require a legible monospace terminal; never encode a browser font, pixel size, line height, or letter spacing in component design.

### Semantic text roles

| Role | Foreground | Modifier | Placement and use |
|---|---|---|---|
| Primary content/value | `text-primary` | none | Main data and normal interactive labels. |
| Heading/title | `text-primary` | `BOLD` | Page/panel/dialog title; one concise line. |
| Focused field label | `text-primary` | `BOLD` | Label for keyboard destination. |
| Unfocused label | `text-secondary` | none | Form or group label. |
| Secondary text | `text-secondary` | none | Supporting prose, result details. |
| Metadata/helper/action hint | `text-muted` | none | Counts, timestamps, descriptions, footer action words. |
| Faint/disabled | `text-faint` | none | Lowest live tier or disabled state. |
| Keyboard key | `text-primary` | `BOLD` | Key half of contextual hint. |
| Placeholder | `text-muted` | none | Empty field only; disabled becomes `disabled`. |
| Error | `error` | message; `BOLD` on `!` | Validation and failed state. |
| Warning/dirty | `warning` | underline or glyph | Pending/dirty/risky context. |
| Code keyword | `text-primary` | `BOLD` | Syntax structure. |
| Code identifier/plain | `text-primary` | none | Primary code. |
| Code string/number | `text-secondary` | none | Secondary syntax. |
| Code operator/punctuation | `text-muted` | none | Syntax scaffolding. |
| Code comment | `text-faint` | `ITALIC` | Commentary. |
| Editing | inherited | `UNDERLINED` plus hardware cursor | Editable graphemes/cell. |

Rules:

- Bold identifies focus or hierarchy. Do not bold every row, heading-like label, or status.
- Sentence/title case only. Do not introduce all-uppercase headings.
- Align numeric table cells right; textual cells left. Keep row height one cell.
- Truncate by Unicode display width. Use `…`; never slice bytes or assume one code point equals one cell.
- Hide low-priority metadata before starving a primary label; list metadata disappears when label would receive fewer than 12 cells.
- Do not use dim gray so low it becomes unreadable for live content. `text-ghost` belongs only behind a modal.
- Do not make every label green. Required field `*` may use accent because it is a compact semantic marker.

## Layout

All measurements are terminal cells. Values describe current implementation, not an abstract scale.

### Spacing and shell rhythm

| Pattern | Exact cells | Rule |
|---|---:|---|
| Inline separator/control gap | 1 | Dialog action gap, tab gap, split divider. |
| Normal component/pane gap | 2 | Button groups, ordinary columns, shell/sidebar separation. |
| Form column gap | 4 | Dense paired form columns. |
| Section break | 1 row | Insert between related blocks; do not replace with a border. |
| Shell header/footer | 1 row each | Body begins at `y+2`, height `H-4`, preserving one blank row above and below. |
| Table/grid row/header | 1 row | Dense data stays scan-friendly without blank rows. |
| Card inset | `2×1` | Horizontal × vertical; titled card consumes title row before content. |
| Framed pane effective inset | left 3, right 2, vertical 1 | One-cell border inset plus two-cell content inset from left. |
| Dialog inner margin | `3×2` | Horizontal × vertical inside rounded frame. |

Whitespace is semantic. Do not remove the blank shell rows, title-to-content row, or section spacers merely to expose more records. For data density, reduce optional metadata or collapse secondary panes before compressing core component anatomy.

### Component dimensions

- Button: height `1`; width `display_width(label)+2`, or `+4` when busy/toggle marker exists. One cell at each side; left padding becomes `▎` on focus.
- `TextInput` and `Select`: height `3`: label, one-row body, helper/error. Input text begins `x+2`; body reserves three trailing cells and two extra for error.
- `TextArea`: height `rows+2`: label, requested body rows, helper/status footer. Body text inset two cells; last body column may be scrollbar.
- `DataTable`/`DataGrid`: minimum height `2`; one-row header; two-cell column gap. Basic table content starts at `x+3` and reserves five structural cells plus optional scrollbar.
- `Tabs`: height `2`: label row and baseline/active underline row. One-cell gap between tabs. Overflow reserves three columns at each side plus four for new-tab control.
- Dialog: requested width capped at screen width minus `4`, floor `20`; height capped at screen height minus `2`; base height is wrapped body plus `8` rows. Actions right-align three rows above bottom.
- Anchored select popup: width clamped `12..40`, height capped at `10`.
- Completion popup: content-derived width `max(label+detail+8)`, clamped `24..48`; height `visible rows+2`.

### Split and responsive rules

`Split` subtracts gap, applies configured percentage and minima, and clamps adjustable percentage to `10..90`. Insufficient vertical space gives all area to first pane; insufficient horizontal space gives all area to second pane. Do not assume split always yields two nonempty rectangles.

Global minimum for both binaries is `72×20`. Below minimum, replace application with centered “Terminal too small”, current dimensions, and required `72×20`; do not render clipped controls.

| Terminal | Observed approved shell behavior |
|---|---|
| `80×24` | Fully usable compact state. Showcase sidebar `19`, main `59`, body height `20`. TablePro explorer is a focus-driven full-body drawer; main returns when focus leaves. |
| `100×30` | Showcase sidebar `19`; optional inspector is `30`, leaving main `47`. TablePro still uses drawer because body width is below `100`. |
| `120×40` | Showcase sidebar `24`, main `94` without inspector; standard visual baseline. TablePro explorer/main are about `29/88` with one-cell gap. |
| `160×50` | Preserve density and negative space; TablePro explorer/main about `39/118`; do not stretch explorer past `40`. |

Application conformance evidence—not reusable component law:

- Showcase sidebar is `19` columns below terminal width `110`, `24` at `>=110`; main gap `2`. Optional inspector appears only at width `>=100`, fixed `30`, gap `2`. Short height removes section labels and gaps, leaving one compact contiguous item list.
- TablePro root uses one-cell horizontal outer margins. Workbench drawer condition is body width `<100`; therefore terminal width through `101` remains narrow, side-by-side begins at `102`. Wide explorer is `(body_width/4).clamp(28,40)`, gap `1`.
- Query editor/results split is `38%`, minima `4/6`, gap `1`; it may maximize either pane.
- Connections become list/detail at content width `>=80`, corresponding to terminal width `82`; list width is `(content_width/3).clamp(26,40)`, gap `2`.
- History list/detail is `50/50`, minima `30/30`, gap `2`; under total body width `62`, list collapses and detail owns width.
- Generic showcase two-column helper stacks into equal vertical halves when width `< left_width + gap + 20`.

Responsive design means information prioritization:

1. Retain identity, current task, focus, errors, and destructive context.
2. Drop lowest-priority segments and footer hints from right.
3. Hide metadata before primary labels.
4. Convert secondary navigation to drawer/overlay or collapse one split side.
5. Truncate with `…` and show horizontal overflow controls.
6. Never make essential controls overlap or shrink below component anatomy.

## Elevation & Depth

No shadows. Depth comes from tonal planes, modal backdrop remapping, and structural borders.

### Surface hierarchy

1. `canvas`: app foundation and whitespace.
2. `surface`: normal card/chrome.
3. `surface-elevated`: raised card, dialog, popup, canvas hover.
4. `surface-overlay`: neutral control fill and one-step lift from surface/elevated.
5. `popover`: strongest neutral selection/lift plane, used locally.

Use a background shift for transient hover or text selection. Use a border when a persistent spatial boundary matters: split pane, modal, popup. Use whitespace when proximity alone can group content. Never surround every region, card, nested element, or data subsection with a frame.

Modal backdrop preserves canvas/surface/elevated shapes; field fills become elevated, other colored fills become overlay. Primary/accent/error/warning foregrounds collapse to muted, secondary/on-accent to faint, remaining text to ghost. All modifiers clear. Dialog remains elevated with strong rounded frame. This keeps context legible but inert.

## Shapes

Terminal shape is glyph geometry, not pixel radius.

### Border system

- Card: filled `surface`, no border. Default container.
- Framed pane: `Borders::ALL`, Ratatui `BorderType::Rounded`; `border-subtle` when unfocused, `border-strong` when focused.
- Popup, picker, dialog: rounded all-side frame on `surface-elevated`; implementation uses focused/strong border.
- Table/list/tree/grid: no per-row boxes and no column walls.
- Tabs: `─` subtle baseline, `━` accent active underline.
- Progress: `─` subtle track, `━` filled segment.
- Scrollbar: `│` track, `┃` thumb, one column.

Use light/rounded Unicode only where it conveys boundary. Heavy glyphs show active progress/tab/thumb, not ornamental borders. Do not invent ASCII box grids, double borders, nested frames, CSS radii, or shadows.

### Glyph inventory

| Glyph | Meaning | Use constraints |
|---|---|---|
| `▎` | Keyboard focus gutter | Focused controls/rows and focused card titles; hidden foreground equals background otherwise. Framed panes use border/title strength instead. |
| `›` | Current/chosen item | List/table choice or current target; not persistent multi-selection. TreeView does not render this marker. |
| `✓` | Checked/selected/completed | Checkbox/multi-row selection/progress completion. |
| `!` | Error | Pair with error text/color; trailing status slot. |
| `•` | Dirty/modified | Warning tone in data grid/tab status. |
| `+` / `−` | Inserted/deleted | Data change slot; deleted row also faint and crossed out. |
| `▸` / `▾` | Collapsed/expanded | Tree nodes only. |
| `▴` / `▾` | Ascending/descending | Table header sort suffix. Context disambiguates from tree. |
| `∇` | Filter applied | Data-grid header suffix. |
| `⚷` | Primary-key column | Data-grid schema meaning only. |
| `→` | Referenced value | Cell trailing link/reference affordance. |
| `‹` / `›` | Hidden tabs | Clickable direction; current Tabs implementation does not show a count. |
| `‹n` / `n›` | Hidden grid columns | Clickable count/direction. |
| `…` | Truncation/hidden overflow | Only when content is omitted. |
| `⠋…⠏` | Ten-frame activity spinner | Active/busy state, accent. |
| `○` / `●` | Off/on | Toggle/radio markers; always paired with label/state. |
| `×` | Close | Tab/chip affordance, subtle until hover. |
| `↓` | Fetch more | Virtual data row with explanatory text. |
| `◆` | Production/environment identity | Contextual application identity, not decoration. |

Glyphs communicate. Do not add decorative Unicode, mascots, icon walls, or duplicate textual meaning without a state/accessibility reason.

## Components

### Canonical interaction-state model

States are independent semantic dimensions. Never collapse them:

`HOVERED != FOCUSED != ACTIVE/PRESSED != CURRENT != SELECTED != EDITING != DIRTY != DISABLED != ERROR != LOADING`

| State | Meaning | Visual treatment | Behavior | Combination/precedence |
|---|---|---|---|---|
| Default | Available, neither pointer nor keyboard target. | Base surface and normal text. | Accepts focus/hit according to component. | Lowest precedence. |
| Hovered | Pointer is over hit target. | Lift one neutral plane; editable cell may gain quiet underline. | Preview only; never moves keyboard focus by itself. | Hidden while disabled; suppressed after keyboard input until pointer moves; row hover overrides selected-focus tint but not focus gutter. |
| Focused | Next keyboard input goes here. | Accent `▎`, bold row/label, stronger containing frame/title. | Receives keyboard events. | Coexists with hover, current, selected, error, dirty. |
| Active/pressed | Activation is physically held or in 140 ms flash. | Primary darker green; danger white-on-red; supporting controls/rows commonly reverse to black-on-white. Tabs and DataGrid cells have no separate pressed visual. | Fires only when enabled and mouse-up matches press origin, or keyboard activates. | Where implemented, overrides normal row foreground/background. |
| Current | Navigation cursor/logical present item. | `▎`/bold cursor row or context-specific `›`; data cell reverses white-on-black. | Movement changes location, not persistent selection unless component explicitly couples them. | May differ from selected and hovered. |
| Selected/chosen | Persisted choice or set membership. | `›`, `✓`, `[✓]`, `(●)`, `●`, or component-specific chosen label tone; focused selected row may use `accent-selection`. | Activation changes model. | List/table markers become secondary outside focus. Checkbox/radio/toggle markers remain green outside focus while their glyph carries meaning. TreeView's selected label also remains green, but lacks a noncolor cue; see exception below. |
| Editing | Text input owns hardware cursor and mutates buffer. | Field plane, accent underline, hardware cursor, global `EDIT` badge. Multiline/code current-line underline may use `border-strong`. | Text bindings capture before navigation/global commands. | Keeps focus. Table/grid validation can replace underline with red; TextInput/TextArea keep normal edit underline and add error glyph/message. |
| Dirty | Value differs from committed source. | Warning `•` and/or warning underline; pending count/action bar. | Supports undo/preview/save/discard. | Error outranks dirty glyph; dirty underline may remain on error/current cell. |
| Disabled | Visible but unavailable. | `disabled`/`text-faint`; no focus gutter, hover, press, or activation. | Excluded from focus ring; mouse hit may consume without action. | Highest early exit for row styling. |
| Error | Invalid/failed state. | Error text, trailing bold `!`, message where surfaced; error cell may become white-on-red when current. | Blocks invalid commit where validator owns commit. | Coexists with focus/editing/dirty. Table/grid edit errors use red underline; TextInput/TextArea retain their normal edit underline. |
| Loading/busy | Work is active. | Accent spinner or indeterminate sweep; supporting text secondary; no press. | Busy button rejects activation; async owner updates on tick. | Busy row foreground overrides error in generic row resolver; domain widgets may prioritize running state over dirty/error tab marks. |

Generic row resolver precedence is exact: disabled returns immediately; selected+focused applies green tint; hover replaces that background with one-plane lift; error sets foreground; busy replaces foreground with secondary; focus adds bold; pressed replaces full style. Gutter is resolved separately and remains visible through compatible states.

### Focus language

- Focus answers one question: “Where will the next keyboard input go?” Use one focus cue per logical stop: usually `▎`; framed panes instead strengthen border/title.
- Focus ring rebuilds every frame in render/reading order and wraps forward/backward. Disabled controls do not register.
- Composite components—list, tree, table, grid, tabs, radio group, chip bar—are one Tab stop. Their internal cursor moves with arrows or `h/j/k/l` as appropriate.
- Focused containers strengthen frame/title; focused child control owns gutter. Do not add a second green box around child.
- Responsive hiding invalidates stale focus; `ensure_valid` moves to first reachable stop. Narrow TablePro explorer drawer is visible while it owns focus and closes when focus moves into main content.
- Opening a modal saves exact previous focus, clears hover/press, inserts focus/hit barriers, and chooses initial focus. Closing restores saved focus; next frame repairs invalid target.
- Confirm dialog starts on affirmative primary action. Destructive dialog starts on Cancel. Prompt starts on input. Typed acknowledgement starts on acknowledgement field and disables final action until exact trimmed token matches.
- Picker owns logical row/query cursor rather than normal focus stops; application temporarily sets focus to none, then restores saved focus on close.

### Hover language

- Hover is pointer preview, not keyboard destination. Never draw focus gutter because of hover.
- Buttons lift or change approved variant fill. Rows/lists/trees/tabs lift neutral surface. Fields use `field-hover` only outside editing. Sortable headers underline on hover. Scrollbar thumb brightens from muted to secondary.
- Keyboard input suppresses stale hover until pointer movement. This makes keyboard focus dominant after input-method switch.
- Disabled controls ignore hover. Error, editing, current cell, selection, and focus retain their stronger cues.
- Mouse hit registry rebuilds each frame; later registrations win. Register container, then rows, then cells/close affordances so most specific target wins.

### Current, selected, active, and editing

- **Current** is where navigation is. In a grid it is one reversed cell; in a list/tree it is cursor row; in a sidebar it may be current page. Moving current must not silently create persistent multi-selection.
- **Selected** is stored membership/choice. A different row may be hovered while one row is current and several rows are selected.
- **Active** is the displayed tab/view. Tab active is bold white with green `━`; tabstrip focus cursor remains `▎`. Implementation usually synchronizes tab cursor and active tab, but meanings remain distinct. Tabs do not render a separate transient pressed style.
- **Editing** begins only from focused navigation state using `Enter`, `F2`, typing where supported, or second click on already-focused/current editable target. Hardware cursor appears only during edit/search/picker query.
- Single-line input/cell: `Enter` commits, `Esc` restores edit snapshot, `Tab` commits and advances. Multiline text area/code editor: `Enter` inserts newline and `Esc` commits/leaves. Losing focus commits current editor.
- Shared editing keys: Shift+arrows extend; Ctrl/Alt+Left/Right move by word; Home/End line; Ctrl+Home/End document; Backspace/Delete; Ctrl/Alt+Backspace or Ctrl+W delete word; Ctrl+A/E line start/end; Ctrl+U/K delete to start/end; Ctrl+L select all; Alt+B/F word movement.
- Text selection uses `popover` background. Do not equate selected text with selected row.

### Keyboard design language

Application-wide conventions:

- `Tab` / `Shift+Tab`: next/previous focus stop, wrapping in reading order.
- Arrows and `h/j/k/l`: move internal cursor; `PageUp/PageDown`, `Home/End`, `g/G` provide viewport/start/end where meaningful.
- `Enter` / `Space`: activate or choose; `Enter` begins editing when current target is editable.
- `Esc`: cancel single-value editing, commit/leave multiline editing, dismiss overlay, or move outward from local mode.
- `[` / `]`: previous/next page or tab where app shell owns them.
- `y/n`: direct answer only in confirm dialogs and only with no Ctrl/Alt modifiers.
- Contextual keys such as sort/filter/run belong to owning component/screen and appear in footer. Suppress incompatible global chords while editing.
- Footer shows bold key plus muted action. Order most important first; drop pairs from right. Right status always wins; do not wrap into a shortcut wall.

### Mouse design language

- Pointer move updates hover only. Design rule: left down sets press origin and focuses the owning reachable stop; left up activates only the same target. Drag routes to press origin for selection/scrollbar. See the current composite-hit exception below.
- There is no native double-click semantic. Input/table/grid use second click on already-focused/current target to start edit.
- Wheel scrolls topmost container under pointer without moving focus; vertical step is `3` rows. TablePro can route horizontal wheel to grid/editor; Showcase ignores horizontal wheel.
- Scrollbar track/thumb supports click and drag. Column header click sorts. Tab/chip close affordances are separate topmost hit targets.
- Modal barrier prevents interaction below. Cancelable dialogs and TablePro modal pickers/filter close on outside click; noncancelable modal consumes it.
- Switching mouse to keyboard must not leave competing hover; switching keyboard to mouse must not silently change current/selection until click.

### Scrolling

Scrolling belongs to container, not scrollbar.

- Model tracks `offset`, `content_len`, `viewport_len`; clamps on all updates.
- Arrows/`j/k` move one; page keys move one viewport; start/end jump; cursor movement calls minimal `ensure_visible`.
- Scrollbar appears only when content length exceeds nonzero viewport. Thumb length is proportional, minimum one cell; click maps track position around thumb center.
- Track `│` uses `border-subtle`; thumb `┃` is muted, secondary on hover/press, primary when owner focused.
- Position label format is `12–24 of 120`. Dense tables may also show loaded/estimated rows and visible column range.
- Nested scrolling sends wheel to last-registered/topmost region only; there is no parent bubbling at child edge.
- `ScrollPanel` follow-tail mode: End/`G` enables follow; manual key, wheel, or thumb movement disables it; `f` toggles.
- Wide tables/grids preserve column widths and use horizontal viewport/overflow counts (`‹n`, `n›`) instead of compressing every column. Current item remains visible.

### Overlays and modal hierarchy

| Layer | Surface | Frame | Focus/event model | Dismissal |
|---|---|---|---|---|
| Completion | Elevated anchored popup | Rounded strong | Nonmodal; owner editor retains focus and forwards keys. | `Esc`, accept, owner close. |
| Select popup | Elevated anchored popup | Rounded strong | One owner focus stop; popup rows have logical cursor/hits. | `Esc`, choice, focus loss; outside-dismiss must be handled by app owner. |
| Picker/quick switcher | Elevated centered upper-third | Rounded strong; dim backdrop | Modal logical query/row cursor; page inert. | `Esc`, choice, outside click where app routes it. |
| Dialog | Elevated centered | Rounded strong; dim backdrop | Focus/hit barriers; Tab trapped; prior focus saved. | Cancel action, `Esc`, `n`, outside click if cancelable. |
| Destructive/ack dialog | Same as dialog | Same | Cancel or acknowledgement starts focused; final action disabled until armed. | Never implicit confirm. |

Anchored placement prefers below, flips above when needed, then clamps to screen. Pickers center horizontally and sit at one-third of remaining height, at least one row down; dialogs center on both axes. Popup surface inset is `1×1`; dialog inner margin is `3×2`. Render overlays last whenever composing siblings so their hit barrier stays topmost.

### Safety language

Visual severity must match action severity:

- Routine primary action: accent fill.
- Reversible warning/pending change: amber marker/underline and explicit count, not red flood.
- Destructive action at rest: red text on neutral overlay. On press: white on red. Destructive dialog starts on Cancel.
- High-risk action: facts list states connection/environment/database/table, scope, risk, reversibility, triggering safety level, and code; optional typed target acknowledgement gates action.
- Read-only refusal: explicit error message and no enabled write action. Disabled alone is insufficient explanation.
- Production identity: compact `◆ production`/amber safety context in identity strip; never paint entire app red.
- Error: red `!` plus message/location. Do not rely on red alone.

### Motion and temporal states

- Spinner sequence: `⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏`; frame is `tick % 10`.
- Showcase/TablePro request fast ticks around active work; TablePro uses `80 ms` active tick and `400 ms` idle cadence. Do not animate inactive decoration.
- Activation flash lasts `140 ms` after keyboard or mouse activation.
- Indeterminate progress uses accent `━` segment of one-fifth track, clamped `2..8` cells, sweeping across subtle `─` track.
- Determinate active fill is secondary white, completed fill green, error red, paused muted. Fixed trailing glyph column uses `✓`, `!`, or `‖`.
- Narrow determinate track below six cells degrades to percentage only.

### Non-happy paths

- Empty: centered muted title, optional faint wrapped hint after one blank row, no large icon. Empty is not error.
- Loading: spinner/indeterminate motion plus action label; not faint/disabled styling.
- Error: localized red glyph/text/message; keep surrounding hierarchy intact.
- Disabled: faint, no hover/focus/activation; retain readable label and reason in nearby helper text when consequential.
- Read-only: content remains fully readable; mutation affordance is absent/disabled with explicit mode/status.
- Partial/fetch-more: render a virtual row `↓ N loaded · Enter fetches more` or spinner `fetching…`; do not disguise it as data.

### Component catalog

Every item below is canonical unless marked composition or domain-shaped. Reuse `render` + `on_*` behavior; do not duplicate state styling in application code.

#### Button

- **Purpose/anatomy:** action or toggle; one row `▎` + optional `●/○` or spinner + label + padding.
- **Variants:** primary, secondary, subtle, danger, toggle; disabled and busy.
- **Layout/visual:** exact dimensions above. Primary green; secondary/toggle overlay; subtle ambient; danger red-on-overlay. Focus gutter/bold, variant hover, 140 ms press.
- **Interaction:** `Enter`/`Space`, click. Busy/disabled consume without activation; toggle mutates on/off.
- **Do:** one primary action per local decision; expose busy. **Don't:** box button, activate busy/disabled, or hand-pack without row helpers.

#### Checkbox, RadioGroup, Toggle

- **Checkbox:** independent boolean, `▎ [✓] label` / `[ ]`; `Enter`/`Space`/click.
- **RadioGroup:** one-of-many, one focus stop; label plus `▎ (●) option`; height `options+1`; Up/Down or `j/k` changes cursor and selection immediately; option rows are mouse targets, not Tab stops.
- **Toggle:** persistent setting, `▎ ──● label on` / `○── label off`; textual state shown when room.
- **Do:** select component by semantics. **Don't:** use radio for independent flags, checkbox for exclusive choice, or action button for persistent state.

#### TextInput

- **Purpose/anatomy:** validated single-line value; three rows: label, filled field, helper/error. Required `*` accent; optional suffix appears only whole; value begins `x+2`; error has trailing `!` plus message.
- **Variants/states:** required/optional/plain label, placeholder/value, help, validator, disabled, navigation/editing/selection/error.
- **Interaction:** first click focuses; second edits/positions. `Enter`/`F2` edits; single-line commit/cancel/traverse rules above; paste only while editing; loss of focus commits.
- **Do:** validate on commit; preserve navigation/edit distinction. **Don't:** claim edit changes field background—the implemented edit signal is underline plus hardware cursor—or clip “optional”.

#### TextArea

- **Purpose/anatomy:** multiline document; label + fixed body + footer, two-cell text inset, optional last-column scrollbar, line/scroll footer.
- **Interaction:** navigation scrolls; edit uses multiline semantics; click positions; wheel scrolls.
- **Visual:** current edit line uses quiet strong-border underline; selection popover fill; cursor visible.
- **Do:** use for documents. **Don't:** promise Esc rollback; it commits/leaves.

#### Select

- **Purpose/anatomy:** bounded single choice; closed three-row field with trailing `▾`; open anchored popup rows `▎ › option`.
- **Interaction:** closed arrows change choice; `Enter`/`Space` opens. Open arrows/`j/k`, `Enter`/`Space` choose, `Esc` restores selected cursor; field/option click.
- **Do:** keep choices short and bounded; close on focus loss. **Don't:** treat popup row as separate Tab stop or leave lower siblings above overlay barrier.

#### ChipBar

- **Purpose/anatomy:** one-line filter/tag tokens with optional lead/add; chip `▎ label ×`; one focus stop; one-cell gaps; `…` overflow.
- **States:** enabled/disabled/error/toggle; logical chip cursor separate from selection.
- **Interaction:** Left/Right or `h/l`; Enter activate/add; Space toggle; Delete/Backspace/`x` remove; `+` add; `X` clear; mouse lead/chip/close/add.
- **Do:** layer close hit above chip. **Don't:** wrap silently or lose overflow signal.

#### ListBox

- **Purpose/anatomy:** scrollable single/multi list; `▎`, `›` or `✓`, label, optional right metadata, optional scrollbar.
- **Variants:** single, multi, disabled rows, custom empty text.
- **Interaction:** arrows/`j/k`, page, Home/End, `g/G`; Enter/Space; multi Shift-range and `a` toggle-all; click/wheel/track.
- **Do:** keep one focus stop, skip disabled activation/range, hide metadata before label drops below 12 cells. **Don't:** move focus on wheel.

#### TreeView

- **Purpose/anatomy:** hierarchy with lazy nodes/filter; `▎`, two cells per depth, spinner or `▾/▸`, optional kind glyph, label, right metadata.
- **Variants:** leaf, directory, lazy, note, busy, custom glyph/meta.
- **Interaction:** arrows/`h/j/k/l`, pages, Home/End, `g/G`; Right expands/child, Left collapses/parent, Enter/Space toggles/activates, `*` expand all, `-` collapse all; row/fold click; wheel/scrollbar.
- **Do:** owner supplies lazy children; filter keeps ancestors and opens match paths. **Don't:** activate note rows or merge fold and row hit targets.

#### Tabs

- **Purpose/anatomy:** two-row horizontal document navigation; active label + accent underline; optional prefix, spinner/error/dirty, close, new; bare directional overflow arrows.
- **Interaction:** Left/Right or `h/l` activates; 1–9 jump; Enter/Space activate; `x`/Delete close closable; `n` new; mouse tab/close/new/overflow.
- **Do:** scroll strip and preserve label widths. **Don't:** shrink labels or show close for fixed tabs.

#### Panel and ScrollPanel

- **Panel:** card (default, filled/no frame) or framed pane. Optional title, focus, meta, badge/background override. Card focus places `▎` in title padding; frame focus strengthens border/title.
- **ScrollPanel:** focusable read-only prose/log body composed inside panel; wrap/no-wrap, styled lines, tail follow; standard scrolling plus `f`.
- **Do:** card for contained content, frame for pane boundary. **Don't:** double-box, add container and child gutter, or retain follow after manual scrolling.

#### DataTable

- **Purpose:** general sorted row/cell table with optional inline text editing.
- **Anatomy:** one-row header; gutter/selection marker; columns with two-cell gaps; optional vertical scrollbar/horizontal `…`; reversed current cell; edit field/cursor.
- **Variants:** row or cell navigation; left/right columns; fixed/min constraints; editable/sortable columns; cell tone/error; empty.
- **Interaction:** arrows/vim/pages/start/end; `s` cycles asc/desc/none; Enter/Space row selection; Enter/F2 cell edit; Tab writable-cell advance; header/cell click; wheel/track.
- **Do:** keep source-row permutation so selection/edit survives sort. **Don't:** use for pending database mutations or conflate row/current/selection.

#### DataGrid (database-domain reusable component)

- **Purpose:** typed, paged database grid with range/row selection, filters/sort requests, references, pending edit/insert/delete, undo, SQL preview, save/discard.
- **Anatomy:** header; exact row slots `▎`, `✓`, change (`•/+/-/!`), row number, cells; pending footer. Primary key `⚷`; reference `→`; filter `∇`; horizontal hidden counts.
- **Types:** null/default/text/int/number/bool/json; column Text/Id/Number/Bool/Timestamp/Json/Enum, nullable/read-only/primary/reference metadata.
- **State:** current reversed; dirty warning underline; error red/`!`; null/default muted italic; deleted faint crossed out; Error > Deleted > Inserted > Modified > Clean.
- **Interaction:** arrows/vim and ranges, horizontal page, Home/End/page/`g/G`; Enter/F2 edit/view/toggle/fetch; Space row-select; Delete null/delete; `+/-`, Ctrl+D, `u/U`, `y/Y`, Ctrl+S, `s/S`, filter keys, `p`, Ctrl+] reference; header/cell/row clicks, drag, wheel. Cells have no distinct pressed style; resulting current/selection/edit state is the feedback.
- **Do:** use only for the implemented typed pending-mutation/server-paging database workflow; owner handles query/commit. The widget itself exports SQL-preview intent and fixed preview/discard/commit action slots. **Don't:** use instead of `DataTable`, bake application record types into the widget, or save silently.

#### CodeEditor

- **Purpose:** language-agnostic code/document editor; owner supplies syntax highlighter and block segmenter.
- **Anatomy:** focus/block/line/diagnostic gutter, text viewport, optional scrollbar, find/diagnostic/position footer; syntax/selection/find/bracket/diagnostic underlines.
- **Variants:** read-only, placeholder, diagnostics, running block, inline find, completion anchor.
- **Interaction:** navigation Enter/`i`, `a`, arrows/vim/pages/`g/G`, `{}` blocks, `/`, `n/N`; editing shared multiline keys, indentation unless `tab_leaves`; click twice to edit, drag select, horizontal/vertical wheel.
- **Do:** keep language/parser knowledge in caller. **Don't:** bake SQL into generic editor or make read-only content look disabled.

#### Completion

- **Purpose/anatomy:** anchored nonmodal suggestions; kind glyph, matched label bytes bold, optional right detail; max eight rows plus frame; scrollbar.
- **Interaction:** arrows, Ctrl+`p/n`, pages; Tab/Enter accept; Esc dismiss; click/wheel. Owner keeps keyboard focus and applies `replace_len`.
- **Do:** compose with editor/input. **Don't:** trap modal focus or consume unrelated typing.

#### Picker

- **Purpose/anatomy:** centered modal ranked/grouped chooser; title/scope, optional editing query, `▎ glyph label · detail · tag · group`, footer. Columns are derived from all items so scrolling never shifts alignment: label width is clamped from `6` cells to `45%` of row; tag/group reserve their maximum widths; detail uses remaining space and appears only with at least `4` cells.
- **Variants:** searchable/fixed, scope/group/tag, alternate/secondary action, disabled rows.
- **Interaction:** Esc clears query then cancels; Enter/Alt choose; arrows/Ctrl+j/k/n/p/pages; Tab scope; Delete secondary; typing edits query; click/wheel.
- **Do:** owner ranks and resupplies rows. **Don't:** put domain ranking in widget, activate disabled rows, or rely on clipped one-line detail for safety-critical distinctions.

#### Dialog

- **Purpose/variants:** confirm, destructive, prompt, custom actions, facts/code/typed acknowledgement.
- **Anatomy:** dimmed context, centered framed elevated surface, title/body, right-aligned actions.
- **Interaction:** input captures first; trapped Tab; Left/Right or `h/l` actions; Esc cancel; plain text `y/n`; outside click cancellation when allowed. Prompt Enter submits primary. Facts acknowledgement Enter advances to action rather than confirming.
- **Do:** safe initial focus and explicit cancel action; exact token gates destructive confirm. **Don't:** allow click/key fallthrough, focus dangerous action initially, or confirm acknowledgement by Enter from field.

#### Progress

- **Variants:** determinate, indeterminate, spinner; Active/Done/Error/Paused.
- **Anatomy/visual:** `label ━━━━━──── 64% glyph`; active secondary, done green, error red, paused muted. Spinner accent + secondary label.
- **Do:** shared tick, passive display, percentage-only narrow fallback. **Don't:** make progress focusable or use green for every running bar.

#### EmptyState

Centered muted title with optional faint hint; no large glyph, focus, or action ownership. Compose owning action nearby if needed.

#### Scrollbar

One-column `│/┃`, hidden without overflow, owned by container. Not a separate Tab stop. Click/drag changes owner offset.

#### Segment bar / status indicators

One-line left/right segments with two-cell separation; each has semantic `Tone`, bold flag, priority, optional clickable ID. Low priority drops first; clickable segment gets one-cell padding and neutral hover lift. Use for identity/status, not a noisy permanent dashboard.

#### Property list

Aligned read-only label/value facts. Label column equals longest label plus two; labels muted; values semantic tone; each value wraps or truncates by variant. Use for connection details, dialog facts, plan details; do not align by hand spaces.

#### Key hints and edit badge

Footer pair is bold key + muted action; optional leading `EDIT` badge is bold `text-on-accent` on accent; right status has priority. Never wrap footer or show every possible shortcut.

### Canonical compositions, not standalone widgets

- **Form:** TextInput/TextArea/Select/Radio/Checkbox/Toggle plus action row. Validate locally; Tab follows reading order; Ctrl+S may submit at screen level.
- **Sidebar/explorer:** List/Tree row language inside framed or ambient region. Current page/object uses marker; keyboard cursor uses gutter. Collapse to drawer when secondary to workspace.
- **Split pane:** `Split` plus framed panels; one focused pane has stronger frame/title. Maximize may give one child entire area.
- **Status bar/identity strip:** Segment bar plus key hints. Keep one line, priorities explicit.
- **Execution plan:** TreeView plus overlaid metrics and Property list detail; no separate plan-tree design language.
- **Searchable picker/quick switcher/tab list:** Picker configurations; no duplicate modal primitive.
- **Autocomplete:** Completion owned by CodeEditor/Input; no modal focus.
- **Confirmation/safety gate:** Dialog facts/ack composition.

No canonical reusable diff viewer, context menu, notification/toast, or generic badge widget exists. Do not claim one. Add only after recurring need and showcase coverage.

### Data-dense UI rules

- One primary work area; secondary explorer/inspector gets bounded width and may collapse.
- Rows remain one cell. Preserve two-cell column separation; do not add vertical column borders.
- Numeric cells right-align; text left-align; null/default use muted italic; primary/reference metadata uses glyphs.
- Preserve semantic column minima. Hide entire offscreen columns and expose counts/arrows; do not squeeze values into illegibility.
- Keep current cell, selected rows, dirty values, errors, hover, and editing simultaneously legible through different slots/cues.
- Header uses muted text by default; current/sorted/filtered header becomes primary; hover underlines sortable header.
- Truncate with `…`; reference cell may reserve final cell for `→`; error reserves final cell for `!`.
- Paging/fetch-more is explicit; loaded vs estimated totals appear in quiet metadata.
- Pending mutations stay local and visible until deliberate save/discard. Preview precedes database write when available.
- Dense workspace stays calm through neutral surfaces, bounded green budget, sparse frames, and low-priority metadata—not by deleting all whitespace.

## Do's and Don'ts

### Composition rules

- **Do** begin with information hierarchy and one primary work area. **Don't** begin by drawing boxes.
- **Do** use `canvas` for foundation, `surface` for contained cards, `surface-elevated` for raised overlays. **Don't** invent a fourth arbitrary dark gray in application code.
- **Do** use whitespace before cards and cards before frames. **Don't** frame every section or nest rounded frames without a real pane boundary.
- **Do** reserve green for focus, primary action, active tab, chosen marker at focus, edit badge, spinner/indeterminate activity, and completion. **Don't** use green as general body text/background or terminal-brand decoration.
- **Do** communicate state with geometry/glyph/modifier plus color. **Don't** use color alone.
- **Do** keep hover subordinate to focus, current, selection, error, and editing. **Don't** make hover move keyboard focus.
- **Do** preserve current and selection independently. **Don't** turn every cursor move into persistent selection.
- **Do** show editing with hardware cursor and underline. **Don't** render navigation and editing identically.
- **Do** keep actions contextual and one-line. **Don't** build permanent shortcut walls or noisy status dashboards.
- **Do** use progressive disclosure for secondary panes and advanced controls. **Don't** use modal dialogs for routine navigation.
- **Do** make destructive emphasis proportional to risk. **Don't** make every warning red or paint production screens red.
- **Do** preserve component anatomy and collapse low-priority information responsively. **Don't** shrink every rectangle until labels and state cues collide.

### Anti-patterns

Prohibited unless this file and approved implementation are intentionally revised:

1. Generic Ratatui-example look: blue/yellow rainbow palette, default block around every widget, centered demo labels.
2. ncurses/Midnight Commander aesthetic: dense ASCII walls, double borders, full-screen boxed grids, inverse-video everywhere.
3. htop-style dashboard: many equal loud panels, meters competing for attention, persistent telemetry chrome.
4. Cyberpunk terminal: neon green body copy, scanline/noise decoration, unrelated magenta/cyan accents.
5. Green flood: accent as panel fill, row hover, normal text, every icon, or success/error replacement.
6. Border inflation: frame around card, nested frame around list, vertical table walls, separators where one blank row works.
7. Unicode decoration: arrows/stars/diamonds without stable semantic mapping.
8. State collapse: hover as focus, current as selection, selected as editing, dirty as error, disabled as merely unfocused.
9. Color-only state: red cell with no `!`, green selected row with no marker, hidden focus in monochrome.
10. Cramped density: removing insets/gaps/section rows before dropping optional metadata or collapsing secondary pane.
11. Arbitrary RGB: literals outside `src/theme.rs`; local “almost gray” variants; multiple unrelated accent colors.
12. Typography fiction: CSS font sizes, terminal “border radius,” shadows, hover transitions, or browser breakpoints.
13. Modal overuse: confirmation for harmless navigation, chooser for inline bounded selection, modal error for field validation.
14. Application leakage: SQL/table names/safety levels inside generic Button/List/Editor primitives.

### Token and component governance

- All render colors must come through semantic `Theme` fields/resolvers. Raw RGB belongs only in palette construction in `src/theme.rs`.
- Reuse existing semantic token even when an alias shares RGB; semantic role matters. Do not substitute `text-faint` for `border-strong` in APIs merely because values match.
- New color token requires a distinct recurring semantic role, truecolor value, downgrade behavior, state-precedence rule, showcase evidence, and this file update.
- Do not promote dormant `accent_bg_subtle`, `error_bg`, or overview-only `info` without a real recurring role.
- Reuse spacing conventions: 1 cell inline, 2 cells normal, 4 cells form columns, one blank section row, card/dialog insets. A new measurement needs repeated layout evidence.
- Extend existing component before duplicating its focus/hit/scroll/state logic. New variant requires recurring semantic need, not one screen preference.
- Domain-neutral primitives accept caller data/validation/actions; application-specific models stay in `src/bin/*` adapters. `DataGrid` is a documented database-domain exception with SQL-preview and commit workflow semantics.
- Every new interactive component must define: focus stop ownership, hover, pressed, current vs selected, disabled, error/loading where relevant, keyboard, mouse, scrolling, responsive minimum, glyph meanings, and state precedence.
- Add every generic component/variant to Showcase at `120×40` and `80×24`; add state-specific tests, not only default render.
- Never regenerate a visual baseline to hide an unintended visual change. Inspect diff and obtain design approval first.
- Update `DESIGN.md` in same change as an intentional token/state/component rule change. Implementation remains final evidence.

### Validation contract

Before accepting a new screen or component:

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.
2. Render at `72×20`, `80×24`, `100×30`, `120×40`, and `160×50` using truecolor; exercise focus, hover, selection/current, editing, error, disabled, loading, modal, and overflow states that apply.
3. Verify only one hardware cursor and one logical keyboard destination are visible.
4. Verify mouse hit order, same-target press/release, wheel ownership, scrollbar drag, outside-modal behavior, and keyboard-to-mouse transitions.
5. Verify monochrome/ANSI degradation keeps every required state legible without color.
6. Compare component anatomy, palette, spacing, border, glyph, and priority against this file and approved Showcase frames.
7. Run official DESIGN.md linter. Structural errors, broken references, duplicate sections, and accidental unknown token keys are not acceptable. Terminal-specific prose must not be distorted into browser tokens to silence irrelevant warnings.

Current evidence baseline:

- Showcase visual digest covers all 20 registered pages at `120×40` and `80×24`; it hashes symbol, foreground, background, and modifiers while excluding navigation sidebar.
- Application tests exercise Showcase at `72×20`, `80×24`, `100×30`, `120×40`, `160×50`, `200×60`; TablePro at `72×20`, `80×24`, `100×30`, `120×40`, `160×50`.
- `shots/` contains truecolor ANSI/text/HTML/PNG captures for representative default, hover, editing, pending, error, modal, narrow, and wide states.
- This is evidence, not blanket proof: new work needs relevant state captures/tests. TablePro currently has no full visual digest equivalent to Showcase.

### Current implementation exceptions

These observed behaviors are not reusable design rules. Do not copy them into new work:

- Inline popup hit barriers depend on render order. Render popup/overlay after siblings and route outside dismissal explicitly; otherwise later siblings may register above it.
- Facts-dialog acknowledgement field is keyboard-operable but current generic dialog click router does not focus that field. New modal inputs must support both keyboard and mouse.
- At `80×24`, current TablePro connection-create state can remain active while narrow connection layout renders only list. New responsive flows must never keep active controls offscreen; use full-body form/drawer/overlay.
- Composite child hits (list rows, tabs, grid cells) use IDs different from the owner's focus-ring ID. Shell mouse-down therefore records press but may not focus the owner until page/workbench mouse-up routing. New composite controls must map child hits to owner focus on mouse-down.
- TreeView selection is currently green label text only, including when unfocused. Add a stable marker/modifier before treating it as a model for new color-degraded selection states.
- Dialog `y/n`, ListBox `a`, and several DataGrid letter commands currently raw-match characters without requiring plain modifiers. Treat Ctrl/Alt leakage as an input-safety bug; new handlers must use modifier-safe matching.
- DataGrid retains cell/commit error strings internally but its renderer exposes only `!`; current owners may replace the cause with a generic failure line. Surface the specific reason/location before treating its error state as conforming to the message rule.
- Capture tool default binary name is stale; pass `BIN=target/debug/showcase` or `BIN=target/debug/tablepro` until tooling is corrected.

These exceptions document conformance gaps without changing approved visual language. Fix them structurally in implementation work; do not alter palette, spacing, border, or state vocabulary to mask them.
