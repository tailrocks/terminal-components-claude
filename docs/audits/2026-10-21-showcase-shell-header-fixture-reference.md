# Showcase shell header and page heading fixtures: reference defect and resolution

Date: 2026-10-21
Branch: `wip/showcase-heading-fixtures-20261021b`
Base: `f9de4f443` (`wip/session-consolidation-20261021`)
Reference oracle: the `visual-baseline` tag (never moved, never re-created)

## The question

Two Showcase checks failed and were recorded as a reference conflict instead of a
regression:

- `showcase-ui::page_headings all_page_headings_match_the_reference_capture_matrix`
- `showcase-ui::page_headings all_shell_headers_match_reference_captures_and_visibility_thresholds`

The live application rendered

```text
 ▪ Junie Design system  Inputs & forms › Buttons         16 colors · 100×30   i Inspector   ? Help
```

while the recorded fixture expected

```text
 ▪ Junie Design system / Components / Buttons            16 colors · 100×30   i Inspector   ? Help
```

The question is which side is the reference truth: the application, or the
fixture.

## Method

The tag was read directly. No candidate output was used as an oracle.

1. The tag's Showcase captures were extracted with
   `git archive visual-baseline snapshots/showcase` and the first line of every
   frame was read with the SGR sequences stripped.
2. The tag's Showcase sources were read with
   `git show visual-baseline:src/bin/showcase/app.rs`,
   `git show visual-baseline:src/widgets/breadcrumb.rs` and
   `git show visual-baseline:src/theme.rs`.
3. The live application was rendered through the same harness the tests use
   (`showcase_ui::App` + `termrock_test_support::Harness`) for all 355 fixture
   rows, and every rendered row was compared with the tag capture for the same
   page, size and colour tier.

## Evidence

### The tag holds two eras of Showcase captures

| tag captures | last changed | header model |
| --- | --- | --- |
| `snapshots/showcase/audit/**` | 2026-09-13 | ` ▪ Junie Design system / Components / Buttons` (flat "section" names, ` / ` separator) |
| `snapshots/showcase/pages/**` | 2026-10-16 | ` ▪ Junie Design system  App shell › Chrome` (grouped tree, ` › ` separator) |

The tag's own source belongs to the second era:

- `visual-baseline:src/bin/showcase/app.rs:1821` builds the header crumb from
  `[entry.group, entry.label]`;
- `visual-baseline:src/bin/showcase/app.rs:194-197` puts `Buttons` in the
  group `Inputs & forms`;
- `visual-baseline:src/widgets/breadcrumb.rs:79` separates the crumb with the
  `overflow_right` glyph, `visual-baseline:src/theme.rs:2427` defines it as
  `›`;
- `visual-baseline:src/bin/showcase/app_tests_shell_pages.rs:251` asserts
  `App shell › Breadcrumbs & rails`.

So the tag is internally inconsistent: the frozen `audit/**` captures are older
than the tag's own source and its own assertions. **Reference defect #1**: the
`audit/**` captures were never re-approved after the sidebar became a grouped
tree, and they still encode the superseded flat model.

### The live application reproduces the tag's newest captures exactly

Every one of the 288 fixture rows that has a matching new-era capture
(`snapshots/showcase/pages/**`, 18 pages × 4 sizes × 4 colour tiers) renders
**byte for byte** as the tag capture, on row 0 (the header) and on row 2 (the
page heading). Example, `visual-baseline:snapshots/showcase/pages/chrome/100x30/16.ansi`:

```text
 ▪ Junie Design system  App shell › Chrome               16 colors · 100×30   i Inspector   ? Help
```

The header row and the page heading row match exactly. The body rows of some
frames differ (the demo content evolved after the captures were approved: the
Editable page shows a task table where the capture shows the older table, and
a scrollbar thumb renders `▾┃` against `┃`). Those differences belong to the
`showcase-ui::visual showcase_visual_baseline` digest baseline and are not part
of this record.

### The four pages without a new-era capture

`buttons`, `forms`, `inputs` and `textareas` only exist in the old-era
`audit/**` captures. For all 64 of those rows the live rendering keeps the
tag capture's left prefix (` ▪ Junie Design system`) and its whole right-hand
part (the capability hint, the inspector and help affordances, with identical
columns) and changes only the crumb, exactly as the tag's source prescribes.

**Reference defect #2**: the tag's `audit/textareas/**` capture and
`visual-baseline:src/bin/showcase/pages/textareas.rs:131` carry the older
blurb `Multi-line editing, Unicode cursor motion, horizontal and vertical
scrolling`, while the app lineage (the fixture of 2026-09-08 and the live demo)
carries `Multi-line editing, wrapping cursor motion, scroll position`. The
Text areas blurb was therefore not taken from the tag: the fixture keeps the
value it already had and only the row layout changes.

### The capability clearance threshold

`visual-baseline:src/bin/showcase/app.rs:1809` draws the capability hint only
when `rx > x + crumb_w + cw + 2`. The Overview crumb has no group, so the
guard opens at 76 columns; the frozen captures agree
(`pages/overview/72x20/truecolor.ansi` has no hint,
`pages/overview/80x24/truecolor.ansi` has `truecolor · 80×24`). The old
fixture bracketed the same guard for the then-current crumb
(`Foundations / Overview`, 23 columns wide) at 89, 90 and 91 columns; the
guard now opens at 76, so the three threshold rows moved to 74, 75 and 76 and
keep the same shape (absent, absent, present).

## Verdict

The application rendering is reference-correct and the fixtures were stale:
the fixtures encoded the pre-tree sidebar model that the tag itself already
superseded on 2026-10-16. The fixtures were updated to the reference truth.

## What changed

- `crates/showcase-ui/tests/fixtures/holla-shell-headers.tsv` (355 rows): every
  row 0 expectation is now the tag reference value. 288 rows are verbatim tag
  captures; 64 rows keep the old-era tag capture's right-hand part with the
  grouped crumb; the three threshold rows moved from 89/90/91 to 74/75/76.
- `crates/showcase-ui/tests/fixtures/holla-page-headings.tsv` (352 rows): every
  row 2 expectation is now the tag reference value. 288 rows are verbatim tag
  captures; 64 rows keep the old-era tag capture's page title and blurb with
  the new-era row shape; the pickers, progress and scrolling blurbs follow the
  new-era tag captures.
- `crates/showcase-ui/tests/page_headings.rs`: two regression tests were added,
  `the_shell_header_matches_the_visual_baseline_capture` (pins the header line
  of the tag capture quoted above) and
  `the_capability_hint_clears_the_crumb_at_the_reference_threshold` (pins the
  clearance at 75 and 76 columns).

## Both sides preserved

The replaced expectations are kept verbatim:

- `docs/audits/evidence/2026-10-21-showcase-shell-header-fixture/holla-page-headings.tsv`
  (`sha256 faa2a62d72736bc6ead7b99135a7d6e3f1ff1193b8bf48dc5f424241e3ccf0c9`)
- `docs/audits/evidence/2026-10-21-showcase-shell-header-fixture/holla-shell-headers.tsv`
  (`sha256 0c049901918b25d610dca3b3d4a693cd2621af2a6d673056159bb16988fc098b`)

They are the fixtures of `f9de4f443` and stay reachable in the history of
`crates/showcase-ui/tests/fixtures/`.

## Independent review

This record resolves a conflict between a frozen artifact and repeatable
reference output, so it needs independent review before it is treated as
closed.
