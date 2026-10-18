# Termrock visual contract

This document owns the rendered visual rules for the in-place Termrock refactor.
It describes the output that the library must preserve while its implementation
and public names change. It does not authorize a redesign of `showcase`,
`tablepro`, `jackin-preview`, or `holla`.

The visual authority is the frozen `visual-baseline` commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. The approved cells, cursor state,
and application frames at that commit are the target. Read this contract with
the [architecture overview](../architecture/overview.md), the [theme
foundation](../foundations/theme.md), the [component index](../components/README.md),
and the [visual parity proof](../verification/visual-parity.md).

## Authority and evidence

Visual parity has three explicit evidence lanes:

1. **ExistingOracle** is an approved artifact already present under the frozen
   source tree, including the protected [baseline tree](../../baselines/tuiscotti-v1/) and
   application output.
2. **ExtractedOracle** is a newly isolated component or state captured from the
   unchanged baseline by a reviewed oracle adapter.
3. **Extension** is a reviewed Termrock capability with no equivalent baseline
   frame. It is tested separately and is never used to alter an ExistingOracle.

The candidate implementation cannot create, accept, or bless its own expected
output. Custom themes and parts use the separate [customization contract](../architecture/component-composition.md#arc-008--customization-contract).
A custom theme never excuses default-theme drift.
The exact capture, provenance, comparator, and tool limits belong to
the [oracle and provenance contract](../verification/oracle-and-provenance.md)
and [conformance contract](../verification/conformance.md). A snapshot that
looks better to a reviewer is still a regression if it differs from the
approved baseline without a separately approved product change.

Useful baseline evidence includes:

- current theme and token resolution in [`src/theme.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/theme.rs);
- current text measurement and projection in [`src/ui/text.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/text.rs) and [`src/core/text.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/core/text.rs);
- current scrollbar and edge fade behavior in [`src/widgets/scrollbar.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/scrollbar.rs) and [`src/ui/fade.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/fade.rs);
- current component recipes under [`src/widgets/`](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets);
- unchanged application compositions under [`src/bin/`](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin).

The source pack's visual rules are consolidated here from
`termrock-library-spec/foundations/theme.md`,
`termrock-library-spec/foundations/layout.md`,
`termrock-library-spec/foundations/text.md`,
`termrock-library-spec/components/**`, and
`termrock-library-spec/VISUAL-VERIFICATION.md`. Those files are migration
input; this repository's `docs/` tree is the current authority after the
rewrite.

## Visual language

Termrock preserves a near-black canvas, a small neutral surface hierarchy, a
white text ladder, and one green accent. Colour supports the state grammar; it
does not replace geometry, glyphs, weight, underline, or cursor evidence.

### Semantic surfaces and tokens

The default `Theme::termrock()` must reproduce the following baseline anchors.
The values are semantic theme data, not literals for component painters.

| Role | Baseline value | Use |
| --- | --- | --- |
| `canvas` | `#000000` | page and pane background |
| `surface` | `#111111` | cards and grouped content |
| `surface-elevated` | `#18181b` | dialogs, popups, picker surfaces, canvas-row hover |
| `surface-overlay` | `#27272a` | secondary/toggle/danger controls and lifted surface rows |
| `popover` | `#3f3f46` | anchored menus, text selection, grid range selection |
| `field` | `#1e1e22` | editable field body |
| `field-hover` | `#232328` | non-editing field hover |
| `highlight` | `#2f5aa8` | current anchored-menu command row |
| `highlight-danger` | `#7a2a2a` | destructive menu row under the cursor |
| `error-soft` | `#d98a8a` | resting destructive label on a neutral plane |
| `border-subtle` | `#262626` | quiet frame, tab baseline, track |
| `border-strong` | `#4d4d4d` | focused frame, neutral underline |
| `text-primary` | `#ffffff` | main content, titles, focused labels |
| `text-secondary` | `#b3b3b3` | supporting content, busy labels, active progress fill |
| `text-muted` | `#808080` | metadata, placeholders, hints, headers |
| `text-faint` | `#4d4d4d` | disabled content, comments, row numbers, quiet metadata |
| `text-ghost` | `#262626` | modal-backdrop text only |
| `accent` / `focus` | `#48e054` | focus, current markers, primary action, live activity |
| `accent-hover` | `#3ab343` | primary-button hover |
| `accent-pressed` | `#2b8632` | primary-button press |
| `accent-tint` | `#0f2e13` | selected and focused row plane |
| `on-accent` | `#19191c` | text on an accent fill |
| `error` | `#e44545` | invalid/error state and diagnostics |
| `warning` | `#f59e09` | dirty, pending, and warning state |

The default palette reserves blue for transient menu selection. It must not
become a second general accent. Red and amber are safety tones. Dormant source
fields such as `info`, `error_bg`, and `accent_bg_subtle` are not new visual
roles unless a future extension receives its own contract.

Hover lifts exactly one plane and keeps the same hue:

```text
canvas            -> surface-elevated
surface           -> surface-overlay
surface-elevated  -> surface-overlay
field             -> field-hover
other             -> popover
```

Hover is suppressed on disabled controls and while an editable field is in
editing mode. Hover never moves keyboard focus and never replaces a selected
marker with a colour-only indication.

Depth is tonal, structural, and modal; there are no shadows. Use whitespace
before decoration: blank row before a card, card before a frame, frame before
an overlay. Cards have no border, independently scrolling panes and floating
surfaces use rounded frames, and child widgets are not boxed inside a card.

The accent has a closed allowlist: focus gutter; primary action fill; `›`/`✓`
markers on a focused current row; active document-tab underline; the `EDIT`
badge; required-field `*`; checked-box `✓`; spinner and indeterminate sweep;
completed progress. It is not body text, counts, environment identity, dirty
state, or a general background. Accent hover/press tones belong only to the
primary button. A selected-and-focused row uses `accent-tint`; an unfocused
selection has no tint, and a current hover lift replaces the tint.

The baseline deliberately retains a few low-contrast pairings: pressed primary
text on `accent-pressed` for 140 ms; resting danger label on its neutral plane
and pressed white-on-error; placeholder on `field`. Each has a second signal
in a glyph, label, or the value that replaces the placeholder. Danger buttons
use the red label at rest and the error fill only while pressed; red is not a
routine destructive accent.

### Text hierarchy and modifiers

Terminal fonts and cell metrics belong to the terminal. Termrock controls tone,
weight, slant, underline, strikethrough, alignment, case, and spacing:

| Role | Treatment |
| --- | --- |
| title, focused label, focused row | `text-primary` + bold |
| ordinary content | `text-primary` |
| supporting value or busy label | `text-secondary` |
| metadata, placeholder, helper, hint action | `text-muted` |
| disabled content, comments, row numbers | `text-faint` |
| status warning / dirty | warning plus `•` or `▲` |
| status error / diagnostic | error plus bold `!` |
| selected content | `›` or `✓` marker; label remains readable |
| key hint key | bold `text-primary` |
| key hint action | muted |
| code keyword | bold |
| code string or number | secondary |
| code operator or punctuation | muted |
| code comment | faint italic |

Modifiers have stable meanings. Bold marks the keyboard destination or a
heading. Accent underline means active editing. Strong-border underline means a
quiet affordance such as a hovered editable cell, bracket match, other find
match, or current multi-line line. Error/warning underline marks diagnostics.
Strikethrough is reserved for a row queued for deletion. Do not use dim or
reverse video as a general hierarchy shortcut; the monochrome rules below
define the limited reverse-video exception for text selection.

Italic is reserved for `NULL`/`DEFAULT` values and code comments. Dim is not a
general hierarchy tool; monochrome disabled content uses DIM only where the
baseline specifies it. Reverse video is limited to the current cell/cursor and
text-selection recipes. Use sentence case for titles, buttons, hints, and
labels; `EDIT`, SQL keywords, and literal identifiers retain their source
spelling. A trailing `…` means an action opens a dialog or needs more input
(`Delete…`); terminal actions such as `Cancel` and `Save` have no ellipsis.
Use ` · ` between same-line clauses, with the next clause lower case; ` › `
for hierarchy; `…` for truncation; an en dash for ranges; grouped thousands
and a spaced unit for durations. Status sentences are past tense without a
period. A colon introduces a refusal reason. Numeric cells right-align, text
left-aligns, and ordinary rows remain one cell high.

Use sentence case. Preserve literal labels and application-visible names from
the oracle, including `jackin❯`, `holla❯`, `TablePro`, and `EDIT`. Truncate with
the single ellipsis glyph; preserve identifiers' tails with middle truncation
where the baseline does so. Numeric values are right-aligned; text is
left-aligned; ordinary rows remain one cell high.

### Geometry, density, and glyphs

All measurements are terminal cells. The baseline rhythm is:

| Metric | Cells |
| --- | ---: |
| focus gutter | 1 |
| inline gap | 1 |
| pane/column gap | 2 |
| form gap | 4 |
| card inset | 2 |
| frame inset | 3 |
| dialog inset | 3 |
| tree indent | 2 |
| field height | 3 rows (label, value, message) |
| tab height | 2 rows (label, underline) |

The common row anatomy is a one-cell focus gutter at the row start, a separate
one-cell marker slot, and content beginning after that. A focused control paints
`▎` in the gutter; an unfocused or disabled control paints a real space. A
selection marker never occupies the gutter. Cards are filled planes; rounded
frames are used only where a pane or modal needs an edge. Do not box every
child or add a second frame around an already framed surface.

Baseline glyph meanings remain stable: `▎` focus, `›` current/selected row,
`✓` checked/completed, `•` dirty, `!` error, `▲` warning, `│` scrollbar track,
`┃` scrollbar thumb, `…` truncation, `‹`/`›` hidden content, and the ten
spinner glyphs `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`. Components may use their documented local
glyphs, but must not assign an existing glyph a second meaning.

The baseline glyph ownership table is:

| Glyph | Meaning | Baseline use |
| --- | --- | --- |
| `▎` | keyboard focus | row/control gutter or focused card title |
| `›` | current or chosen item | list, table, select, editor block; also path separator by context |
| `✓` | checked, selected, completed | checkbox, multi-select row, grid row, finished progress |
| `•` | modified or pending | dirty grid row/tab and pending bar |
| `+` / `−` | inserted / deleted | grid change slot |
| `!` | error or diagnostic | field, tab, grid row, editor gutter, failed progress |
| `▲` | warning weight | warning/cost cue |
| `▸` / `▾` | collapsed / expanded | tree disclosure; spinner while children load |
| `▴` / `▾` | sort ascending / descending | sortable header; `▾`/`▴` also closed/open Select |
| `∇` | filtered column | grid header suffix |
| `▪` | primary key or list bullet | primary-key header or picker row kind |
| `→` | follow a reference | foreign-key cell |
| `↓` | more rows available | fetch-more virtual row |
| `‹` / `›` | hidden content direction | tab overflow and hidden-column count |
| `…` | truncated or clipped | text, horizontal scroll edge, collapsed JSON |
| `×` | close or remove | tab and chip affordance |
| `●` / `○` | on / off | toggle and radio state |
| `[✓]` / `[ ]` | checked / unchecked | checkbox |
| `◆` / `◇` | production / staging | environment identity |
| `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏` | activity | ten-frame spinner, primary tone, 80 ms tick in baseline consumers |

The `▾` overlap between tree and Select is disambiguated by component
context. Do not extend that collision to another component.

## State rendering rules

These rules apply when a component advertises the corresponding state. A
decorative component does not gain fake focus, hover, or pressed frames merely
to fill a matrix. The [component contracts](../components/README.md) define
applicability and local recipes.

| State | Required visual evidence |
| --- | --- |
| normal | baseline surface, text tone, spacing, and glyphs |
| focused | `▎` gutter or documented focused frame/title treatment; bold destination |
| hovered | one-plane lift; content and focus marker remain semantically unchanged |
| focus + hover | both focus gutter/weight and hover lift |
| keyboard-suppressed hover | no stale hover lift after a key until real pointer motion |
| selected/current | stable marker (`›`, `✓`, or component-specific documented marker) |
| selected + focused | marker plus accent-tint where the recipe calls for it |
| pointer down / held press | press-owned visual while capture remains valid |
| release inside | one activation and its feedback state |
| release outside | no activation; no click-through |
| activation feedback | pressed/flash recipe for exactly the characterized interval |
| disabled | faint/readable, no hover or focus bar, no activation |
| read-only | readable content and navigation/selection where supported; mutation affordances absent |
| editing | field plane, accent underline, hardware cursor, and `EDIT`/local editing indicator where the baseline uses one |
| valid / invalid | ordinary value versus error tone and bold `!` plus message |
| empty / loading / partial / ready / error / success | explicit state content and status described below |

Focus, selection, current cursor, hover, held press, and edit mode are separate
axes. A selected but unfocused row keeps the marker without accent-tint. A
focused row keeps the gutter even when hovered. Composite activation styling is
painted by the real child control; a whole dialog is not made to look pressed
because one action button was activated.

### State content

- **Empty:** centered or component-local muted title with a faint, wrapped hint
  that names the available action. No invented rows are used to fill space.
- **Loading:** primary spinner plus secondary label; a busy button drops bold,
  refuses activation, and keeps its reserved geometry. A lazy node uses its
  disclosure slot.
- **Partial:** show loaded/total facts and a fetch-more or continuation affordance
  when supplied by the caller. Do not imply that partial data is complete.
- **Ready:** render the normal content and any position/total line.
- **Error:** place red `!` and the message at the field, cell, tab, editor
  gutter, or footer where the failure occurred; retain focus and value when
  recovery is possible.
- **Read-only:** keep content fully legible, remove mutation affordances, and
  expose the reason in the surrounding status when one exists.
- **Success:** show the baseline status sentence and completion marker for the
  owning application or fixture; no toast or flashing replacement surface.

Partial lists use a virtual row such as `↓ 500 loaded · Enter fetches more`;
estimated totals carry `~`. No-match uses the same centered empty-state shape
with its specific message. A recoverable error places the red `!` and message
where the failure happened, keeps value and focus, and offers retry only when
the caller supplies it. A disconnected item can show its error/detail and a
Retry action as caller data. Disabled actions use `text-faint`, leave the Tab
ring, and may have adjacent muted explanation. Pending changes use `•`,
warning-toned changed values, and an explicit pending count/action row; do not
show a success state until the caller reports one.

Disabled and read-only are different. Disabled means unavailable: no Tab stop,
focus gutter, hover, or activation. Read-only means available for navigation,
selection, scrolling, and copy as applicable, but no mutation. A disabled
selected row may retain its marker while using the disabled tone; it must not
become a live target.

### Editing and cursor presentation

Editing is visibly distinct from navigation. The insertion cursor is the
hardware cursor, positioned at a grapheme boundary. An editing field keeps its
field plane, underlines the active text or current line in the accent, and
shows its local editing indicator when the baseline composition has one. A
focused non-editing field shows the gutter and remains navigable.

Selection uses the popover plane with primary text. In monochrome it additionally
uses reverse video so selection remains visible when colour escapes are absent.
Wide-cell continuation, combining marks, tabs, truncation, and copy/display
projection follow the [text foundation](../foundations/text.md); a visual frame
must never split a grapheme or leave a stale wide-cell continuation.

### Scroll surfaces, bars, and fades

Scrolling belongs to the owning container. A vertical scrollbar appears only
when content overflows and occupies one cell: `│` for the track and `┃` for the
thumb. The track uses `border-subtle`; the thumb is muted at rest, secondary on
hover, and primary-text when the container is focused. Clicking the track jumps
the view; dragging the thumb preserves the grab offset. The wheel operates on
the topmost container under the pointer without moving focus.

Position labels use the baseline form such as `12–24 of 120`; grids add the
loaded/total and column ranges appropriate to their contract.

An overflowing scroll surface fades only the edge that hides additional
content. The outermost faded row retains 55% of its foreground contrast. A
viewport tall enough for the two-row ramp gives the next row 80%. Short
viewports where every row is meaningful receive no fade. Selected, focused,
hovered, marked, reversed, and cursor rows are protected as whole rows and are
never partly faded. The exact implementation and protected-cell behavior are
owned by the [layout](../foundations/layout.md) and [component](../components/README.md)
contracts; the baseline evidence is [`src/ui/fade.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/fade.rs).

## Layers, backdrops, and focus-visible composition

### Modal backdrops

Dialogs and modal pickers are centered or placed according to their component
contract on the elevated plane with a rounded strong frame. Before painting the
modal, dim the covered application area through the theme backdrop resolver:
preserve surface planes and shape, lower live foregrounds through the text
ladder, and remove coloured fills/selection tints that would misrepresent the
inactive page. The footer or other explicitly live modal-owned chrome remains
live when the baseline does so. Never dim the modal itself.

Baseline backdrop transformation is deterministic: primary, accent, error,
and warning foregrounds become `text-muted`; secondary becomes `text-faint`;
other foregrounds become `text-ghost`; `field` backgrounds become
`surface-elevated`; every other colored fill becomes `surface-overlay`; and
all modifiers are cleared. The footer row is excluded when it carries live
modal hints. Preserve surface shape and blank cells. This is a cell transform,
not an opacity or shadow effect.

The backdrop is composited by written-cell ownership and clipping. A partial or
transparent overlay cannot overwrite unrelated cells. An outside click dismisses
only the top cancelable layer and never activates a control below it.

### Focus and hit visibility

The frame rebuilds visible enabled focus stops in render order. A modal pushes a
focus and hit barrier; content below is inert. The modal's initial focus is
component-specific: a confirm action for routine confirmation, Cancel for a
destructive action, the field for a prompt or typed acknowledgement. Closing a
layer restores the saved focus if its identity survives, otherwise the first
reachable valid stop. Opening a layer clears stale hover and press state.

Anchored menus, completion, select, and other popovers are above siblings and
claim pointer ownership. They may leave keyboard focus with their owner when
their contract permits editor typing. Escape dismisses one top applicable layer
at a time. A resize reanchors an anchored layer within the screen; a removed
owner releases its layer and capture.

## Responsive and degenerate geometry

The baseline applications use a 72×20 minimum for their composed frame. At
71/72/73 columns and 19/20/21 rows, capture the below, exact, and above cases.
At the composed minimum, the baseline displays its centered four-line too-small
notice and no underlying app content. This threshold does not become a global
minimum for every component: every component also proves local 0×0, 1×1,
nonzero-origin, exact-fit, one-cell-short, and narrow-to-wide cases.

Responsive layout is prioritization, never scaling text or overlapping controls:

1. Drop low-priority header/identity segments from the right.
2. Drop footer hints from the right without cutting a word; keep status space.
3. Drop row metadata all-or-none for the view.
4. Drop lower-priority status columns.
5. Collapse secondary panes into a drawer or give a split one side.
6. Truncate with `…`; expose hidden tabs/columns with their arrows/counts.

Labels are not shrunk to fit and controls never overlap. A resize preserves
caller-owned state, draft, selection, scroll policy, and surviving focus where
the component contract allows it. A too-small transition must not reconstruct
the scene or silently select a different item.

## Capabilities and motion

The same visual state must remain identifiable in each capability lane:

- truecolor;
- ANSI 256;
- ANSI 16;
- explicit no-color/mono;
- `NO_COLOR` environment mode.

In the frozen applications, `COLORTERM=truecolor` selects truecolor; a `TERM`
containing `256color`, `ghostty`, or `kitty` selects the nearest xterm-256
palette value; other terminals use 16 named colors. `NO_COLOR` forces the
monochrome path. The binaries accept `--color truecolor|256|16|none` with
documented aliases. At 16 colors, accent maps to LightGreen and error to
LightRed. Monochrome removes hue while retaining the state glyph/modifier
language; the final backend strips color metadata but preserves modifiers.
The requested capability cannot exceed the terminal ceiling.

Capability projection happens in the theme, and user-requested capability cannot
exceed the terminal ceiling. In every lane preserve the focus gutter, bold
focus, editing underline, hardware cursor, selection marker, `!`, `›`, `✓`,
and `•`. Unfocused/disabled gutters contain a real space, not a hidden glyph
whose colour happens to match its background. Monochrome uses reverse video for
selection and DIM for disabled content only where the baseline uses it. A
truecolor screenshot does not prove the other lanes.

Animation receives an explicit phase/time sample and motion policy. A spinner
has ten unique glyph phases and tests its wrap; a phase index does not define a
universal wall-clock cadence. Button activation feedback is characterized at
0, 139, 140, and 141 ms. Full-motion, paused, and reduced-motion captures are
separate. Pausing visual motion may freeze a spinner or transition, but it must
not freeze caller data updates. Every unique phase, phase wrap, and deadline
boundary that a component advertises is part of its proof.

## Non-negotiable preservation rules

- The four applications and their fixture worlds remain conformance consumers,
  not redesign targets.
- The future Termrock default theme reproduces the baseline tokens, glyphs,
  spacing, density, surfaces, clipping, and motion policy.
- Consolidating implementation mechanisms does not flatten component recipes:
  Grid, Menu, Picker, text editing, and ScrollRegion share engines while their
  documented visual differences remain.
- A style override is semantic and constrained to the documented part. It cannot
  replace geometry, focus/capture ownership, or the whole component surface.
- Any intentional visual change requires a separate product decision and a new
  oracle disposition. This refactoring specification contains no such change.
