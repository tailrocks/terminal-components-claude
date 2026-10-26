# Foundation API values and caller rules

## Status

This page preserves selected API rules from source commit [`1d6ee92`](https://github.com/tailrocks/terminal-components-claude/tree/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd). T37 removed the Rustdoc comments. These clauses describe intended caller rules and source observations. They do not prove current runtime behavior. T37 ran zero behavior cases. Behavior remains `NOT VERIFIED`.

The complete 322-group decision record is [t37-api-values/decisions.json](../implementation/pr38-final/results/t37-api-values/decisions.json). It records the original source identities, cleanup status, destinations, and group-specific dispositions. Raw source text in that record is provenance, not maintained caller guidance.

## Intent claims and diagnostics

- `FocusContinuationUnacknowledged` identifies an unacknowledged settled focus continuation after bounded retention expires. Its `owner` names the affected component. See the [diagnostic variants](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/diagnostics.rs#L41-L65).
- `InvalidComponentLayout` identifies a component that cannot produce valid layout from its configuration. Its optional `field` names a field or child; `None` means a structural error.
- `UnsupportedClippedComponentSlot` identifies a legacy part painter that cannot describe a partially clipped part. Its fields name the component owner and affected part.
- `claim_next_layer_event` claims the next unclaimed event for its owner when the payload matches. Other events in that owner bucket remain observable. See the [source contract](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/intent.rs#L388-L412).
- A tracked token is opaque and identifies one queue entry. Callers cannot construct it. Clones share one claim and cannot add entries. A cleared or dropped queue does not keep its entry pending. See [`LayerIntentToken` and `layer_tracked`](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/intent.rs#L110-L113).
- `Claimed` means this call claimed the exact pending entry. See the [claim result](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/intent.rs#L126-L136).
- `AlreadyClaimed` means the exact entry remains queued with an earlier claim.
- `NotPending` means it was cleared, its queue dropped, or the token belongs to another queue.
- A `Mismatch` result leaves the exact entry and its neighboring intents unchanged. The [existing authoring guide](authoring.md#runtime-routing-and-layer-receipts) records exact-token identity and stale-token behavior.
- `Diagnostics::append` keeps existing items first. It appends source items only while the destination has capacity. It adds the source dropped count with saturating arithmetic. See the [source method](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/diagnostics.rs#L68-L110).
- Capacity limits retained diagnostic items. It does not cap the dropped count. The comment can be read as a cap; no source behavior change is authorized here.
- `INACTIVE_LAYER` reports an inert layer context, such as a modal, popup, or reference frame. It distinguishes temporary non-interaction from a configured disabled state. See the [state flags](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/response.rs#L41-L65).
- For `RoutedInput`, middle-button Down uses the routed variant. Every other event uses the existing `Input` decoder unchanged. See the [decoder](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/event.rs#L46-L53).

## Keyboard protocol and delivery

- `ProtocolState::enter` resolves `Auto` through the supplied detection probe. It returns a push sequence only when the state enables the protocol.
- `leave` returns a pop only when this state pushed flags.
- `resume` pushes once when enabled and not already pushed. See the [protocol state](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/keys.rs#L181-L217).
- The core `KeyboardProtocol` value defaults to `Auto`; the session policy defaults to `Off`. The [keyboard policy](authoring.md#keyboard-list-layout-and-text-rules) records negotiation and environment-override rules.
- `TERMROCK_KEYBOARD_PROTOCOL` accepts `on`, `1`, `true`, or `kitty` for `On`, and `off`, `0`, `false`, or `legacy` for `Off`. Other values resolve to `Auto`. See the [policy parser](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/keys.rs#L140-L168).
- The T20 target assigns protocol lifetime to one session-owned state and retires global mirrors. These are target rules, not proof of current behavior.
- `AltKeys::Refused` permits caller-installed stand-ins.
- `UntilSeen` keeps macOS Option unconfirmed until an unambiguous Alt chord arrives.
- `WithProtocol` allows Alt only while Kitty protocol flags are active.
- `TERMROCK_ALT_KEYS` accepts `on`, `1`, `true`, or `yes` for `Delivered`. It accepts `off`, `0`, `false`, or `no` for `Refused`.
- Without a recognized override, non-macOS clients resolve to `Delivered`.
- On macOS, `LC_TERMINAL=iTerm2`, `TERM_PROGRAM=Apple_Terminal`, or `TERM_PROGRAM=iTerm.app` selects `UntilSeen`.
- `TERM_PROGRAM=WezTerm` selects `Delivered`; other macOS programs select `WithProtocol`. See [`AltKeys::from_env`](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/keys.rs#L309-L358).
- Legacy literal hints reject `Shift+click`, `Shift+drag`, `Shift-click`, and `⇧click` prefixes. Super prefixes require Super delivery.
- Alt prefixes require Alt delivery unless their suffix starts with `click` or `drag`. See the [literal-label filter](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/keys.rs#L388-L423).
- `observe` learns Super unless delivery was refused.
- For an Alt-modified key, `observe` learns Alt only when `ALT == 0`.
- Learning also requires active protocol flags or a code outside these variants: `Char`, `Esc`, `Enter`, `Backspace`, `Tab`, and `BackTab`.
- T20 owns global observation; T19 owns runtime hints.

## List, selection, and type-ahead

- A context click keeps an existing selection when the clicked row is selected. An enabled row outside the selection becomes the sole selection. A disabled outside row clears selection through `select_only`. See [`context_select`](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/selection.rs#L388-L425).
- `select_only` clears selection when its row is disabled. A disabled row does not toggle. See the [selection operations](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/selection.rs#L280-L329).
- `RowSelection` stores source indices and has no rendering behavior. Callers provide enabled-row predicates to selection operations.
- `set` changes row membership without changing the range anchor. Use `retain` after filtering. Use `remap` after a source reorder or removal; a `None` result drops that row from selection.
- On macOS, Option or Cmd toggles selection. On other clients, Ctrl or Cmd toggles it. See the [gesture rules](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/selection.rs#L76-L115).
- A click uses Shift to extend, a toggle modifier to flip, and no modifier to select only the clicked row. Shift has priority.
- For key gestures, unmodified Space toggles. A shifted movement key without Ctrl, Alt, or Super extends.
- A glyph-only checkbox scope toggles only when the checkbox glyph is hit. A row scope may toggle from anywhere in the row. See [`CheckClick`](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/selection.rs#L147-L177).
- The navigation click policy activates on a single click and never edits. The data-grid default activates and edits on a double click.
- Range extension starts from the saved selection base, excluding the anchor. It adds selectable rows in the displayed range. Disabled rows already in the base can remain selected; rows selected only by an earlier extension can drop. See [`RowSelection::extend`](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/selection.rs#L264-L336).
- List cursor navigation is an opt-in capability. Existing widgets keep their prior routing until they adopt it.
- `Shortcuts` precedence gives bound printable letters to shortcuts first; `TypeAhead` precedence sends every printable character to type-ahead. The shortcut-first route may offer an unchanged `Ignored` key to type-ahead.
- `Cursor::new(index)` stores the index unchanged; it defaults to vertical, no-wrap movement that skips disabled rows. It has no row source for validation. See [`Cursor::new`](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/list.rs#L132-L140).
- `step` bypasses disabled rows when `skip_disabled` is true. False permits landing on them.
- `Moved` maps Ignored, Stayed, and To to the matching `Outcome` variants. `Outcome::or` keeps the strongest: Changed, Consumed, then Ignored.
- `consumed()` is false for Ignored and true otherwise.
- A prefix resets after a pause longer than its timeout. The default timeout is one second; a repeated single letter cycles from the cursor.
- Type-ahead accepts bare alphanumeric characters. Punctuation and modified keys remain available to the application. See the [type-ahead rules](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/typeahead.rs#L9-L80).
- Type-ahead starts at the supplied row and wraps once. It returns the first eligible label whose lowercase text starts with the lowercase prefix.
- Repeated one-letter prefixes start after the current row.
- `Moved::Ignored` means movement does not apply, including wrong axis or empty input.
- `Stayed` means edge, all-disabled list, or type-ahead miss. `To` carries the new index. See the [cursor result](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/list.rs#L77-L108).
- `Outcome::Ignored` leaves input available to later handlers. `Consumed` means handled without a visible change; `Changed` means handled with a change. See the [input outcomes](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/outcome.rs#L4-L29).
- `ClientPlatform::from_env` checks `TERMROCK_CLIENT_PLATFORM` first. It recognizes `macos`, `mac`, or `darwin` as macOS, and `other`, `linux`, or `windows` as other.
- An unknown override falls through to `LC_TERMINAL=iTerm2`, then `TERM_PROGRAM=Apple_Terminal` or `iTerm.app`, then the build target.
- `current` caches its first result per process. See the [platform resolver](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/platform.rs#L20-L54).

## Layout and bounded work

- The [existing composition rules](authoring.md#keyboard-list-layout-and-text-rules) require pure measurement callbacks. Each callback receives bounded constraints and must not handle input or cause domain effects.
- Intrinsic composition size includes child sizes, padding, and gaps. Alignment and flex use only space supplied by the parent.
- Wrapping preserves source order and starts a new line before a child whose preferred width exceeds remaining content width. Flex resolves within each line; line height is the tallest measured child.
- When minima exceed main-axis space, `Compress` shrinks in source order and clips final overflow.
- `Clip` preserves child minima and clips visible bounds.
- `ColumnSpec` constructors normalize bounds so `min <= ideal <= max`. Private fields keep the solver's input within those bounds. See [`ColumnSpec`](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-layout/src/columns.rs#L3-L52).
- `solve_columns_first` avoids allocation for up to four specifications. Larger slices use the full solver without truncation.
- `solve_columns_head` avoids allocation for up to four specifications. It returns zero for a requested column beyond the slice; larger slices use the allocating solver. See the [bounded helpers](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-layout/src/columns.rs#L154-L170).
- Use basis points to retain a precise split ratio across resizes. The public `percent` field remains the coarse compatibility value.
- A first-pane extent follows the split axis and excludes the gap. The resolver applies configured minima after it receives that extent.
- A fixed first-share request rounds to the nearest cell. A fixed second-share request rounds down; both requests respect pane bounds and the other pane's minimum.
- Required-extent methods use their matching layout resolver. Fixed first- and second-share methods use their matching share resolver. This keeps measurement and drawing aligned on rounding and pane minima.
- Basis-point drag and nudge updates retain the precise ratio for later resizes. A nudge moves the first pane by its requested cell delta. See the [split resolvers](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-layout/src/layout.rs#L393-L836).
- `ShapeSync` is opt-in. The frozen source comment reports no production emitter; this is a repository observation, not behavior proof. See the [shape module](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/shape.rs#L1-L6).
- `ShapeSync::forget` clears its remembered pointer and caret shapes. The next supported `apply` can resend the requested shapes. See [`ShapeSync::forget`](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/shape.rs#L180-L184).

## Text offsets and undo

- Selecting a line excludes its newline. See [`select_line_at`](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-text/src/buffer.rs#L329-L337).
- Insertion keeps the current selection; it clamps the byte offset and floors it to a grapheme boundary.
- Cursor and anchor offsets at or after insertion shift by inserted bytes. The cursor rounds up; the anchor rounds down.
- Removal uses a clamped half-open byte range. Empty or reversed ranges do nothing.
- Positions at or before expanded start stay put; inside positions collapse there. Positions at or after end move left by removed bytes.
- The cursor rounds up and the anchor rounds down after removal. See the [buffer edits](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-text/src/buffer.rs#L598-L649).
- `TextBuffer::pos_of` clamps past-end offsets and maps an offset inside a UTF-8 scalar to that scalar's start.
- Valid UTF-8 boundaries stay as supplied, even inside a grapheme. See [`pos_of`](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-text/src/buffer.rs#L699-L704).
- Record an undo snapshot before applying an edit. A record clears redo history; consecutive typing or deleting records fold into one step.
- An `Other` edit breaks the current run. Call `seal` to end a run before the next edit. The stack keeps at most 200 undo steps.
- Undo restores the previous snapshot and places the current state on the redo stack. Redo restores the undone snapshot and places the current state on the undo stack. See the [undo rules](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-text/src/undo.rs#L17-L103).

## Open source wording conflicts

- `current` and `install` use the calling thread's cell. Each thread starts with `Shortcuts` precedence and the `Desktop` navigation profile. An install persists in that thread until changed.
- These values are not process-wide or runtime-instance isolated. They do not propagate across threads. The comments say “process-wide”, but the source uses thread-local storage. See [`list.rs`](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/list.rs#L330-L445).
- The `Cursor::new` comment says it clamps its index, but initialization has no row source. It stores the supplied index and sets vertical, no-wrap, skip-disabled defaults. T14 bounds and eligibility cases must distinguish initialization from later movement; no source defect is established.
- The `observe` comment describes Super and protocol-active Alt evidence. The code also learns some non-text Alt keys without active protocol flags. This page records the source predicate; T20 owns the target transition. See [`observe`](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-core/src/keys.rs#L275-L308).
- The selection module says disabled rows are never selected. It says range extension preserves their prior state.
- The implementation rebuilds from saved base, then adds selectable rows. Keep this wording gap open for T01 review.
- `from_key` documents movement keys. Its predicate accepts any shifted non-character without Ctrl, Alt, or Super.
- Callers must supply a movement key for extension.
- The `Diagnostics::append` comment says dropped counts obey item capacity. The implementation accumulates prior dropped counts with saturation. This page records the exact source observation and does not infer a defect.
