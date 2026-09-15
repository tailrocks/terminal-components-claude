# Campaign iteration guide — visual baseline performance

**ADDENDUM** to [Campaign execution prompt](campaign-execution-prompt.md). Stored **2026-09-15** alongside the frozen campaign catalog on `visual-baseline`.

This guide clarifies how executors iterate quickly during development without weakening the fail-closed visual oracle. It does **not** relax acceptance boundaries: the full matrix still gates task acceptance, application-chain closure, cross-application joins, and final closure (TASK-069).

Related docs: [Visual validation during refactoring](../../refactoring-tasks/visual-validation.md), [tuisnap coverage](../baseline/tuisnap-coverage.md), [nextest config](../../.config/nextest.toml).

---

## Suite shape (why filters work)

The visual baseline is `tests/visual_baseline/`: **7,550** per-combo `#[ignore]` PTY tests (one process per capture; nextest `pty` group, max 16 threads). Default `cargo nextest run` compiles the suite but runs only the cheap non-PTY [`store_integrity`](../../tests/visual_baseline/main.rs) check (~544 other unit/lib tests in the default inventory).

Canonical roots expand to **25 combos** each (5 terminal sizes × 5 colour modes). Combo tests are named:

```text
<app>::<capture_root>::c<cols>x<rows>_<color>
```

Example: `holla::holla_concept_first_use_default_120x40_truecolor::c120x40_truecolor`.

List names for a surface:

```sh
cargo nextest list --run-ignored only -E 'binary(visual_baseline) & test(showcase_)'
```

---

## Tiered gates (when to run what)

| Phase | Command | Scope | Typical wall time* |
| --- | --- | --- | --- |
| **Edit loop** (per task) | Targeted filter + `TUISNAP_FAST=1` | Affected app/module/scenario only | seconds–few minutes |
| **Ordinary regression** | `cargo nextest run` | ~544 non-ignored tests (no PTY captures) | ~1–3 min |
| **PR / CI smoke** | `cargo nextest run --profile ci --run-ignored only -E 'binary(visual_baseline)'` | ~302 captures (120×40 truecolor only) | **~1–2 min** |
| **Nightly speed feedback** | `TUISNAP_FAST=1 cargo nextest run --run-ignored only -E 'binary(visual_baseline)'` | Full 7,550 combos; tiered gate skips PNG/HTML when `.ansi`+`.txt` match | **~30–45 min** |
| **Task acceptance / pre-release fidelity** | `cargo nextest run --run-ignored only -E 'binary(visual_baseline)'` | Full matrix; every combo renders PNG/HTML at fidelity timing | **~45–60 min** (current tree); **~2 h** legacy pre-per-combo path |

\*Measured on a 16-thread PTY pool (2026-09-15 benchmarks on `visual-baseline`). CI runners with fewer cores may take longer; smoke remains ≪ full matrix.

**Rule:** During an edit loop, never run all 7,550 captures. Run the **smallest filter** that covers the changed surface, then escalate through the tiers above at boundaries.

---

## 1. During development iteration (per task)

After a production edit that can affect rendering or interaction:

1. Run focused unit/lib tests for the touched crate or app (`cargo nextest run -E 'test(…)'` as named by the task contract).
2. Run a **targeted visual filter** for the affected app/family/scenario with fast mode:

```sh
TUISNAP_FAST=1 cargo nextest run --run-ignored only \
  -E 'binary(visual_baseline) & test(<filter>)'
```

Pick `<filter>` from the table below. Prefer a single capture root or one combo when debugging a specific regression.

Do **not** substitute a green targeted run for the mandatory full gate at task acceptance (see §4).

---

## 2. PR / CI smoke

PR CI uses the **`ci` nextest profile**, which sets `NEXTEST_PROFILE=ci` → smoke matrix (120×40 truecolor only) via [`support.rs`](../../tests/visual_baseline/support.rs). No `TUISNAP_FAST` required for smoke.

```sh
cargo nextest run --profile ci --run-ignored only -E 'binary(visual_baseline)'
```

~302 active captures, **~1–2 min** wall time. This is the default automated gate on pull requests; it is **not** sufficient for task acceptance or final closure.

---

## 3. Full matrix gates

**Task acceptance and closure** require the **fidelity** full matrix (no `TUISNAP_FAST`):

```sh
cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
```

Optional **nightly speed feedback** (not a substitute for acceptance):

```sh
TUISNAP_FAST=1 cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
```

| Mode | Behaviour | Timing |
| --- | --- | --- |
| `TUISNAP_FAST=1` | 100 ms settle, 0 ms step pacing, tiered grouped gate (`full_render: false`) | **~30–45 min** |
| Default (fidelity) | 400 ms settle, 120 ms pacing, full PNG/HTML gate every combo | **~45–60 min** current tree; **~2 h** on pre-per-combo / pre-fast infrastructure |

**FAST vs fidelity snapshots:** committed `snapshots/` are blessed at **fidelity** timing. `TUISNAP_FAST=1` can therefore report `cells-differ` on timing-sensitive flows (animation, duration labels, wheel settle) even when fidelity passes. Measured on `623a1a59`: **7350/7550 PASS** under FAST; **200 FAIL** across 11 flow families (~194 snapshot drift, ~6 parallel-capture timeouts that pass in isolation). Do **not** re-bless snapshots to match FAST unless explicitly adopting FAST as the new oracle; use targeted FAST filters during edit loops and fidelity for acceptance.

