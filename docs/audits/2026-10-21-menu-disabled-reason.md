# Shared ContextMenu disabled reasons (WIP)

This audit records a bounded shared-widget implementation on PR #38's WIP
branch, based on read-only contract
`menu-disabled-reason-contract-20261021/contract.json` (SHA-256
`95a36ea84f66c7fa5bb5f7c0fb082f8259e10604475e3a4d0b01844d84fd660f`). The
frozen source is `visual-baseline` commit
`4169783ae883788186d161af09ba9c481bd84e5c`; no frozen artifact was modified.

`MenuItem::reason` borrows explanatory text, marks the item disabled, and is
independent of its executable chord and display hint. A later
`disabled(false)` re-enables activation and suppresses the reason. The shared
ContextMenu renders the selected disabled reason as a noninteractive `HELP`
decoration in its bottom border. It does not add a row or affect measurement.
The only theme change adds the muted semantic `MENU/HELP` recipe. Opt-in
`skip_disabled(false)` and `clear_on_leave(true)` model the frozen menu's
disabled-row keyboard and hover lifecycle while existing menus retain their
defaults.

Focused widget tests cover the borrowed reason/chord contract, unchanged
natural width, disabled pointer and keyboard activation, bottom-border
placement, all four color profiles, pointer-to-keyboard priority, clear-on-leave,
and `HELP` style/slot overrides without a hit target. Before the implementation,
the focused public API probes failed to compile because the reason API and
status decoration did not exist; after implementation the focused tests passed.
The pre-existing natural-width assertion remains exact (`16` cells), and passed
against the base and after the change.

The integration test `crates/termrock-overlays/tests/context_menu_disabled_reason.rs`
also compiles a consumer against public crate/runtime APIs with its own actions
and data. It verifies disabled-hover reason output, nonactivation from Enter or
the displayed chord, and reason clearing after keyboard movement.

Independent review of `c78e449b5d4f7f41f1f53d94260bde9aff2c068f` found two
additional menu-state cases. `MENU-REASON-01` was a new reason-selection bug:
a pointed disabled row without a usable reason hid a reason on the highlighted
disabled row. The public consumer regression was red before the fix. Reason
resolution now tests a nonempty disabled reason for the pointed item first,
then falls back to the visible highlighted item; reasonless and empty-reason
rows preserve the highlighted explanation.

`MENU-REASON-02` was a pre-existing shared disabled-style gap, present at the
WIP base and required by the component contract: disabled rows received
`HOVERED` and `PRESSED` flags, so a held press changed their full-row styling.
Frozen source evidence at `4169783ae883788186d161af09ba9c481bd84e5c` sets
`VisualState::disabled` before the shared `RowUi::resolve` path
(`src/widgets/menu.rs:688-705`, `src/ui/row.rs:95-105`) and withholds release
activation targets for disabled rows (`src/widgets/menu.rs:744-753`). The shared
ContextMenu now keeps hover and press styling off disabled rows while still
recording disabled pointer hover for its reason decoration. This intentional
shared disabled-state correction may change frames while disabled rows are
held; it does not change default enabled-row behavior.

Two real-runtime public-consumer tests failed on the pre-fix `c78e449b5` source:
reason fallback disappeared on hovering a disabled row without a reason, and
disabled held-press cells changed. The fallback consumer test now passes in
TrueColor; the held-press/drag/release cell probe passes in all four theme
profiles, with no activation. Existing reason-rendering profile coverage also
passes in all four profiles. These tests are source regression probes, not
baseline artifact approval.

At the final fix tip, `cargo nextest run -p termrock-overlays --all-targets
--no-fail-fast` passed 134/134, `cargo nextest run -p termrock-theme --lib`
passed 94/94, scoped all-target Clippy passed with `-D warnings`, and scoped
formatting plus `git diff --check` passed.

Before the separate padding fix, the unfiltered overlays library suite reported:
`menu::tests::default_menu_bar_scope_keeps_title_patches_off_the_padding_cells`.
It reproduces at this WIP's base `6841e67ed4d240d9b39478b2e99f39ff48b8189b`,
which contains incoming commit `262ac80ed138f14a204f8a1a32d4a48fe0cde111`.
An independent before/after run confirms it is introduced by `262ac80`:
before that commit, 125/125 overlays tests passed; after it, 124/125 passed and
this was the sole failure (logs at
`/private/tmp/incoming262-review-20261021/{before,after}-overlays.log`). Review
traced it to `262ac80`'s `MenuBar::draw` change: padding was
added to the label string and received the `Part::TITLE` style under the default
title-only hover policy, violating the assertion that the default title patch
applies only to title cells. A separate authorized fix restored painting the
label through its inset rect, keeping padding outside the title part. The exact
regression test was red at the WIP base and green after the fix; the complete
80-test overlays library suite, scoped Clippy, and formatting all pass. This
records the 262-introduced issue as fixed on this WIP, not as globally accepted
until integration and review.

