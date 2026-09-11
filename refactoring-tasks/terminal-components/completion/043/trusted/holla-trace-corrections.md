# Holla source-correct interaction branches

This normative expansion amends HO-PREVIEW-SELECT, HO-ARGS and HO-OVERLAY-PASTE without changing their primary IDs or deleting any important behavior. Oracle authority is `02f5294bfdbf38004cc49130d0aff1d01f31434c`. The rejected earlier interpretations remain in `reaudit-holla.md`. TASK-003 must materialize every branch below into separate immutable action/checkpoint identities. No branch may pass from its title, a static frame, or a direct-model continuation after process exit.

All branches run independently at 80×24, 100×30, 120×40 and160×50. Existing palette, full-frame, cursor, semantic-state and native/resized requirements still apply. Coordinates are extracted once from the oracle, then frozen literally for candidate replay. Every original source-native test remains unchanged.

## HO-PREVIEW-SELECT — TASK-042 complete ownership

`BROWSER-TEXT(path)` means NEW(parity-browser,Reduced,0); TICKS(4); OPEN(Browse ~/work/site); select the exact stable file path through the actual list; TICKS(1); Right to its preview. The source list supplies both the path and its oracle hit rectangle. If a row needs reveal at a size, use bounded Down/PageDown from the original list before freezing the pointer. Do not navigate by candidate text search. Use a fresh constructor for each branch/path.

1. **preview-copy-find-live:** BROWSER-TEXT(/Users/alex/work/site/index.html); Ctrl+Shift+Home; `y`; `/`; TYPE(hello); Enter; Down; Up; Backspace×5; TYPE(zzzz-no-preview-match); Enter; Down; Up; Escape. Assert a nonempty selection before `y`; exact copied payload `    1 <html>\n    2 <body>hello</body>\n    3 </html>`; clipboard generation increments exactly once; no quit; find shows a positive count for hello and zero for the missing query; next/previous wrap and zero-match inputs obey source behavior; Escape clears find and restores preview ownership. Capture every action boundary. The numbering belongs to oracle viewport text and must not be stripped.
2. **preview-selection-sweep:** for every actual text preview in the existing source inventory, BROWSER-TEXT(path); perform the original complete SWEEP; restore the fresh preview; Ctrl+Shift+Home; `y`. For nonempty selected text require its exact source payload and selection/scroll/cursor state. For a source-empty preview retain the explicit no-selection/no-copy result. Preserve all original file-safety, wrapping, selection, scrolling and clipboard-refusal applicability; do not claim a nonempty selection from Shift+Right/Shift+Down at the default end caret.
3. **preview-ctrl-c-exit:** BROWSER-TEXT(/Users/alex/work/site/index.html); Ctrl+Shift+Home; Ctrl+C; stop the program. Assert quit=true in the direct lane, no copy event, and actual executable exit/terminal restoration in the PTY lane. There are no post-exit `/` or typing checkpoints. A driver that continues calling the direct App after quit cannot certify executable reachability.
4. **preview-end-selection-noop:** BROWSER-TEXT(/Users/alex/work/site/index.html); Shift+Right; Shift+Down; `y`. The fresh viewport's default browse caret is at the final visible line's end. Freeze the actual no-selection/no-copy outcome separately; it never satisfies branch1's selection assertion.

Sources: app.rs::on_key at575–578; screens/files.rs::on_key at1163 and viewport event handling at1224; widgets/viewport.rs::browse_caret at1182, on_click at1092, on_key at1352 and plain-y copy at1386. A pointer click sets a drag anchor and clears the keyboard browse caret; it does not establish the keyboard selection anchor.

## HO-ARGS — TASK-043 contributions, TASK-048 complete ownership

`PORT-EDIT` means NEW(rust-dirty,Paused,0); TYPE(port); Enter; click the first text cell of the oracle `ARG.child(0)` hit (`area.x+2, area.y`); require focused editing and initial value5173. The pointer is source-frozen separately at each size. Preserve this exact initial value; it is not an empty field.

1. **args-insert-fallback:** PORT-EDIT; PASTE(5173); Ctrl+S. Require raw value51735173, emitted kind `port:51735173`, rendered title `Port 5173`, and exact displayed lsof command/follow-up identities from the oracle. This is the previous insertion trace, now accurately classified as fallback behavior rather than a successful replacement.
2. **args-valid-replacement:** PORT-EDIT; Ctrl+L; PASTE(5173); Ctrl+S. Require raw value5173, emitted kind `port:5173`, exact command/cwd/host and all resulting Snapshot frames. Ctrl+L is the source select-all chord; Ctrl+A moves to Home.
3. **args-required-empty:** PORT-EDIT; Ctrl+L; Backspace; Ctrl+S; Escape. Require empty value, `Required` and `Fill the required fields first`, unchanged Args route, no Snapshot/action, and source-defined Escape edit cancellation. Do not assert that one Escape closes the whole Args page while editing.
4. **args-nonnumeric-fallback:** PORT-EDIT; Ctrl+L; PASTE(abc); Ctrl+S. The source field validates required nonemptiness only: require emitted `port:abc` and the same source u16 fallback display5173. Numeric validation is not permitted as a refactoring improvement.

