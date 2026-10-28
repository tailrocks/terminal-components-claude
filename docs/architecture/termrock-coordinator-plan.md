# Coordinator plan — Termrock migration completion (PR #38)

Delivery: `parity/termrock-visual-baseline` (PR #38) only. `main` is never
pushed to directly. `design/interaction-polish` (PR #26) and the
`visual-baseline` tag stay unchanged.

Ledger: [termrock-migration-record.md](./termrock-migration-record.md).

## Acceptance surface

- Reference inventory: 302 screens (holla 125, showcase 84, jackin 49,
  tablepro 44), each at 5 sizes and 5 colour modes, 7550 captures,
  75500 artefacts. Oracle store `baselines/tuiscotti-v1/`.
- Pending paired-comparison cases: 293 named pending tests in
  `crates/termrock-conformance/tests/visual_baseline/`.
- Architecture gate: `python3 tools/acceptance_architecture.py` — green at
  0 findings, 0 dependency violations. Five requirements remain
  unresolved in `tools/acceptance_policy.json`.

## Workstreams

| Stream | Owner | Files owned | Goal |
| --- | --- | --- | --- |
| A — Jackin | agent-jackin | `crates/jackin-preview-app/**` | Delete every `historical_*` / size-keyed screen bypass; make the live chrome produce the reference frame. |
| B — Holla | agent-holla | `crates/holla-ui/**`, `crates/holla-sim/**` | Delete every `*_plate` / `parity_*_plate` screen bypass; real composition instead. |
| C — TablePro | agent-tablepro | `crates/tablepro-ui/**`, `crates/tablepro-demo/**` | Delete picker/grid/tab canned views; real Picker/Grid/Tabs composition. |
| D — Showcase | agent-showcase | `crates/showcase-ui/**`, `crates/showcase-demos/**`, `crates/showcase/**` | Shared widget adoption, no app-side drawing or interaction logic; coverage gate green. |
| E — Termrock library | agent-termrock | `crates/termrock-*/**` | Close the library gaps the apps report; keep Termrock free of app internals. |

Rule for A–D: an app writer owns only its application crates. It must not
edit any `crates/termrock-*` crate. A library gap is recorded in the
migration record with the exact API needed and scheduled for stream E.

Rule for E: a library writer must not edit any application crate, and must
keep every new API independent of application internals.

## Work order

1. Audit round (read-only, parallel): false-parity inventory, verification
   infrastructure map, PR #26 extraction gap, branch census, library API gap.
2. Implementation round (parallel, file-disjoint): streams A–E.
3. Independent review round: separate reviewers for UI/UX, runtime
   correctness, API boundaries, test integrity. No author-only approval.
4. Integration: coordinator rebases each increment, runs the gates, and
   pushes under the shared push lock.

## Push protocol

```bash
LOCK="$(git rev-parse --git-common-dir)/push.lock"
mkdir "$LOCK" && printf 'branch=<name>\ndate=<iso>\n' > "$LOCK/owner"
git fetch origin parity/termrock-visual-baseline
git rebase origin/parity/termrock-visual-baseline
cargo nextest run --all-targets ...   # scoped gate on the exact HEAD
git push origin HEAD:parity/termrock-visual-baseline
rm -rf "$LOCK"                          # also on failure
```

If `mkdir` fails the lock is held: keep working, retry at the next
checkpoint. Never force-push.

## Done means

- Every `historical_*`, `*_plate` and size/fixture-keyed screen bypass in
  application crates is gone, replaced by real component composition.
- Every pending paired-comparison case in the reference inventory is
  admitted and green, run against the real binaries with the pinned
  tuiscotti revision.
- The complete application comparison is run twice with the same result.
- `cargo nextest run --all-targets`, `cargo clippy --all-targets` and
  `cargo fmt --check` are green on the delivered HEAD.
- Termrock builds and tests with no application dependency; a separate
  consumer compiles through public APIs only.
- The five unresolved acceptance requirements in
  `tools/acceptance_policy.json` are discharged with recorded evidence.
- Independent reviews for UI/UX, runtime correctness, API boundaries and
  test integrity are recorded with no unresolved finding.
