# Shared theme menu recipe change (`afb602936`): deliberate decision record

Date: 2026-10-22. Branch: `parity/termrock-visual-baseline`. Docs only; no
product source or expected-output change in this record.

## The change

Commit `afb602936` ("fix(jackin): match the 120-col capsule tab menu")
modified the shared `menu` recipe in
`crates/termrock-theme/src/builtin/mod.rs`:

- `Part::TITLE`: was theme default; now `Fg(Muted) + BOLD` (the `PRESSED`
  press-glyph rule is unchanged).
- `Part::KEY`: was `Fg(Muted)`; now `Fg(Secondary)`, with a new `ACTIVE`
  rule setting `Fg(HighlightFg)` and removing `BOLD`.

Verified against `git show afb602936 -- crates/termrock-theme/src/builtin/mod.rs`.

## Decision: DELIBERATE and SHARED — keep, do not revert

The recipe change is recorded as a deliberate shared-layer decision, not a
jackin-local accident:

- The demanded consumer is the jackin capsule menu: the frozen
  `visual-baseline` frame the commit targets requires the muted bold title
  and the secondary/highlighted key ink. Reverting the recipe re-breaks that
  frozen frame.
- Collateral consumer: holla's `MenuBar` (`crates/holla-ui/src/app.rs:24`
  imports `MenuBar`/`Part`; `crates/holla-ui/src/app.rs:197` defines
  `fn menu_bar() -> MenuBar<'static>`). Its title/key ink renders through
  the same shared recipe, so every holla digest row touching that ink
  drifted. Holla's store independently went red at `afb602936` — the same
  change, the same window, computed `83a32521d5113b5d` vs expected
  `e30768d9339a0fb9` on `first-use 80 24 junie truecolor` (bisect evidence:
  `docs/audits/2026-10-22-holla-reference-regeneration-case.md`).
- Blast radius: tablepro has no `MenuBar` surface (no `MenuBar` use in
  `crates/tablepro*`). Showcase demonstrates one `MenuBar` through the same
  shared recipe (`crates/showcase-demos/src/pages/pickers.rs:364`), so its
  visual expectations cover the recipe as well; no other app-level menu-bar
  surface exists.

## Process defect at landing

The change landed through a jackin-only commit message
("fix(jackin): ...") with no record of the cross-app effect on the shared
recipe, and it stayed unrecorded until the 2026-10-22 audits (the holla
reference-regeneration case named it first via bisect). A shared-layer
theme change must be recorded as shared at landing; this one was not. The
defect is in the process, not the pixels — see the regeneration-case audit,
"Reference defect recorded".

## Follow-up correction: `849d26dad`

`849d26dad` ("fix(theme,overlays): restore frozen SQL syntax ink and menu
title identity") corrected the menu-bar title path: the title resolved
through the private custom variant (`MENU_BAR_TITLE_VARIANT`), which
bypassed the public MENU/DEFAULT/TITLE identity; it now resolves through
`Variant::DEFAULT` so the `Part::TITLE` rule above (and the family default
variant, overlay rules and local part patch) applies as documented. This
correction is part of the same decision: the TITLE rule is a shared
`Part::TITLE` rule, not a private menu-bar variant.

## Independent review

The independent review of the holla reference-regeneration case
(2026-10-22, verdict "CASE STANDS") explicitly adjudicated this recipe
change on its own merits: **keep, do not revert**. The full findings are
appended to `docs/audits/2026-10-22-holla-reference-regeneration-case.md`
("Independent review 2026-10-22: CASE STANDS").

## Holla closure path

Holla's red store proceeds via reference regeneration (Option B of the
regeneration-case audit): all 1088 digest rows plus the 27 red families'
frozen snapshots, captured from one candidate build at one owner-pinned
HEAD, in one batch — executed only after the owner records the scope
approval at that pinned HEAD. The recipe itself is not touched again by
that closure.
