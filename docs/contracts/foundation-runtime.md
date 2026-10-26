# Foundation runtime caller rules

This page preserves selected runtime and session rules from frozen source input `c817b99e9e43abbbf3519d56b2f5ee86924243d8`.
The accepted source cleanup is `7368d4fc5a71e46cae5ee92963bcf3a6943953e9`.
These are source contracts, not behavior proof; product behavior remains `NOT VERIFIED`.
The T37 disposition record maps each raw group to maintained text or a group-specific reason.

## Wheel routing and scroll windows

`App::route_wheel_attempt` is an opt-in fallback callback. It receives no ordinary intents and dispatches only the named wheel registration.
Resolve the live source for the attempt position and axis. Check its registered source revision before acknowledging one local result.
The response contributes repaint or layout invalidation only. It does not report whether the target handled the wheel.
Attempts from one physical wheel share an event identity. `attempt_index` identifies each target's position in the bounded route.
The attempt delta is already scaled and stays the same for every target.
`Ignored` means no wheel-related change; `Consumed` means accepted without a model change; `Changed` means the model changed.
Accept one acknowledgement only. A stale, wrong, duplicate, or conflicting acknowledgement invalidates fallback.
`Cx::wheel_route_attempt` is available only during the original event update or targeted `ScrollRoute` callback.
It must match the exact owner, part, and axis.

An opt-in `ScrollRouteKey` belongs to one registry frame. Ordinary scroll registrations keep single-target wheel routing.
`register_event_local_hit_stack_wheel` adds scroll routing metadata without changing an existing control or part registration.
Only other targets enrolled through the opt-in method receive fallback offers.
Only pointer-enabled targets on the same layer, containing the position and handling the axis, take part in ambiguity checks.
Repeated owner and part registrations may use disjoint geometry or axes. Exact duplicates coalesce; conflicting eligible registrations stop fallback.
Link each routed child to its immediate containing parent. That parent must be the next eligible target below it in reverse hit order.
Invalid parent links disable routing for the child.

Keep one source revision across routed update and draw. Advance it for source replacement, reorder, or other changes affecting scrolling or layout.
Do not derive revisions from content length or state generation. Do not reuse a revision while an older registration can still be serviced.
At exhaustion, fail closed; never wrap and reuse a revision.
Check the live revision before layout preparation or scroll-state mutation. Leave missing or stale attempts unacknowledged and unchanged.
Ordinary `ScrollRegion::update` behavior remains unchanged. A route key is frame-local; empty or clipped-away windows return no key.
Use `draw_without_wheel_registration` only when the caller registers its own disjoint wheel targets for that painted region.

For a windowed part painter, `part_rows` names its original logical interval. `visible_rows` is that interval intersected with the window.
The callback rectangle matches that intersection's physical rows. A windowed scroll region retains logical state and thumb sizing.
The window-slot callback takes precedence for clipped parts. Legacy slots remain unchanged when the entire part is visible.
Passive-reveal regions use vertical wheel input. Eligible unmodified input scrolls three lines; unavailable, modified, horizontal, zero-delta, or edge input may continue on the same top layer.

