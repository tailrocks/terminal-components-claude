---
schema: task/v5
id: TASK-079
title: "Add oracle-exact fade-mix helper to the theme builder"
kind: bugfix
---

# TASK-079 — Add oracle-exact fade-mix helper to the theme builder

## Goal

A named theme path blends oracle fade colors so TASK-014 can comply with its contract.

## Context

TASK-014's contract (R-001) requires oracle `src/ui/fade.rs` arithmetic:
outer rows retain 55% contrast, inner rows 80%, componentwise rounding
toward the majority background, majority-background ties, and a
non-RGB outer `DIM` fallback. Two independent agents confirmed
TASK-014 BLOCKED-STANDS on 18 evidence points: the palette rule
(`COMPONENT_ARCHITECTURE.md` §22.7 R-10/D-10, enforced by
`architecture::palette_literals_are_confined_to_theme_builtins` in
`xtask/src/main.rs`) forbids `Color::Rgb(`/`from_u32`/hex construction
anywhere outside `theme/builtin/junie.rs`, `theme/builtin/paper.rs`,
`theme/builder.rs`, and `theme/downgrade.rs`, with no exemption path
(`crates/tui/tests/allow/legacy_api.txt` must stay empty; the §22
table bans `#[allow(`). No callable theme API constructs a blended
RGB today: `blend`, `shift_l`, and `lab_to_rgb` in
`crates/tui/src/theme/builder.rs` plus `nearest_256` in
`crates/tui/src/theme/downgrade.rs` are all private; `rgb_of` extracts
but cannot discriminate `(Rgb, Rgb)`; `downgrade_color` collapses;
`derive_unset` has wrong fixed alphas. TASK-014 scope excludes
`theme/`. Hence a named theme path must exist before TASK-014 can
comply, and this repair provides exactly one: a `pub(crate)` helper
plus its outcome type in a single already-path-allowed theme file,
with exhaustive parity proof against the pinned transcription.

This task has no visual surface of its own: no capture/compare lanes
exist here. The consumer proof lands with TASK-014 r3, whose oracle
lanes prove transcription == oracle; this task proves implementation
== transcription exhaustively.

This is a repair-task package. Its sole dependency is TASK-011, the
theme scope owner: this task writes inside TASK-011's owned `theme/`
scope, so it consumes TASK-011's accepted product as its base. There
is deliberately no TASK-014 edge: TASK-014 already depends on
TASK-011, so once this repair integrates, TASK-014's tree contains the
helper through its existing edge (see D-002). TASK-001 and TASK-070
stay retired fail-closed. UI authority is immutable oracle
`02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural starting
source is `7b27732a8c3c131760ec3438f641cb3c11343a42`. Never contact
the oracle beyond the in-tree descriptions cited here; never write
`snapshots/`, bless output, or mutate the tag.

Read before editing:

- `CAMPAIGN_AGENTS.md`: repository scope and integration constraints; this task-local AGENTS.md defines subagent execution and verification.
- `trusted/obligations.md` in full: the pinned transcription, branch table, pinned vectors, fixed test names, and driver contract are normative.
- `../014/README.md` and `../014/trusted/obligations.md` plus `../014/trusted/source-witnesses.md` (read-only demand source; never edit the 014 package).
- `../../../../docs/refactoring-plan/components.md` CP-01, `architecture.md`, `architecture-adjudication.md` and `proof-contract.md`.
- `../../../../crates/tui/src/theme/builder.rs` and `../../../../crates/tui/src/theme/downgrade.rs` in full (the file-choice evidence for D-001).

## Preconditions

- **P-001:** Accepted TASK-011 evidence and actual integrated source ancestry resolve from protected receipts; the candidate starts at the recorded parent and scope base with a clean tree.
- **P-002:** Source pins, taskfmt `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, toolchain/lock and immutable catalog/context identities resolve from protected receipts. The base tree carries TASK-078's accepted accounting projection, consumed via the base (D-006), so the CHK-005 accepted claims exist to be judged.
- **P-003:** The oracle/commit pins, the task contract, and the trusted driver/templates are immutable. Derived bindings come from the task manifest, the oracle, and the candidate worktree — never from candidate-written expectations.
- **Host-local verifier inputs:** The verifier subagent builds the native proof tool with `scripts/campaign-build-proof.sh`, runs `tc-proof prepare` for this package, then standalone taskfmt verify with explicit `--task-dir`, `--root`, `--base`, and `--log-dir` paths and `RUN_DIR` exported. Per-check contexts resolve under `$RUN_DIR/contexts/`. CHK-002 and CHK-003 run `cargo nextest` directly; never `cargo test`. No mounts are involved.

## Scope

In scope:

- `crates/tui/src/theme/builder.rs` solely for the O-001 helper plus its outcome type and the O-002 colocated parity tests.
- Independent qualification outputs in verifier-subagent-owned external run directories; no production application changes.