Heavy flows (long `wait:` needles, e.g. `holla_flows_docker_done`) dominate single-combo time (~15–20 s); fast mode only trims settle/pacing/render overhead there.

Optional HTML index after bless/review (not the capture gate):

```sh
cargo nextest run --run-ignored only --ignore-default-filter -E 'test(rebuild_review_html)'
```

---

## 4. When the prompt says “mandatory full visual gate”

In [Campaign execution prompt](campaign-execution-prompt.md), **“mandatory full visual gate”** means the **fidelity** full matrix:

```sh
cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
```

Use `TUISNAP_FAST=1` only for **targeted edit-loop filters** (§1) or optional nightly speed feedback (§3). It is **not** equivalent to the mandatory acceptance gate when snapshots are fidelity-blessed.

**Required at:**

- Task **acceptance** after canonical host verification (when the task can affect product output).
- **Failure/repair loop** step 8 — after a candidate revision, before a fresh independent verifier.
- **Parallel sibling integration** — union of impacted checks on the combined tree (not each sibling in isolation).
- Application-chain closures **039 / 050 / 057 / 064**.
- Cross-application joins and **final closure** (TASK-069).
- **Definition of done** — zero unintended visual differences on the final tree.

**Not required on every edit loop.** The prompt already says “run targeted feedback first if useful”; this guide makes that explicit: implementers should use §1 filters while iterating, then run the full gate once per acceptance boundary.

Receipts must record which tier ran (smoke / fast full / fidelity full) and the exact command.

---

## 5. Example nextest filters

| Intent | Example `-E` expression |
| --- | --- |
| Whole app | `binary(visual_baseline) & test(showcase_)` |
| Holla flows family | `binary(visual_baseline) & test(holla_flows_)` |
| One capture root (25 combos) | `binary(visual_baseline) & test(holla_concept_first_use)` |
| Single combo | `binary(visual_baseline) & test(holla_concept_first_use_default_120x40_truecolor::c120x40_truecolor)` |
| Smoke slice of one app (CI profile) | `cargo nextest run --profile ci --run-ignored only -E 'binary(visual_baseline) & test(holla_)'` |
| Store inventory (non-PTY, always cheap) | `cargo nextest run -E 'test(store_integrity)'` |
| Pointer / mouse group | `binary(visual_baseline) & test(pointer_)` |
| Audit fixtures | `binary(visual_baseline) & test(audit_)` |

Combine with `\|` for a small set:

```sh
TUISNAP_FAST=1 cargo nextest run --run-ignored only \
  -E 'binary(visual_baseline) & (test(holla_concept_first_use) | test(holla_concept_docker_cleanup))'
```

---

## 6. Ordinary `cargo nextest run` scope and timing

Default profile ([`.config/nextest.toml`](../../.config/nextest.toml)):

- Runs **~544** tests across the workspace **excluding** ignored visual captures and `rebuild_review_html`.
- Includes `store_integrity` (validates committed `snapshots/` inventory vs suite).
- Does **not** spawn PTY captures.
- Typical wall time: **~1–3 minutes** (compile excluded).

Use after production changes unless canonical host verification already proves an equivalent complete inventory (per prompt § Functional non-regression). This is the fast “did I break unit tests?” gate; it is not a substitute for visual regression on rendering tasks.

---

## 7. Environment variables

| Variable | When set | Effect |
| --- | --- | --- |
| **`TUISNAP_FAST=1`** | Local iteration, optional nightly speed feedback | 100 ms settle; 0 ms inter-step pacing; tiered grouped gate — skips PNG/HTML render when `.ansi` and `.txt` both match ([`GroupedCheckOptions { full_render: false }`](../../tests/visual_baseline/support.rs)). Not equivalent to acceptance when snapshots are fidelity-blessed (§3). |
| **`TUISNAP_MATRIX=smoke`** | PR CI or manual smoke | Restricts matrix expansion to **120×40 truecolor** only. Equivalent to `--profile ci` (which sets `NEXTEST_PROFILE=ci` → same hook). |
| **`TUISNAP_BLESS=1`** | Host bless/regeneration workflows only | Matrix runners continue after combo failure so all actuals are written before panicking. **Candidates must never set this**; executors must never bless. |
| **`NEXTEST_PROFILE=ci`** | Set by `--profile ci` | Enables smoke matrix via `smoke_matrix()` in support.rs. |

Blessing approved snapshots remains **host-only**:

```sh
ln -sfn target/tuisnap/actual snapshots.actual
tuisnap accept --grouped --store snapshots --name <group/…/size/color>
```

See [visual-validation.md](../../refactoring-tasks/visual-validation.md).

---

## Prompt amendments (applied 2026-09-15)

All coordinator amendments below are **applied** in [campaign-execution-prompt.md](campaign-execution-prompt.md):

- Visual regression: iteration vs acceptance split; link to this guide; targeted `TUISNAP_FAST=1` edit loops; **fidelity-only** mandatory full gate at boundaries.
- Closure bullets: fidelity full matrix at app closures and TASK-069 (not FAST full matrix).
- IMPLEMENTER: targeted visual filters allowed for feedback, not acceptance.
- Final report: require command, env, and tier (smoke / fast full / fidelity).