Frozen source: [`App` and wheel attempts](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/runtime.rs#L112-L257), [`wheel registration and parent links`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/mod.rs#L1185-L1283), [`routed ScrollRegion`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/scroll_region.rs#L211-L515), [`position label`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/scroll_region.rs#L31-L52).

## Hit ownership and releases

A non-interactive region participates only in top-layer hover. A backdrop is a light-dismiss region registered behind a popup or modal.
Register a non-empty scrollbar bar before its thumb. Its track becomes the owner's `TRACK` part.
A thumb without a registered bar is ignored.
Thumb hits return the scrolling widget as owner and keep the bar ID as their control ID.
`has_owner` accepts either a hit owner or the region's control ID. A backdrop hit returns its owner; its ID remains the control ID. `Registry::bars` returns registrations oldest first; `Registry::bar` looks up only a registration from the current frame. `Registry::region_at` considers the topmost containing region of any kind. Backdrop routing proceeds only when that region is a backdrop.
For secondary presses, `Nothing` means no hit; `Target(hit)` means the press landed on that hit.
Release acceptance uses the hit owner, which is the ID the widget gives `Registry::accept_release`.
A press-origin release remains associated with its original owner; secondary-release acceptance is a separate choice.
A secondary drag updates hover so release can target a new hit. Release is a click only after pointer movement and owner acceptance. `SecondaryTracker::on_secondary` records the press point and hit; `hover()` returns the current hit.
The backdrop helper finds the topmost backdrop and searches below it for a containing, pointer-enabled `Control` or `Part` region. Hover-only regions are excluded.
`BarInput::Press` means track jump or thumb grab; `Drag` means thumb movement; `Release` ends the grab; `None` is not a bar phase.
`bar_input_for` maps press, drag-start/drag, and click/release/drag-end to those values.
Backdrop `pass_through` is stored registry metadata; the flag alone does not implement lower-layer click delivery.
`SecondaryTracker::route_secondary` uses the separate `SecondaryOutside` policy. For a secondary backdrop press, `Reopen` returns `Outside` with an optional lower target; `Close` returns `Outside` without that target; `Ignore` returns `Swallowed` without dismissing. The caller decides layer mutation and input delivery; the tracker itself does not deliver a click to the lower component.
The intended automatic pass-through requirement remains open as `T15-PASS-THROUGH-01`.

Frozen source: [`hover-only region`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/hit.rs#L27), [`hover-only registration`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/mod.rs#L1147-L1190), [`scrollbar registry queries`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/hit.rs#L1300-L1354), [`Registry` hit and release identities](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/hit.rs#L1316-L1383), [`secondary release tracking`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/capture.rs#L119-L192), [`backdrop selection and bar phases`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/capture.rs#L204-L258), [`frame-local bar lookup`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/hit.rs#L1316-L1354).

## Binding publication and focus scope

Publish low-priority scope bindings for real focused controls. The runtime checks them after the focused control declines the same key.
`publish_bindings_filtered` keeps replacement semantics and adds only entries selected by `include`.
Use `extend_bindings` to add selected entries to an existing same-owner table without replacing it.
Extension rejects duplicate actions and ambiguous default chords with diagnostics; key-map remapping conflicts remain runtime diagnostics.
`NestedLayerSlotAuthor` is the low-level adapter for a clipped nested slot; ordinary callers obtain slots from `Ui::layer_slot`.
`Ui::command_scope` composes real controls; it creates no focus target or global key behavior, and accepts an empty table when no fallback exists.
`publish_additional_bindings` adds entries to the focused owner's current table and preserves hint metadata.
Render chord labels for the detected client platform. On macOS, `Super` uses `⌘`; other platforms use `Super+`.
Show only chords the terminal can deliver.

Frozen source: [`author bindings`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/author.rs#L15-L16), [`filtered publication and extension`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/author.rs#L212-L231), [`additional bindings`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/mod.rs#L1008-L1023), [`platform labels`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/keymap.rs#L832-L843), [`deliverable hint policy`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-shell/src/footer.rs#L26-L34).

## Layer opening and focus receipts

An `UpdateToken` identifies one application update callback within its runtime. Use it only to associate a component action with a close from that callback.
Treat `LayerOpening` as the identity of one exact opening, including runtime identity.
Layer completion follows replacement layout and settled focus. A focus-continuation outcome belongs to its exact opening and owner.
Consume a completion only after its focus restoration has settled. Observe a continuation without consuming it; acknowledge after applying resume or blur.
The layer owner must reconcile the exact focus-out transition and acknowledge that continuation.
`continues_focus_owner` keeps its live underlying control logically active while the layer owns input.
`focused_in_scope` returns only enabled registered focus within the scope on the top interactive layer.
Programmatic dismissal is never refused. Show a close affordance only when Escape can dismiss the layer.

`focus_continuation_opening` identifies the child opening for an exact pending owner-to-target transition. A layer viewport margin records total horizontal and vertical space reserved outside that layer.
Pending focus transitions retain their originating layer identity after that layer closes and leaves the new focus ring.
The runtime's internal `RoutedLayerEvent` envelope pairs each receipt with its exact intent identity.
By-ID receipt calls keep their existing behavior: they take the next receipt for that layer ID and match its next unclaimed intent.
Use the exact-opening receipt API when runtime and opening identity must match. A matching closed opening remains valid for its dismissal receipt.
`AlreadyClaimed` and `NotPending` retire only this opening's owned receipt; neither claims or recreates another intent.
A payload `Mismatch` leaves its receipt and queue unchanged.
Draw a layer only for the exact opening admitted to the frame. A stale renderer with the same ID cannot claim another opening's draw slot.
`Dismiss::pass_through` is not consulted by the runtime. `RefuseFeedback::Flash` names the one-time marker choice and `RefuseFeedback::None` is silent; the runtime does not render refusal feedback, and its default preserves no-feedback behavior.
A centered layer places an odd leftover row above the layer, matching Ratatui `Flex::Center`.

Frozen source: [`callback serial`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/cx.rs#L247-L254), [`balanced layer placement`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/layer.rs#L291-L294), [`refusal feedback and close affordance`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/layer.rs#L324-L430), [`layer identities and dismissal`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/layer.rs#L11-L15), [`focus continuation and receipts`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/cx.rs#L670-L845), [`layer viewport margin`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/layer.rs#L455-L466), [`layer receipt claims`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/cx.rs#L927-L981), [`exact-opening draw`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/mod.rs#L1448-L1466).

## Composition, windows, and row cells

Composition supports at most 128 nested groups. Greater depth returns `CompositionError::DepthExceeded`. The framework owns the root area and shares its resolved plan between leaf drawing and interaction registration.
`Canvas::put_text_w` uses its explicit width. `Canvas::put_text_in` clips to its supplied area and uses that area's width. Frame helpers return the border interior rectangle.
`Canvas::text_width` counts Rust scalar values and casts the count to `u16`; counts above 65,535 truncate. It does not measure grapheme or terminal-cell widths.
`Canvas::put_text` passes that count as a one-row area to `Ui::paint_str`, then returns the painter's actual horizontal advance.
`Ui::paint_str` clips to the UI clip. It skips controls and zero-cell clusters; the first cluster wider than remaining cells stops the run.
These helpers do not establish exact cell width or following-run placement for arbitrary Unicode. The intended content-derived cell-width and adjacent-run contract remains open as `T10-CANVAS-WIDTH-01`.
Owned leaves use the borrowed leaf measurement, geometry, clipping, and draw path. The group owns one box and drops the leaf with that group.
`VerticalWindow` maps logical source rows to physical rows without changing `Ui` coordinates or allocating an off-screen buffer. Its visible-source range is empty for zero-sized bodies; row conversions return `None` outside visible and logical bounds.
Clipped windows must be non-empty. Empty intersections and source ranges outside `logical_rows` return a typed error. Visible rows past the logical body and physical rows outside the terminal coordinate range also return typed errors.
`SizeWatch` starts with an optional known size. It records resize inputs and ignores other input kinds. It emits one resize only when actual dimensions differ, even without an initial size. It does not query the terminal or dispatch input.
Prepared layout facts can carry the visible physical rows of a taller logical body; they do not change `Ui` coordinates.
Each draw call draws each visible composition leaf once. Width changes can trigger another measurement callback.
`CompositionSlots::measure` shares draw's resolved-width remeasurement and is read-only; it does not draw leaves or dispatch input.
`PaintSurface` uses absolute terminal coordinates; its area defines the owned region. `PaintSurface::symbol` ignores positions outside that area and merges style channels using Ratatui cell rules.
`fill` and `style_area` use the requested and owned intersection. Unspecified channels inherit; `Color::Reset` resets ink, and removed modifiers apply through Ratatui cell merging.
`restyle` callbacks receive owned `CellView` values, not references to the backing buffer; `None` leaves each cell unchanged. A `CellView` stores a visible glyph and represented style patch; `fg()` and `bg()` return `Color::Reset` when unset, while `modifier()` returns added modifiers.
`PaintSurface::text` advances one cell per Rust scalar and saturates its x-coordinate at `u16::MAX`.
`PaintSurface::string` uses Ratatui grapheme and cell-width measurement.
An out-of-range `string` start skips the whole write; a valid start keeps measured widths and truncates at the right edge.
Component leaf measurement has no input handling or domain effects. Drawing receives logical bounds; `Ui` clips painting and registration to visible bounds.

Marker patches override only channels they set; the row resolver still chooses marker glyph and style.
Leading slots and resolved glyphs take precedence over fallback content. The leading width and gap remain reserved on narrow rows.
`trailing_part` draws and reserves its part before row content. `reserve_right` increases the right-side reservation with saturating arithmetic.
It paints nothing and consumes no left-side space. `begin_content` starts caller content while preserving its row-area contract.
Middle-truncated labels pad their logical slot with resolved label style and leave its final cell untouched. Positive slots are consumed even for empty text; a zero-width slot is a no-op. Truncation follows `truncate_middle`, including its narrow-slot behavior.
`label_end` keeps its final cell when truncation or `keep_row_tail` requires it; otherwise, a fitting label fills it.
Positive `label_end` slots are consumed even for empty text; one-cell areas are no-ops.
`keep_row_tail` is sticky and does not repaint an earlier cell. The padded-tail result remains true after later calls; it is not a paint rectangle or an instruction to erase the row edge.
Vacated cells from alignment use the initial row style by default.
`glyph_before_meta` uses the latest admitted metadata position, reserves no width, and is a no-op without admitted metadata, row height, or an in-row target.
It preserves the target background when the merged icon patch inherits background; explicit `Clear` and `Reset` remain authoritative.
The icon remains four logical cells before admitted metadata after labels consume the content budget.
Right-aligned metadata respects its configured gap and leaves one content cell available. Metadata records required width, admitted width, and logical x; omitted content has zero width, and x may fall outside the visible buffer. A scoped container patch applies independently of named-part overrides. A component row default is a fallback below authored recipes, state rules, global and scoped overrides, overlays, and the local instance patch.
Authored style values remain authoritative per channel even when they repeat the component default.
`RowUi::with_author_parts` limits component-author patches and slots to widget-owned parts; caller-owned metadata and arbitrary-cell parts keep their existing `RowUi` configuration.
`CellUi::available_width` excludes any resolved trailing-glyph reservation.
The scroll-part clear operation leaves cells the caller already painted untouched; the picker card samples backdrop ink first.
`suffix_style` sets the trailing glyph's base style. Deferred tone, modifier, and patch operations still apply.
Its saved patch overrides only channels it sets; unset channels keep the base style.

Frozen source: [`composition bounds and owned leaves`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/composition.rs#L29-L69), [`resolved-width remeasurement`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/composition.rs#L155-L175), [`measurement and draw remeasurement`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/composition.rs#L240-L274), [`vertical windows`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/author/window.rs#L5-L64), [`prepared vertical facts`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/cx.rs#L27-L63), [`SizeWatch`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/size_watch.rs#L1-L26), [`VerticalWindow row conversions`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/author/window.rs#L85-L124), [`composition root and measurement`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/author/composition.rs#L1-L13), [`scroll part painting`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/scroll_region.rs#L89-L153), [`row metadata and container patch`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/rowui.rs#L267-L280), [`component default precedence`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/rowui.rs#L1653-L1675), [`author row scope`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/rowui.rs#L469-L576), [`CellView and owned region`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/author/surface.rs#L19-L71), [`Surface cell operations`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/author/surface.rs#L76-L125), [`row cell rules`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/rowui.rs#L698-L1171), [`available width`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/rowui.rs#L1399-L1403), [`Canvas width and paint helpers`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/author/canvas.rs#L19-L100), [`Ui grapheme painter`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/paint.rs#L64-L129), [`command scope`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/command_scope.rs#L8-L27).

## Session input and cursor policy

`StartupInput::default` reads `JUNIE_TUI_START_DELAY_MS` and discards input queued before startup.
`delay(None)` disables the delay. `discard_pending(false)` keeps queued startup input.
The session adapter owns terminal queries and input dispatch.

The default `App::keyboard_protocol` honors explicit environment overrides and returns `Off` when neither exists.
An application return of `Auto`, `On`, or `Off` takes precedence over environment settings.
When `TERMROCK_KEYBOARD_PROTOCOL` is absent, policy lookup can use `JUNIE_TUI_KEYBOARD_PROTOCOL`.
A present canonical setting wins by presence; an invalid or non-UTF-8 canonical value resolves to `Auto` without legacy fallback.
`App::edit_entry` defaults to `EditEntry::Immediate`; text editors use that policy on focus or typing.
`Off` writes no protocol bytes. `Auto` probes; `On` pushes without probing.
Suspension pops only a protocol push owned by the session; otherwise it writes no bytes.
Resume restores an enabled suspended protocol once.

`SessionCursorShape::Auto` requests the baseline bar only when capability detection reports support.
Explicit `Terminal`, `Block`, `Underline`, and `Bar` choices select their startup command regardless of detected support. An explicit request does not prove terminal support or the visible cursor shape.
`Terminal` uses the terminal-configured default; the other choices request their named steady shape. Rendered bar requests remain capability-gated.
`run_with_cursor_shape` applies the explicit initial terminal cursor shape.
The clock paste protocol is active only when `JUNIE_TUI_TEST_CLOCK` is exactly `1`; its body is `junie-tui:advance-clock:` plus a positive decimal millisecond count.

Frozen source: [`startup options`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-session/src/session.rs#L91-L129), [`App keyboard policy`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/runtime.rs#L90-L105), [`session protocol and cursor choices`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-session/src/session.rs#L29-L55), [`startup and rendered cursor gates`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-session/src/session.rs#L332-L348), [`cursor-shape runner`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-session/src/session.rs#L564-L576), [`protocol suspend and resume`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-session/src/session.rs#L503-L511).

## Theme, focus, and pointer context

`Ui::with_theme` changes only the theme reference for its closure. The frame, buffer, and hit state stay shared.
Native layer coverage is independent of inertness; it excludes application overlays and detached references.
Live focus comes from the bound frame. Synthetic reference focus is presentation data, not input permission.
An inert context adds `INACTIVE_LAYER`; that flag defines no style. The theme's inactive look supplies its appearance.
`set_bar_cursor` requests a steady bar for presentation; callers do not write cursor bytes.
Retain a real rendered owner's cursor only through a supported nonmodal popover. The popover may own keyboard focus.
Cursor retention moves the existing request; it does not register another control or invent a cursor.
`set_undrawn_top_bar_cursor` requests a steady bar for an owner on the open top layer without drawing that layer.
Pointer click slop defaults to one terminal cell; the double-click interval remains configurable through `Theme::design.motion.double_click_ms`.

`fill_card_from_page` keeps dimmed page glyph color and sets the card background.

Frozen source: [`theme and layer state`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/mod.rs#L370-L457), [`cursor requests and retention`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/mod.rs#L1307-L1430), [`click slop`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/runtime.rs#L769-L772), [`card fill`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/mod.rs#L1682-L1692).

Low-color composition applies `dim` to every cell written by the layer while preserving foreground, background, and bold. Clearing `dim` leaves other modifiers unchanged; setting it does not clear bold or other modifiers.
The frozen source records the scroll track and popup border ink at low color levels. Theme authors own those per-slot colors.

Frozen source: [`layer composition`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/layer_buf.rs#L384-L389), [`dim modifier helpers`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/paint.rs#L583-L596), [`Canvas and Surface APIs`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/author/canvas.rs#L19-L100), [`App::edit_entry`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/runtime.rs#L135-L138).

## Retained runtime boundaries

`Runtime::handle_routed` returns the exact input while uninitialized, unpresented, or unsettled. Retry that same value after settling and presentation.
It also handles supported middle-button presses through the decoded event path.
Prepared-frame projection validates runtime, generation, and area. Acknowledge only after successful terminal presentation.
Cancel held text-drag autoscroll for the exact captured owner and part; physical pointer capture remains active.
After cancellation, a fresh pointer press may register another autoscroll gesture.
`Ui::register_text_drag_autoscroll` clips the body and requires an enabled `Part` registration for the same owner and part; vertical is required, and horizontal uses `Axes::H`.
`headroom` stops at content edges; change `source_revision` when mapping changes, even if visible geometry stays fixed.
Diagnostics are bounded and components cannot clear entries from other owners. Frame diagnostics commit with frame geometry.
Setting clipboard text copies it into runtime-owned storage and calls the optional integration hook with copied text. Clearing or replacing runtime clipboard text wipes its in-memory `Secret`.
The clipboard stores one `'static` copy hook; setting a hook replaces it, and clearing removes it.
Read clipboard text by borrow; `Cx::clipboard_text` does not transfer ownership.
The optional copy hook receives the copied text; the runtime does not select an operating-system clipboard API.
`Runtime::cursor_bar` reports whether the last presentation kept a steady bar; the session emits the cursor shape.
The private `next_callback_serial` uses `checked_add`; exhaustion returns `None` without changing the serial.
`position_label` reads the current visible logical range without applying pending layout or changing reveal or grab state.
It uses one-based rows and an inclusive end; 13 visible rows at offset 11 display as `12–24 of 120`.
It returns an empty string when content fits or the viewport has no rows.

Frozen source: [`input readiness`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/runtime.rs#L2175-L2184), [`prepared frame`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/runtime.rs#L1987-L2075), [`Runtime cursor state`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/runtime.rs#L3079-L3082), [`autoscroll registration`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/mod.rs#L1114-L1144), [`diagnostics`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/mod.rs#L455-L457), [`drag-autoscroll cancellation`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/ui/cx.rs#L613-L620), [`clipboard lifecycle`](https://github.com/tailrocks/terminal-components-claude/blob/c817b99e9e43abbbf3519d56b2f5ee86924243d8/crates/termrock-runtime/src/runtime/clipboard.rs#L7-L46).

## Evidence limits

All source links identify the pre-cleanup source revision. The artifact record separately verifies each attachment against accepted cleanup `7368d4f`.
This task ran no tests and selected zero behavior cases. Runtime behavior remains `NOT VERIFIED`.