The protected observation schema must separately retain raw field value, selected range/caret/editing state, emitted Snapshot kind, displayed parsed port and exact command/cwd/host. Equal rendered titles do not prove equal arguments. HO-043-F02 contains each successful branch prefix immediately before its Ctrl+S, plus the complete required-empty branch; all successful Snapshot descendants remain in the full048 parent.

Sources: domain/catalog.rs:2277–2305; widgets/input.rs:177–185,237–274; widgets/field_common.rs:112–118; screens/review.rs:999–1013; app.rs:2600–2606; screens/snapshot.rs:337–342.

## HO-OVERLAY-PASTE — TASK-043 complete ownership

Use fresh PORT-EDIT for each named branch, then its exact chord, PASTE(café), Escape. Preserve the source raw value and caret throughout; no generic overlay ownership rule may replace the separate source handlers.

- **paste-help-isolated:** F1 opens the true Help modal; paste is consumed; Escape dismisses it; underlying Args value remains5173.
- **paste-picker-isolated:** Ctrl+G opens the true Activities picker; paste changes only its own query; Escape dismisses it; underlying Args value remains5173.
- **paste-quit-isolated:** Ctrl+Q opens the true Quit modal; paste is consumed; Escape cancels; underlying Args value remains5173 and quit=false.
- **paste-menu-passthrough:** F10 opens MenuBar, which is not a modal; paste reaches the already editing Args field; Escape closes only the menu. Underlying value becomes `café5173` with exact cursor/selection state; menu state and focus restoration remain oracle-equal. This source quirk is required parity.

`PORT-IDLE` means NEW(rust-dirty,Paused,0); TYPE(port); Enter, with no field click, edit key, or direct seed. Args.enter has already focused ARG.child(0), but editing=false and the value is5173. Independently repeat all four overlays from this distinct state:

- **paste-help-idle-isolated:** PORT-IDLE; F1; PASTE(café); Escape. Args remains focused, idle and5173.
- **paste-picker-idle-isolated:** PORT-IDLE; Ctrl+G; PASTE(café); Escape. Only the picker query changes; Args remains focused, idle and5173.
- **paste-quit-idle-isolated:** PORT-IDLE; Ctrl+Q; PASTE(café); Escape. Args remains focused, idle and5173; quit=false.
- **paste-menu-idle-begins-edit:** PORT-IDLE; F10; PASTE(café); Escape. The shared field begins editing, inserts at the default end caret, and retains exact raw value `5173café`; Escape closes the menu, not the new edit session. Assert editing=true and exact draft/committed-value/selection/caret state at every boundary. A helper that rejects all nonediting targets is source-incompatible even if the click-started branch passes.

The main architecture may implement both MenuBar branches through an explicit application compatibility command into the shared field's configured paste helper. It must use the same editor operation as the normal input path, including auto-begin for an enabled editable idle target, commit-only value, validation and response behavior; no application-local text insertion, fake focus, hidden runtime delivery or weakened true-modal capture is permitted. Shared runtime and field ownership remain intact.

Sources: app.rs::on_paste:480–498, on_key:553–565; screens/review.rs::on_paste:1113–1125 and enter:1127–1129; widgets/input.rs::on_paste:237–245. Keyboard MenuBar capture does not imply paste capture; focused idle fields are paste targets and begin editing.

## Qualification and evidence

The protected runner must reject each old false interpretation before accepting its corresponding corrected branch: copy-plus-post-Ctrl+C continuation, claiming a nonempty selection from an end-caret no-op, equating fallback title with accepted raw5173, requiring numeric validation absent from source, and universally blocking F10 paste. Require actual production handler/view binding, nonempty effect predicates where claimed, and real terminal process lifetime checks.

Disposable direct replay uses `/tmp/holla-reaudit.5pIf5v/src/main.rs`, linked to88 byte-verified unchanged oracle Holla/library source files. It exercises the old failure mechanisms and corrected valid paths at all four sizes. Its result establishes these source behaviors only; it is not a sealed production baseline, full SWEEP capture or replacement for the required TASK-003/070 qualification. See `reaudit-holla.md` for measured results and remaining lane limits.

Independent idle-state rereplay extends that disposable driver without editing any oracle source. All four sizes passed all eight overlay branches: PORT-EDIT menu requires the full `café5173` string; PORT-IDLE menu requires the full `5173café` string and EDIT indicator after menu dismissal; every idle true-modal branch retains5173, no EDIT indicator and quit=false. The source transition is `Args.enter -> focused ARG0/idle -> F10 -> TextInput.on_paste -> begin_edit -> insert_str -> Escape menu dismissal`. The old precondition-only editing tests could not detect a helper that wrongly ignored idle targets. Current replay driver SHA256 is `946b999e2495aca0204ca5db20ecf8d08d7ce02bf2086fa7bfb0c24ca9817a59`; compiled disposable executable SHA256 is `5ceb559b03bdf2a6c9b2328dbae44c723724eb48c2c292375e47b18602b2f302`. These are actual App/frame assertions, not protected candidate-baseline receipts or private-field instrumentation; the eventual branch schema still must retain the exact draft/committed/caret states required above.