Out of scope:

- Every theme sibling: `theme/mod.rs`, `theme/border.rs`, `theme/builtin/`, `theme/downgrade.rs`, `theme/glyph.rs`, `theme/palettes.rs`, `theme/patch.rs`, `theme/recipe.rs`, `theme/resolve.rs`, `theme/role.rs`, `theme/tokens.rs`.
- Every TASK-014 scope file: `crates/tui/src/scroll.rs`, `crates/tui/src/components/scroll_region.rs`, `crates/tui/src/ui/paint.rs`, `crates/tui/src/author.rs`, `crates/tui/tests/completion_014.rs`. TASK-079 must not do TASK-014's work.
- Any new `crates/tui/tests/completion_079.rs`: the parity proof is colocated, not an integration target.
- Any other product file, any `refactoring-tasks/**` change, any proof-tool change, baseline blessing, provider execution, branch promotion and publication.

## Requirements

- **R-001 (MUST):** Satisfy every exact clause mapped to R-001 in `trusted/obligations.md` (O-001). Add exactly one `pub(crate) fn fade_mix(fg: Color, bg: Color, amount: f32) -> FadeOutcome` plus the `FadeOutcome` outcome type in `crates/tui/src/theme/builder.rs`, implementing the full oracle fade color branch: `(Rgb,Rgb)` pairs mix per the pinned `f32` transcription at the caller-supplied amount; any other pair yields `ApplyDim` at the outer amount `0.55` and `Unchanged` at the inner amount `0.80`, failing closed to `Unchanged` on illegal amounts. Both pinned vectors hold exactly.
- **R-002 (MUST):** Satisfy every exact clause mapped to R-002 in `trusted/obligations.md` (O-002). Colocate the four fixed-name parity tests in `builder.rs` `mod tests`: both pinned vectors, the exhaustive channel-pair sweep against the pinned transcription at both amounts, and the full non-RGB branch matrix including illegal amounts. All four pass under the CHK-002 filter.
- **R-003 (MUST):** Satisfy every exact clause mapped to R-003 in `trusted/obligations.md` (O-003). Change no other theme behavior: every pre-existing colocated `theme::` unit test passes unchanged (CHK-003). The prepared production account-tests context lifts the template exactly, both requires flags hold, and both accepted claims are present and well-formed (CHK-005 driver).
- **R-004 (MUST NOT):** Violate any prohibition mapped to R-004 in `trusted/obligations.md` (O-004). Never touch a theme sibling, a TASK-014 scope file, any other product file, any task package, or any proof tool. Never add another `Color::Rgb`/`from_u32`/hex construction site; the palette rule stays green (CHK-006). Never hardcode digests, receipts, commits, or membership to satisfy the judge.

## Acceptance criteria

### AC-001 — Helper matches the pinned oracle transcription

```gherkin
Given the pinned oracle fade transcription with outer 0.55 and inner 0.80 amounts and the W-014-01 witness colors
When the colocated fade mix sweep and pinned vector tests run on the fixed candidate
Then every channel pair matches the transcription at both amounts and both pinned vectors and the non-RGB branch matrix hold
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`, `R-002`
- **Check:** `CHK-002`

### AC-002 — Existing theme suites stay green

```gherkin
Given the existing colocated theme unit suites for builder downgrade and sibling theme modules
When the full theme lib suite runs on the fixed candidate
Then every pre-existing theme test passes unchanged
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-003`

### AC-003 — Complete regression accounting holds

```gherkin
Given the declared production accounting template with both requires flags true
When prepare binds the production account-tests context
Then the context envelope is genuine and both accepted claims are present with well-formed receipts
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-005`

### AC-004 — Verification authority stays immutable

```gherkin
Given the campaign-pinned catalog source pins expected artifacts and accepted prerequisite receipts
When exact identity ancestry scope and trust boundaries are checked
Then no forbidden write or candidate-controlled acceptance input is admitted
```

**Verification**

- **Type:** invariant
- **Covers:** `R-004`
- **Check:** `CHK-001`

### AC-005 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-007`

### AC-006 — Palette confinement holds

```gherkin
Given the live theme builder implementation and the palette confinement rule
When the architecture worker scans color construction sites
Then the fade helper is the only new construction site inside the allowed path and every prohibition holds
```

**Verification**

- **Type:** invariant
- **Covers:** `R-004`
- **Check:** `CHK-006`

## Fixed decisions

