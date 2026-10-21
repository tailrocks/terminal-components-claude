# Borrowed NavList composition adapter

The public facade now adapts `NavList<T, K, R>` into a composition `Child`
using borrowed component configuration, `NavListState`, and item data. The
private leaf delegates measurement to `NavList::measure` and drawing to
`NavList::draw`; it adds placement only. Generic `KeyFn` and `RowFn` bounds
remain intact, so non-`Display` application view data continues to work.

The new integration consumer exercises a sectioned catalogue with stable
numeric item keys and application-owned non-`Display` rows. It checks live
row regions, keyboard navigation and activation, pointer activation, reorder
stability, empty data, long Unicode data, scrolling, resize, and the existing
master/detail collapse boundary at widths 107, 108, and 109. It runs those
breakpoint checks at truecolor, 256-color, 16-color, and monochrome levels.
Its draw method measures before drawing and asserts measurement did not mutate
component state or invoke row renderers.

Before the adapter existed, a standalone non-`Display` consumer compiled
against `04a37e1bb63583a34948620897655dd057d7dd86` failed only at
`composition::nav_list` (`cargo check --manifest-path
/private/tmp/nav-list-composition-red/Cargo.toml`, exit 101, E0425). The same
consumer compiled after the adapter was added (`cargo check --manifest-path
/private/tmp/nav-list-composition-red/Cargo.toml`, exit 0, with its path
dependency pointed at this worktree). The adapter fixes that missing public
contract. It does not alter
NavList measurement, row styling, input behavior, or application code. In
particular, Holla's frozen Finder prefix and row metadata differences remain
separate shared-component work; this adapter does not establish Holla parity
or authorize app-local painting.

Validation on the implementation worktree: `cargo nextest run --locked -p
termrock --features testing --all-targets` passed 28/28, including the new
consumer's three tests; `cargo clippy --locked -p termrock --all-targets
--features testing -- -D warnings`, `cargo check --locked -p termrock
--all-targets`, `cargo fmt --check -p termrock`, and `git diff --check` passed.

The same gates passed after merging the then-current PR #38 tip into WIP
commit `637a7e9b0d5498b151392fce203adfc0356c00f6`: testing-enabled facade
nextest 28/28, strict Clippy, default-feature all-target check, package
formatting, and diff check.