**Not accepted as application parity:** Jackin/Capsule adoption, end-to-end CLI
comparison, full frozen flow inventory, resize/reanchor behavior, all-disabled
and zero/narrow Unicode matrix, independent review, full workspace gates, and
PR #38 integration are still outstanding. This WIP is only a shared component
increment. The branch does not claim the full product or library acceptance
gates.

## Shared underlay renderer follow-up (WIP)

This follow-up implements the approved shared-menu contract in
`menu-underlay-shared-contract-e4eae58e-20261021/contract.json` (SHA-256
`e432896194697d05d3f1c63d93716e212ed1958b1707ffcbba6c146399bd9440`). The
parent's immutable local merge `ed051a90f77a35dc15554c42436c0cad6fe11ee6`
was merged normally to carry already-reviewed concurrent source work; this
follow-up's edits remain limited to ContextMenu, its focused consumer test, and
this audit.

At the four unchanged public probes in
`incoming-e4eae58e-independent-review-20261021/review.json`, the pre-fix source
failed `underlay_keeps_chord`, `underlay_keeps_disabled_style`,
`underlay_honors_border_patch`, and `underlay_keeps_underline_ink` (4/8 passed,
4 failed). After this menu-only change, the three menu behavior failures pass.
The reported nested lower-layer foreground pass is invalid evidence: its
fixture ignored an error from `Runtime::handle(Tick)` and never opened the upper
layer. Do not credit that result for the nearest-lower-layer contract. The
unchanged runtime-only `underlay_keeps_underline_ink` still fails because
`Ui::fill_keep_ink` clears the underline channel; its runtime owner is handling
that separately. No runtime source was edited here, and this menu change does
not claim that check is green.

ContextMenu now has one row renderer for default and underlay modes. Both modes
resolve container, border, title, rule, row, marker, label, key, and help through
the existing MENU family, `PartStyle`, slots, effective chord and submenu
paths. `keep_underlay()` only selects background-only base fill; it no longer
chooses row colors or content. The separate generic `separator_inset(u16)`
policy defaults to zero and drives the same rectangle for the RULE glyph/slot
and registered decoration in every fill mode. Frozen source evidence shows a
one-cell separator inset in the Capsule menu; a real application consumer still
needs to configure that policy explicitly. No app integration or frozen
comparison is included in this increment.

The new real-runtime public integration test
`crates/termrock-overlays/tests/context_menu_underlay_shared.rs` passes 7/7.
It covers shared chord and submenu output, disabled appearance over all four
color profiles, global and per-part overrides for container/border/title/row/
label/key/marker/rule/help, slots in both fill modes, mode-independent RULE
geometry, explicit inset geometry, the shared keyboard action path, and
narrow-width resize safety from zero through 40 columns. Package validation is
`cargo nextest run -p termrock-overlays --all-targets --no-fail-fast` (141/141),
`cargo clippy -p termrock-overlays --all-targets -- -D warnings`,
`cargo fmt --check --package termrock-overlays`, and `git diff --check`, all
green. The external consumer's nested-layer result is not credited; Runtime
review also records an admitted nearest-lower-layer failure and an in-place
`ForcedWidth`/ordinary-space metadata failure. Runtime review status is HOLD,
not acceptance. Product parity, actual Capsule configuration, and the frozen
CLI comparison remain outstanding.

## Independent review correction (2026-10-21)

The immutable review
`menu-underlay-7ab3d12e-independent-review-20261021/review.json`
(SHA-256 `54a0994471b177e1d91c04719282ea407666c8927b3dcafa1e91f5c07c98ab5b`)
found that `clear_on_leave(true)` hid the row highlight but Enter/Space still
activated the stored cursor. The menu now consumes these keyboard activation
bindings while the highlight is hidden; pointer activation remains unchanged,
and keyboard navigation restores the highlight before activation. The frozen
source at `4169783ae883788186d161af09ba9c481bd84e5c:src/widgets/menu.rs:463-472`
guards unmodified Enter/Space before the ordinary character accelerator
dispatch. The guard blocks only those physical keys; ordinary letters remain
eligible because they do not match Enter/Space. The real `Harness` regression
failed before the initial fix with `Chosen(RUN)` after pointer leave. A follow-up
regression also reproduced physical Enter/Space remapped to item bindings
bypassing the command guard. ContextMenu now checks the runtime's
`Cx::activation_key()` before dispatching any binding while its highlight is
hidden, preventing those physical-key bypasses. Alternate typed activation
commands, ordinary letter item shortcuts, and pointer activation remain
available. Both remapped physical-key cases were red before this follow-up and
are green with the normal Enter/Space, Home-restoration, and ordinary shortcut
cases. This is a shared widget contract fix, not baseline artifact approval.