- **D-001:** The helper lives in `crates/tui/src/theme/builder.rs`, not `downgrade.rs`. Builder is the sole home of sRGB channel-mix construction: private `blend` has the identical mix/round/clamp shape, and every computed `Color::Rgb` construction (`shift_l`, `blend`, `anchor_l`, the `fg` ladder) already lives there. Downgrade only matches and discriminates `Rgb` and maps to named/indexed/mono outcomes — a blended-`Rgb` constructor would be a foreign shape there. Fade is derivation (a new color from two colors), not capability projection. `pub(crate) mod builder` needs no `mod.rs` re-export, so the single-file scope holds, and colocated tests inherit the path allow-list.
- **D-002:** `task.toml` depends on TASK-011 only, with no TASK-014 edge. TASK-011 owns the `theme/` scope this task writes, so its accepted product is the required base. TASK-014 already depends on TASK-011; once this repair integrates, TASK-014 r3 consumes the helper via the base tree through that existing edge. A 014 edge here would invert the dependency direction.
- **D-003:** This task proves implementation == pinned transcription exhaustively; transcription == oracle is TASK-014 r3's oracle-lane proof. The transcription derives strictly from in-tree sources (014 R-001, CP-01, W-014-01 inputs); the pinned vectors are computed from that formula, not copied from oracle source. If the transcription ever disagrees with the oracle, TASK-014 r3 fails closed downstream — never silently.
- **D-004:** The row is encoded by the caller-supplied amount: `0.55` is outer, `0.80` is inner, per TASK-014 R-001. Outer holds iff `amount == 0.55f32` exactly; same-literal comparison is exact, so no epsilon is used or needed. Non-`Rgb` pairs at illegal amounts fail closed to `Unchanged`: no paint change rather than a wrong one.
- **D-005:** There is no CHK-004 by design. This task has no visual surface, so no capture/compare lanes exist; the consumer proof lands with TASK-014 r3. Check numbering keeps the 029 identities for preflight (CHK-001), account-tests (CHK-005), architecture (CHK-006), and close (CHK-007); CHK-002/CHK-003 are the focused nextest lanes.
- **D-006:** The CHK-005 `acct-presence` driver judges the prepared-context envelope plus accepted-claim presence and receipt shape only; projection correctness stays TASK-078's ownership. TASK-078 is consumed via the base tree (P-002), not a task edge, because this task touches no proof code.
- **D-007:** Reusing the private `f64` `blend` inside the helper is permitted only if the sweep passes: it is not known bit-exact to the `f32` transcription at `.5` boundaries, so reuse is at the implementer's risk and the exhaustive sweep decides.
- **D-008:** The non-RGB outer variant is named `ApplyDim`, not `Dimmed`. Rule 16 of `architecture::no_deprecated_or_legacy_api_usage` (`xtask/src/main.rs`, R-20) forbids the token `\bDimmed\b` in non-test code with no path allow-list and a must-be-empty `legacy_api.txt`, so any production `Dimmed` identifier in `theme/builder.rs` is unimplementable; only `//` comments, string literals, and `#[cfg(test)]` items escape the scan. `ApplyDim` was verified clean against all 28 rule regexes plus the forced-state and domain-vocabulary token lists. Behavior is unchanged: same branch table, amounts, `f32` transcription, pinned vectors, and fixed `fade_mix_oracle_*` test names; only the Rust identifier changed.

## Subagent execution

The implementer, verifier, and reviewer subagents own this task. The coordinator assigns isolated worktrees, reviews evidence, and integrates only reviewed commits; it does not edit task-owned files.

All execution is host-local. Use `$TASK_DIR` for this package, `$WORKTREE` for the isolated repository, `$RUN_DIR` for evidence and logs, and `$SCOPE_BASE` for the recorded parent. Run the latest standalone taskfmt only for this package:

```text
"$TASKFMT" lint "$TASK_DIR"
"$TASKFMT" verify --root "$WORKTREE" --task-dir "$TASK_DIR" \
  --base "$SCOPE_BASE" --progress "" \
  --log-dir "$RUN_DIR/taskfmt-logs"
```

Taskfmt is validation only. No containers, images, mounts, or task orchestration commands are used. The verifier owns the final taskfmt evidence; the reviewer checks it against every `R-*`, `AC-*`, and `CHK-*` obligation before the coordinator integrates. Keep generated evidence under `$RUN_DIR` and do not modify task metadata or protected oracle inputs.


## Checklist

<!-- checklist:start -->
- [ ] **1** Bind exact authority and scope.
    - [ ] **1.1** Validate protected receipts, parent ancestry and task inputs. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Deliver the owned repair outcome.
    - [ ] **2.1** Add the oracle-exact fade helper with exhaustive colocated parity proof. (`R-001`, `R-002`, `AC-001`, `CHK-002`)
    - [ ] **2.2** Keep every pre-existing theme suite green. (`R-003`, `AC-002`, `CHK-003`)
    - [ ] **2.3** Bind genuine production accounting with both accepted claims present. (`R-003`, `AC-003`, `CHK-005`)
    - [ ] **2.4** Preserve palette confinement and every scope prohibition. (`R-004`, `AC-006`, `CHK-006`)
- [ ] **3** Verify the fixed candidate.
    - [ ] **3.1** Pass the complete host gate and preserve exact-tree evidence. (`R-001`, `R-002`, `R-003`, `R-004`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
