# Clap doc comments are help text (2026-11-01)

## Finding

`showcase_page_statushints` failed in CI (and locally with a fresh
build) with a one-cell visual mismatch at col 119, row 23: scrollbar
thumb `┃` where the baseline has track `┆`. The frame was stable
from the first checkpoint; the measured page content was one row
shorter than the baseline's.

## Root cause

Commit `eec7d02f7` ("remove all code comments") deleted `///` doc
comments on clap derive items. Doc comments on `clap::Args` fields
and `ValueEnum` variants are not inert comments: clap turns them
into `--help` text. `StatusHintsPage::shell_height` measures the
showcase "Command-line options" section from
`CommonArgs::augment_args`, including each argument's help string.
With the help strings gone, the section measured one row shorter,
the scrollable content shrank by a row, and the scrollbar thumb
grew by one cell. The same deletion degraded real `--help` output
of every app (`showcase`, `holla`, `jackin`, `tablepro`).

Local runs kept passing while the mbx target cache held a binary
built before the deletion; CI always builds fresh, so it failed
from the first frame.

## Fix

Restored every `///` doc comment that `eec7d02f7` removed under
`src/` (`src/cli.rs`, `src/motion.rs`, the four bin `main.rs`
files, `tablepro/app.rs`, `holla/domain/activity.rs`). No baseline,
assertion or test changed. Verified: the previously failing test
and the holla/jackin/tablepro groups pass locally with a fresh
build.

## Rule

Doc comments on clap items are user-visible help text. A
comment-removal pass must treat them as string literals.
