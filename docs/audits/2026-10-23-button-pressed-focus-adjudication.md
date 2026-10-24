# Button pressed-vs-focus adjudication (2026-10-23)

## The conflict

Does a pressed focused Button keep the `▎` focus gutter? Two committed, opposing
answers, from independent sessions, on `parity/termrock-visual-baseline`:

- **Side A — `d8b2c0eb1b`** ("test(conformance): align W02-01 focus-gutter
  expectations with the FOCUS_VISIBLE recipe", 2026-10-22 04:17 +0700, lineage
  `0648c8a65d`, not merged to the parity tip): upholds
  `ButtonFocusVisibility::Always` + FOCUSED paints `▎` with no pressed
  exception, per the recorded contract
  `docs/audits/2026-10-21-button-shared-contract.md`. Four `button_contract`
  tests in `crates/termrock-controls/tests/button_contract.rs` pin FocusBar
  through the real Harness (pointer focus, inert scope, all variants/levels).
- **Side B — `37f55475f7`** ("fix(conformance): achieve 100% control-state and
  baseline parity", 2026-10-21 23:25 +0700, on the parity tip): added
  `&& !style_flags.contains(StateFlags::PRESSED)` to the `Always` promotion in
  `crates/termrock-controls/src/button.rs`, so a pressed focused button stops
  resolving `Part::GUTTER` to `GlyphRole::FocusBar`. The same commit reblessed
  `crates/termrock-conformance/tests/baselines/components.txt` (2120 lines,
  including all eight `render::components::button::pressed` digests) and
  reversed the W02-01 expectation text — leaving the 4 `button_contract` tests
  red.

The conflict was recorded and stopped for adjudication in
`docs/audits/2026-10-22-showcase-false-parity-audit.md` and
`docs/handoff/README.md`. This document is the adjudication.

## Recorded rationale, both sides

Side A: the commit message and the contract audit both record the rule — the
gutter recipe keys on `FOCUS_VISIBLE`; `Always` "only promotes the runtime's
existing eligible `FOCUSED` state to `FOCUS_VISIBLE`; it never manufactures
focus, promotes disabled/busy controls, or promotes a control drawn inside an
inert reference scope." No pressed exception is listed or anywhere recorded.

Side B: none. `37f55475f7` has a one-line message with no body; no audit doc,
matrix row, or decision record names a rationale. Its only traces are the code
change, the reblessed digest baseline, and the reversed expectation text. The
`2026-10-22-showcase-false-parity-audit.md` record shows the same session's
conformance green was achieved *by* those expected-output edits while the
contract tests stayed red — i.e. the expectations were fitted to the code, not
the code adjudicated against the expectations.

## Standard analysis

- **WCAG 2.2 (repo law adopts WCAG 2.2 AA).** SC 2.4.7 Focus Visible requires
  the keyboard focus indicator to be visible whenever focus is present. A
  keyboard user activating with Enter/Space is pressing the button at the
  moment they most need the indicator; suppressing the bar during press
  removes it exactly then. Nothing in the pressed state ends focus.
- **WAI-ARIA.** `:focus`/`aria-focused` and `:active` are independent states;
  activation state never retires focus indication.
- **Apple HIG (repo's first reference).** A pressed macOS control keeps its
  focus ring; press is conveyed by the control's own pressed appearance (here
  the pressed fill/modifiers), not by removing the focus indicator.
- **This repo's own law (AGENTS.md, Visual states).** "Exactly one
  keyboard-focus indicator is visible at a time" — suppressing the bar while
  pressed leaves zero indicators while the button still holds focus. "Rest,
  hover and pressed are visibly different from each other, and from focused"
  — distinctness is already carried by the pressed fill and modifiers; it
  does not require deleting the gutter. The same law's precedent
  (`InactiveSelection::Accent`, owner decision 2026-10-17) keeps state marks
  on when focus moves; only a modal or popup neutralizes them, and a press is
  neither.

No governing convention supports Side B's suppression.

## Frozen-oracle analysis

The visual-baseline tag is `4169783ae` ("Visually correct target version and
baseline reference"). Its source settles it:

- `src/theme.rs` computes the bar as
  `let bar = s.focused && !matches!(kind, Kind::CursorRow(CursorStyle::Fill) | Kind::Close | Kind::Field)`
  — focused paints the bar; PRESSED appears nowhere in the exception list.
  Press changes fill and modifiers (`pressed`, `pressed_fill`), never the
  bar.
- `src/widgets/button.rs` paints the gutter unconditionally when the theme
  says there is a bar (`buf.set_string(area.x, area.y, it.gutter, gutter)`),
  and its own test `keyboard_only_focus_hides_the_ring_after_a_click` asserts
  `FocusVisible::Always` keeps the bar even with pointer focus
  (`assert_ne!(draw(FocusVisible::Always, true), rest, "Always keeps it")`).
- The frozen buttons page states grid (`states_grid`) specimens Rest, Hover,
  Focused, FocusedHover, Pressed, Disabled, Selected, Busy — there is no
  FocusedPressed frame in any frozen snapshot, so no frozen picture shows the
  pressed-suppressed variant. The oracle's answer lives in its code: pressed
  focused buttons keep the `▎`.

Side B's pressed exception therefore changes the visual language of the
frozen baseline — a deliberate visual change that was never called out, in
violation of the repo's own rule.

## Additional finding: the inert-scope removal inside `37f55475f7`

Side B's diff also silently removed `&& !ui.is_inert()` from the same
promotion. That removal is unrelated to any pressed question and contradicts
the recorded contract ("never promotes a control drawn inside an inert
reference scope"); it broke
`always_policy_does_not_promote_focus_in_an_inert_reference_scope` and
`owner_wide_reference_styles_every_part_without_live_registration_or_cursor`
(the part-style assertions resolve without the promoted flag). This was a
contract regression hidden inside the conformance-green commit.

## Verdict

**Side A stands.** The frozen oracle's source, WCAG 2.2 AA, WAI-ARIA, Apple
HIG, and the repo's own visual-state law all agree: a pressed focused button
keeps the `▎` focus gutter; press is carried by the pressed fill and
modifiers. The owner-discretion route is not reached — the oracle and repo law
decide it. Side B's pressed suppression is reverted as the standard-conforming
repair, together with the unrelated inert-scope removal smuggled into the same
expression.

## Required repairs (applied in this change)

1. `crates/termrock-controls/src/button.rs`: the `Always` promotion is
   restored to the contract form — `!ui.is_inert() && !disabled && !busy() &&
   FOCUSED` — removing `!style_flags.contains(StateFlags::PRESSED)` and
   restoring `!ui.is_inert()`. This is the minimal standard-conforming fix
   identified by the review.
2. Failing-first proof: at tip `e2ba67866e` (Side B's behavior),
   `cargo nextest run -p termrock-controls --test button_contract` = 19 run,
   15 passed, 4 failed (`pointer_focus_visibility_obeys_always_and_keyboard_only_policies`,
   `always_policy_does_not_promote_focus_in_an_inert_reference_scope`,
   `focus_visibility_policies_hold_for_every_button_variant_and_color_level`,
   `owner_wide_reference_styles_every_part_without_live_registration_or_cursor`);
   with only the pressed guard removed (inert guard still missing) the two
   inert-scope tests still failed; after both restorations the three
   pressed-conflict failures are green (`owner_wide` remains red for the
   separate reason recorded below).
3. W02-01 repair (second half). The tip's W02-01
   (`w02_button_variants_all_states_and_focus_gutter`,
   `crates/termrock-conformance/tests/control_states.rs`) looked untouched by
   the conflict — its FOCUSED leg expects `▎` and passes at tip — but that
   pass depends on the inert-guard removal: every W02-01 leg draws inside
   `ui.reference`, and `Ui::is_inert()` is true for any reference scope, so
   under the recorded contract's inert rule (`Always` never promotes inside an
   inert reference scope) a reference carrying only FOCUSED must paint no
   gutter. Side A's lineage confirms the dependency: its parent `0648c8a65d`
   already contained `37f55475f7`'s promotion changes (pressed guard added,
   `!ui.is_inert()` removed), so Side A's revised W02-01 was authored — and
   green — on top of the guard-removal it never adjudicated. With both guards
   restored, two spots re-red and are corrected to the contract's channel:
   the FOCUSED-only reference leg now expects `" "` (the discriminating leg,
   reference carries FOCUS_VISIBLE → `▎`, is unchanged; live pointer/keyboard
   focus promotion stays pinned by the `button_contract` Harness tests), and
   the "focus must add to hover" probes use a FOCUSED|FOCUS_VISIBLE|HOVERED
   reference instead of FOCUSED|HOVERED. Like Side B's edits, Side A's W02-01
   expectations were fitted to unadjudicated code rather than to the recorded
   contract; this repair aligns them with the contract.

## The components.txt rewrite is called out

`37f55475f7`'s 2120-line rewrite of
`crates/termrock-conformance/tests/baselines/components.txt` — including the
eight `render::components::button::pressed` digests reblessed to the
suppressed-gutter frames — was premature whoever lost: expected stores may not
be edited before the behavior they encode is adjudicated. The parity tip has
since already undone that rewrite: `4d7a358d3c` ("fix(conformance): preserve
expectations pending reference review", 2026-10-22 04:33 +0700) restored the
pre-`37f55475f7` digests, so the reblessed values no longer exist on the
branch. Two consequences, verified by paired runs:

- The `render_components` digest reds for
  `render::components::button::default` (baseline `7cad0003dd9cd1fb`, got
  `2c4fd377d36d6c75`) and `render::components::button::pressed` (baseline
  `c04d0deb30b15747`, got `1e7c2be1435e6d13`) exist at tip and produce the
  **identical** got-digests with this adjudication's repair applied — they
  are pre-existing drift from `d1ac0c0dac`'s pad painting and pressed-palette
  change (the same change behind the `owner_wide` contract red above), not
  from either side of this conflict. The `button::pressed` digest scenario
  paints a pressed-only (unfocused) reference, so it never distinguished the
  two sides' behaviors in the first place.
- `37f55475f7`'s W13-02 grapheme expectation reversal (`"a"` vs
  `"a👩\u{200d}"`) was likewise an expected-text edit made before
  adjudication; it was subsequently reworked by the conformance lane
  (`bb4bbb6eb9`) and is out of scope here, recorded for the same reason.

## Separate defect found during evidence (not this adjudication)

One `button_contract` test stays red after the repair and is **not** caused by
the pressed conflict: `owner_wide_reference_styles_every_part_without_live_registration_or_cursor`
(`button_contract.rs:387`, `part style at x=2`, actual white vs expected
`0x112233`). Probe: with the whole pre-`37f55475f7` `button.rs` in this tip
tree the test passes; with only this adjudication's promotion revert it fails,
identically to tip. The remaining difference is `d1ac0c0dac` ("fix(showcase):
match the buttons audit chrome", 2026-10-22 00:58 +0700), which rewrote the
Button pad painting (`ui.fill(area, pad)` with pad fg = container bg, trailing
label-ink pad) to match the frozen 100x30 buttons audit frame. That repaint
changed what an owner-wide `Part::CONTAINER` patch paints on pad cells and no
longer satisfies this contract test. That is a distinct conflict between
`d1ac0c0dac` and the button contract — a deliberate paint change for the
frozen audit chrome against a recorded part-patch contract — and it stopped
here for its own adjudication; it was neither adjudicated nor repaired in this
lane.

## Gate results for this repair

- At tip `e2ba67866e` (Side B's behavior), before any repair:
  `button_contract` 19 run, 15 passed, 4 failed.
- After this repair: 19 run, 18 passed, 1 failed — the single failure is the
  separate `d1ac0c0dac` defect recorded above (all three pressed-conflict
  failures are green).
- Conformance `control_states`: 107 passed / 1 skipped after the W02-01
  corrections (red with the Side A expectations under restored guards —
  FOCUSED-only reference leg and the FOCUSED|HOVERED hover probes —
  failing-first proof for those corrections).
- Conformance `render_components` `button::default`/`button::pressed`
  digests: red at tip before this repair and red after it with byte-identical
  got-digests (paired runs recorded above) — pre-existing `d1ac0c0dac` drift,
  preserved by `4d7a358d3c` pending reference review; not touched here and no
  BLESS was run.
- `cargo clippy -p termrock-controls --all-targets` and `cargo fmt --check`
  clean on the touched file.