The same review rejects the earlier nested lower-layer pass because the old
fixture swallowed a rejected `Tick` and never opened `UPPER`. The independent
Runtime 860 review
`runtime-underlay-860529a3-independent-review-20261021/review.json` (SHA-256
`750216794edc36d3d981fdff9edd3f6ec6cbea0d51d2eb95d99059121abc51df`) confirms
the strengthened admitted upper-layer fixture is red and separately records
the in-place `ForcedWidth(4)` space metadata defect as red. Neither Runtime
finding is in this menu-only scope; both remain unresolved and no nested
underlay correctness is claimed here.

## Independent review correction (2026-10-21, follow-up)

The immutable review
`menu-underlay-4a39a666-independent-review-20261021/review.json` (SHA-256
`3a96dfca5624d864c42099672d64949a55f18796e6e99d8838c818138e2cf7ea`)
corrects the previous paragraph's source-order statement: frozen source lines
463-465 consume Enter/Space before lines 468-472 inspect ordinary character
accelerators. Ordinary letters are permitted because they do not match the
physical activation-key guard, not because the accelerator lookup occurs first.
The runtime's `Cx::activation_key()` identifies the unmodified physical key so
the guard remains effective when Enter or Space is remapped to an item's action.
An alternate remapped activation command uses another physical key and remains
available; pointer activation is unaffected.

The same review records an inherited Mono visual-state gap: with the current
MENU DEFAULT recipe, the enabled row's active and clear-on-leave hidden cells
are identical in Mono despite the hidden state blocking activation. This was
reproduced on both `7ab3d12e` and `4a39a666`; no theme or palette change is made
here. All-profile visual parity is therefore incomplete pending an independently
reviewed shared-theme/component resolution. Existing 4-profile frame checks and
the unchanged default-frame digest do not close this state-specific gap.

The prior push-lock owner line contained a quoted, unevaluated `$(uuidgen)`
placeholder; that lock was safely released after publication. Subsequent lock
owners must use actual UTC, PID, and a generated UUID, and verify the exact owner
token before removing the lock.

The follow-up implementation applies `Cx::activation_key()` at the shared
binding-dispatch boundary before either command or dynamic item-action routing.
This blocks unmodified physical Enter/Space while a highlight is hidden even
when those keys have been remapped to an item action. A regression reproducing
both Enter→RUN/activation→F9 and Space→RUN/activation→F8 was red before the
change and passes after it. Separate real-runtime checks confirm the alternate
typed activation commands, ordinary `x` item shortcut, and pointer click still
activate the row. Exact source-order correction is as stated above: frozen
416 checks Enter/Space before ordinary character dispatch.

The all-profile Mono visual-admission case remains deliberately unresolved.
The current hidden row is mechanically nonactivatable, but its Mono DEFAULT
cells do not visibly differ from the active row. No global palette or widget
default was changed; the 4-profile visual-state acceptance gate remains
incomplete until an independently reviewed shared-theme/component fix lands.

The next independent review
`menu-underlay-35cfdd8a-independent-review-20261021/review.json` (SHA-256
`4038a01428ea800f0ea56a8b37bbcef1f5baf3c95d8e52b7e9fa999ff8342c81`)
identified one remaining side effect: a physical activation key remapped to a
navigation action was consumed, but still ran `after_keyboard_navigation` and
revealed the hidden row. The menu now performs the physical-key guard before
both binding dispatch and its navigation-restoration side effect. Two public
`Harness` regressions cover Space/Enter remapped to First, Last, Up, and Down,
including the follow-up physical activation check; both were red before this
change and pass after it. Alternate activation commands, ordinary letter
shortcuts, pointer activation, and normal Home restoration remain covered and
green. The same audit corrects the prior review checksum to the immutable
64-character SHA-256 `3a96dfca5624d864c42099672d64949a55f18796e6e99d8838c818138e2cf7ea`.
