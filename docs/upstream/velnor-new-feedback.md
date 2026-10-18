# Feedback for velnor-new: visual-test CI jobs, `.github/CLAUDE.md` as a symlink, and the Rust toolchain

## Tracker

Consumer-side status, kept by the Upstream PR loop
(`docs/upstream/README.md`). Checked 2026-10-17 with `gh`; velnor-actions
**v0.1.12** (`f1b044a`) is adopted. v0.1.9 (`a7d49aa`, #156 on top of
#155) carried our items; v0.1.10 (#158, repository-pin tool execution),
v0.1.11 (#159, mr-boxington preflight) and v0.1.12 (#162, `[workflow]
mise_quiet`) fix none of the open rows below: re-checked on the v0.1.12
binary with `generate --output-dir` from our config, and
`baseline_publish.rs` is unchanged since v0.1.9.

| Item (§) | PR/issue | State | Last reviewed head | Last review link | Verdict | Our follow-up after merge |
|---|---|---|---|---|---|---|
| Problem A, §2/§4/§5: suites, partitions, triggers, artifacts, timeouts, `Required`, preserved workflows | [#155](https://github.com/tailrocks/velnor-new/pull/155) | merged as `e34cfb8`; released in **v0.1.9** (`a7d49aa`, 2026-10-16) | `592108c` (= `e34cfb8`) | [verification comment](https://github.com/tailrocks/velnor-new/pull/155#issuecomment-6218923461) | verified on `main`, then on the v0.1.9 binary: `suites.yml` generated from our config, drift check clean, actionlint clean | **done 2026-10-17**: manifest replaced, four visual suites (`workflow_dispatch` only until plan T18), `[test_sharding]` and the commented visual toggle removed (`docs/ci.md` "Visual suites") |
| Problem B, §3: `.github/CLAUDE.md` as a symlink | [#155](https://github.com/tailrocks/velnor-new/pull/155) | merged (same PR); released in v0.1.9 | `592108c` | [verification comment](https://github.com/tailrocks/velnor-new/pull/155#issuecomment-6218923461) | verified on v0.1.9: `[agents] claude_md = "symlink"` generates `.github/CLAUDE.md -> AGENTS.md`, drift check clean | **done 2026-10-17**: exception removed from `AGENTS.md` and `docs/ci.md` |
| Problem C, §8: repository Rust toolchain and tool-version overrides | [#155](https://github.com/tailrocks/velnor-new/pull/155) (added in `592108c`), hardened in [#156](https://github.com/tailrocks/velnor-new/pull/156) (`a7d49aa`) | released in v0.1.9 | `a7d49aa` | [verification comment](https://github.com/tailrocks/velnor-new/pull/155#issuecomment-6218923461) | verified on the v0.1.9 binary: `rust@1.99.1` + clippy/rustfmt from `rust-toolchain.toml`, no `RUSTUP_TOOLCHAIN`. Follow-ups from the comment, checked by `generate --output-dir`: (1) non-exact channel **fixed** in #156, `stable` and `1.99` now fail with `invalid_rust_toolchain`; (2) **open, not blocking, still in v0.1.12**: without a toolchain file the default is still `rust@1.98.1` with `RUSTUP_TOOLCHAIN`, catalog zizmor still 1.30.1 (1.31.1 out); (3) **open, not blocking, still in v0.1.12**: `[tools]` versions are still only shape-checked (`zizmor = "99.99.99"` generates); (4) `--extract-to .` with an existing `./target` **fixed** in #156 (`--extract-overwrite` on every archived suite run) | **done 2026-10-17**: `rust-version` 1.99 (root, xtask), mirror clippy under `RUSTUP_TOOLCHAIN=1.98.1` removed from `docs/ci.md` and `docs/dependencies.md`. zizmor stays at the catalog pin until a `[tools]` raise is proven by a CI run |
| Problem D, §9: `Publish baseline` fails on a working branch that is not the GitHub default | not filed yet | **open, not fixed in v0.1.12** (re-checked 2026-10-17; `baseline_publish.rs` unchanged since v0.1.9): `baseline_publish.rs` still takes `default_branch` from the event payload (`repository.default_branch`) while the job `if:` uses the configured `refs/heads/design/interaction-polish`; v0.1.12 output still has `if-no-files-found: error` on `Upload baseline` | `v0.1.12` | n/a | velnor bug: `publish_gate` (`baseline_publish.rs:224-244` at v0.1.9) refuses every push here with `publish_refused:unprotected_ref`, then `Upload baseline` fails on the missing file. Not fixable by config: `default_branch = "main"` would stop push CI on this branch | needs a velnor PR; after the fix: regenerate and confirm `Publish baseline` is green on a push; until then the job is red on every push while `Required` stays green |


You are working on **velnor-new** (<https://github.com/tailrocks/velnor-new>),
the generator behind `velnor-actions`. A consumer repository needs three
changes that velnor-actions 0.1.8 cannot express today (Problem C, §8,
was added after the first two). This prompt
contains everything you need. You have no access to the conversation that
produced it.

## 1. Context: who we are and how we use velnor

- Consumer: <https://github.com/tailrocks/terminal-components-claude>, branch
  `design/interaction-polish`. It is a Rust (ratatui) TUI component library.
  The Cargo workspace is `members = [".", "xtask"]` (`Cargo.toml:14`).
  Inspect these files on that branch:
  - `.velnor/config.toml`: schema 1, `policy = "consumer-v1"`,
    `runner_label = "ubuntu-24.04"`, `compile_driver = "cargo"`,
    `test_runner = "cargo_nextest"`, `run_ignored = "default"`, and a
    commented `[test_sharding]` block plus a commented "VISUAL SMOKE TOGGLE".
  - `.velnor/release-manifest.json`: the v0.1.8 manifest (release commit
    `7c594f7aef03990d8ff952727b0c56497372e355`).
  - `.config/nextest.toml`: profiles `default`, `ci`, `smoke`, `full`,
    `narrow`, and a `pty` test group (`max-threads = 8`).
  - `.github/`: generated by velnor-actions 0.1.8 (`ci.yml` with
    `Actionlint`, `Plan`, `Rust / junie-tui`, the xtask job,
    `Publish baseline`, `Required`).
  - `docs/ci.md`: how we use velnor and the "Gaps" section this prompt
    turns into requests.
  - `docs/visual-testing/plan.md` §C1 (tiers), §C3 (runtime and sharding),
    §D (planned CI workflows); `docs/visual-testing/README.md`.
  - `AGENTS.md`: "`CLAUDE.md` must always be a symlink to the
    corresponding `AGENTS.md` in the same directory", with one exception
    today: `.github/CLAUDE.md`, because velnor generates it.
- Visual tests: every scenario is one ignored nextest test (PTY capture of a
  real binary, gated against a committed `.ansi` snapshot). Tiers are
  nextest profiles that filter on the test-name suffix:

  | Tier | Profile | Tests | Planned trigger | Planned shards |
  |---|---|---|---|---|
  | smoke | `smoke` | ~890 | every PR and push, **required** | 2 |
  | full | `full` | ~11,000 | nightly 02:00 UTC, manual | 12 |
  | narrow | `narrow` | ~1,050 | nightly, manual | 2 |
  | capture | full + narrow, failures allowed | — | manual only | 12 |

  Command per shard:
  `cargo nextest run --profile <tier> --run-ignored only --partition hash:<k>/<N>`.
  Planned: build once (`cargo nextest archive`), then each shard runs
  `--archive-file … --partition hash:k/N`. On failure, upload
  `target/tuiscotti/{actual,expected,diff}` plus the report as one artifact
  per shard (`visual-<tier>-<k>`, 14-day retention). Never run
  `cargo xtask visual accept` in CI.

## 2. Problem A: visual-test jobs we cannot express

Each limit below is cited at tag `v0.1.8`. Re-verify each one before you
change anything.

1. **Triggers are hard-coded.** `velnor-actions-orchestrator/src/workflow.rs:184-193`
   builds the trigger as PR types + push to the default branch + merge
   group. `workflow_dispatch` is set only for `VelnorRepositoryV1`, and
   `schedule: None` is set unconditionally. The IR already supports both
   (`velnor-actions-contract/src/workflow/ir.rs:26-41`, `ScheduleTrigger`
   at `:151-180`, typed inputs in `workflow/dispatch.rs`, used by
   `freshness_emit.rs:39`), but consumers have no config key for either.
2. **No artifact upload for repository paths.**
   `velnor-actions-workflow-renderer/src/steps_artifact.rs` has upload
   templates only for velnor's own candidate, plan, report and baseline
   artifacts. A consumer cannot ask for `target/tuiscotti/**` with
   `if: failure()`.
3. **Shards are steps in one job and do not partition.**
   `[test_sharding] default_shards = N` creates task IDs
   `…/shard-i-of-n` (`velnor-actions-orchestrator/src/identities.rs:253`),
   rendered as steps named `(shard i of n)`
   (`velnor-actions-orchestrator/src/matrix_step.rs:43-62`). The nextest
   argv (`velnor-actions-rust/src/argv.rs:194-212`) never adds
   `--partition`, so every shard re-runs the full `cargo nextest run
   --profile ci` command. We confirmed this with
   `velnor-actions generate --output-dir`. Shard count is also capped by
   `[resources] test_process_budget` (`velnor-actions-contract/src/config/mod.rs:218-237`).
   Our `docs/ci.md` pins `default_shards = 1` for this reason.
4. **One closed nextest profile.** `NextestProfile` is `Ci | Default`
   (`velnor-actions-rust/src/profile.rs:112-136`), and `run_ignored` is one
   value for the whole stack (`velnor-actions-contract/src/config/stacks.rs`,
   `RustStackConfig::run_ignored`). We cannot run `--profile smoke
   --run-ignored only` as its own job next to the normal `ci` run.
5. **Custom jobs are too narrow for this.**
   - `[[checks]]` (`MiseCheck`, `config/mise.rs:13-36`) runs a Mise task,
     and every tool must be a `[[qualified_tools]]` entry
     (`config/mod.rs:163-172`, `unknown_qualified_tool`) with an exact
     `x.y.z` pin (`velnor-actions-mise/src/catalog_versions.rs:17-28`).
     Our task needs only the Rust toolchain plus cargo-nextest, which
     velnor already pins for the Rust stack (`rust@1.98.1`,
     `cargo-nextest@0.9.148` in our `ci.yml`), and `cargo xtask` (a
     workspace member). Re-declaring them as qualified tools duplicates
     pins, and we have no `mise.toml` at all.
   - `[[workflow.tasks]] kind = "build"` (`config/build_task.rs`) runs
     only on `macos-26-arm64` (`:29-45`) with at most 2 nextest threads
     (`:121-122`). We need ubuntu-24.04 with 4 PTY slots.
   - Checks still run only on the PR and push triggers.
6. **Required aggregation.** `Required` aggregates every generated job. We
   need the smoke tier in `Required`, while scheduled and manual jobs must
   never block a PR.
7. **Timeouts and concurrency** are fixed per job kind
   (`JobTimeout::CRATE = 30`, `contract/src/workflow/timeout.rs:40`). The
   concurrency group cancels in-progress runs only on PRs (`ci.yml:34-36`).
   Nightly needs ~60 min per shard, and a newer nightly must not cancel a
   running one.
8. **No escape hatch.** velnor owns all of `.github/workflows/`
   (`velnor-actions-orchestrator/src/generate_preserve.rs:168-181`,
   `generator_owned`). It retires files it did not emit, and the drift
   check `diff -r --brief .github` (`velnor-actions-workflow-renderer/src/closure.rs:192`)
   fails on any hand-written workflow. So we cannot add `visual.yml` by
   hand either.

## 3. Problem B: `.github/CLAUDE.md` must be allowed to be a symlink

Our convention is that every `CLAUDE.md` is a symlink to the `AGENTS.md`
beside it. velnor writes `.github/CLAUDE.md` as a regular file containing
`@AGENTS.md\n`:

- `velnor-actions-contract/src/formats.rs:17-25` (`CLAUDE_MD_PATH`,
  `CLAUDE_MD_POINTER_BODY`), rendered at
  `velnor-actions-workflow-renderer/src/tree.rs:133-137`;
- `velnor-actions-orchestrator/src/generate_leaf_links.rs` replaces a
  symlink at an emitted leaf with a fresh file;
- the drift check (`closure.rs:192`) then fails on a symlink, because
  `diff` follows it and compares `AGENTS.md` bytes with `@AGENTS.md\n`;
- goldens pin "never symlinks"
  (`velnor-actions-cli/tests/impl_repo_goldens.rs:1-38`).

The stated reason (plugin installers and Windows checkouts cannot carry
symlinks) is valid as a default. We are asking for an opt-in.

## 4. Desired outcomes and acceptance criteria

A consumer on the next velnor release can, from `.velnor/config.toml` alone:

- [ ] Declare extra test jobs that run a **named nextest profile** with
      their own `run_ignored`, on `ubuntu-24.04`, using the stack's
      already-pinned Rust and nextest. No `[[qualified_tools]]` or
      `mise.toml` entry is needed for tools the Rust stack already
      provides.
- [ ] Give those jobs **real partitions**: N jobs (a matrix) each running
      `--partition hash:k/N`. The test binaries are built once and shared
      (a `cargo nextest archive` artifact plus `--archive-file`) or reused
      from the shared target cache. Shard count is not tied to
      `test_process_budget`.
- [ ] Attach **triggers** per job group: `pull_request`/`push` (default),
      `schedule` (cron), and `workflow_dispatch` with typed inputs (a
      `choice` for tier, a `string` nextest filterset appended as `-E`).
- [ ] **Upload artifacts** from repository paths with a condition
      (`failure()` / `always()`), a name template that includes the shard
      index, and a retention period.
- [ ] Set a **per-job `timeout-minutes`**, test-group/thread hints (for
      example `NEXTEST_TEST_THREADS` or a profile override), and a
      concurrency policy (cancel-in-progress on PR only; never for
      schedule).
- [ ] Choose **required membership**: PR-triggered extra jobs can join
      `Required`. Schedule- and dispatch-only jobs never do.
- [ ] Keep **a few hand-owned workflow files** (an explicit allowlist)
      that generation preserves and the drift check ignores. Alternatively,
      first-class "extra jobs" must make this unnecessary. Say which
      approach you chose.
- [ ] Opt into **`.github/CLAUDE.md` as a symlink** to `AGENTS.md`.
      `generate` creates it, the drift check accepts it (comparing link
      target, not bytes), and the default stays the `@AGENTS.md` import
      file.
- [ ] A config that does not use the new keys generates **byte-identical**
      output to 0.1.8 (same marker version aside). Existing consumers see
      no diff.
- [ ] Every new key is validated fail-closed (bad cron, unknown profile
      not in `.config/nextest.toml`, shard count 0, path escaping the
      repo, input names) with errors in the existing `ContractError` style.

## 5. Our design ideas (suggestions, not mandates)

Pick a better design if your research finds one. Shapes:

```toml
# Problem A: a named test suite on its own trigger set
[[stacks.rust.suites]]
id = "visual-smoke"
profile = "smoke"                 # must exist in .config/nextest.toml
run_ignored = "only"
partitions = 2                    # -> matrix of 2 jobs, --partition hash:k/2
build = "archive"                 # build job + nextest archive shared by shards
timeout_minutes = 30
test_threads = 4
triggers = ["pull_request", "push"]
required = true
artifacts = [{ name = "visual-smoke-{shard}", paths = ["target/tuiscotti/actual", "target/tuiscotti/expected", "target/tuiscotti/diff", "target/tuiscotti/report.html"], when = "failure", retention_days = 14 }]

[[stacks.rust.suites]]
id = "visual-nightly"
profile = "full"
run_ignored = "only"
partitions = 12
build = "archive"
timeout_minutes = 60
triggers = ["schedule", "workflow_dispatch"]
schedule = ["0 2 * * *"]
required = false                  # schedule/dispatch suites can never be required
cancel_in_progress = false

[workflow.dispatch.inputs]
tier   = { type = "choice", options = ["smoke", "full", "narrow", "capture"], default = "smoke" }
filter = { type = "string", default = "" }   # appended as -E '<filter>' after validation

# Escape hatch, if you keep one
[workflow]
preserve_workflows = [".github/workflows/visual-extra.yml"]  # exact paths, drift check skips them

# Problem B
[agents]
claude_md = "symlink"   # "import" (default, today's @AGENTS.md file) | "symlink"
```

Other notes:

- Scheduled and dispatch suites could go into a second generated workflow
  file (for example `visual.yml`), so `ci.yml` and `Required` are
  unchanged. A suite's `id` would name both its jobs and its artifacts.
- Dispatch inputs reach `run:` lines only through `env:`, never through
  `${{ inputs.* }}` interpolation. That keeps the zizmor and actionlint
  invariants.
- `capture` mode (failures allowed, upload the whole `actual/`) can be a
  suite flag: `allow_failure = true` plus `when = "always"`.

## 6. How to work

1. **Research first.** Read `AGENTS.md`, the specs at the repo root, and
   the crates above (config contract, orchestrator `workflow.rs`, renderer
   steps and closure, rust argv/profile, generate and preserve). Check
   every file:line in this prompt against `main`, since it may have moved
   past v0.1.8.
2. **Analyse** how schema 2 (`[execution]`, `velnor-actions-orchestrator/src/routing.rs`,
   `render_schema2_workflows`) relates to this. If schema 2 is the right
   home for multiple workflows, design there and tell us how to migrate
   (the `config migrate` subcommand, `velnor-actions-cli/src/dispatch_config.rs:15`).
3. **Brainstorm at least two alternative designs** for each problem: suites
   under `stacks.rust` vs generic `[[jobs]]`; matrix vs N named jobs;
   archive vs shared cache; allowlist vs first-class jobs. Weigh them
   against velnor's invariants (generic generator, strict safety, hash-pinned
   actions, fast CI, zero legacy) and choose the best. Record why in your
   design notes or DECISIONS file.
4. **Implement** behind new optional keys with defaults that keep current
   output byte-identical. Do not break existing consumers or the release
   and qualification workflows.
5. **Test:** contract validation tests (good and bad configs), renderer
   golden tests for a smoke, nightly and dispatch config, an actionlint and
   zizmor pass on the goldens, drift-check tests for a symlinked
   `CLAUDE.md` and for a preserved workflow, and a regression golden that
   proves an unchanged config yields unchanged output. Run the repo's own
   gates.
6. **Document:** the config reference, a "visual or long test tiers" example,
   the CLAUDE.md option, and migration notes if any key moves.
7. **Release** a tagged version with its `release-manifest.json`.

## 7. Report back

Reply with:

- the new version tag and release commit;
- the exact config keys added (with defaults) and one complete example
  config for the tier table in §1;
- what a consumer must change: config edits, `release-manifest.json`
  replacement, regenerate commands, branch-protection changes (check
  names);
- whether `.github/CLAUDE.md` can now be a symlink, and how the drift check
  treats it;
- any request you declined or changed, and why.

Do not open issues or PRs against our repository. We will apply the
upgrade ourselves.

## 8. Problem C: CI ignores the repository's Rust toolchain

This is the smallest of the three changes and the most urgent: our
dependency policy keeps the Rust toolchain on the latest stable, and CI is
the one place we cannot follow it. Re-verify every citation at tag `v0.1.8`
and on `main`.

- **The toolchain is hardcoded.** `crates/velnor-actions-mise/src/catalog.rs:50`
  sets `RUST_VERSION = "1.98.1"`, and every generated job exports
  `RUSTUP_TOOLCHAIN: 1.98.1`. That variable overrides `rust-toolchain.toml`,
  so the file is ignored.
- **Repository pins are ignored.** Our `rust-toolchain.toml` now pins
  `1.99.1` (the latest stable). Adding a repository `mise.toml` with a
  newer `rust` does not change the output either. We confirmed both with
  `velnor-actions generate --output-dir` into a scratch directory and
  compared the rendered jobs.
- **Tool versions are fixed too.** The Rust stack pins
  `cargo-nextest@0.9.148`, and the lint jobs pin `actionlint 1.7.12`,
  `shellcheck 0.11.0`, `zizmor 1.30.1` and `gh 2.102.0`. A consumer cannot
  raise any of them between velnor releases.

Consequences for us today (recorded in our `docs/dependencies.md`, "Rust in
CI"):

- `rust-version` in `Cargo.toml` stays at `1.98`, one release behind
  stable, so CI can build and Cargo's MSRV-aware resolver keeps the
  lockfile buildable on 1.98.
- Before every push we run clippy a second time under CI's toolchain
  (`RUSTUP_TOOLCHAIN=1.98.1 cargo clippy --workspace --all-targets -- -D warnings`),
  because lints differ between 1.98 and 1.99.
- CI lags stable by however long a velnor release takes, and every
  consumer moves in lockstep with velnor rather than with Rust.

### Acceptance criteria

- [ ] When the repository has a `rust-toolchain.toml` (or
      `rust-toolchain`), CI uses the channel it pins. velnor does **not**
      export `RUSTUP_TOOLCHAIN` in that case, so rustup resolves the file
      as it does locally.
- [ ] Alternatively or additionally, a `[rust] toolchain = "<channel>"` key
      (or `[stacks.rust] toolchain`) in `.velnor/config.toml` sets it
      explicitly. If both are present and disagree, `generate` fails
      closed with a `ContractError` naming both values.
- [ ] With neither present, the default is the latest stable known at
      velnor's release, and a velnor release bumps it promptly.
- [ ] Components the jobs need (`clippy`, `rustfmt`) are installed for
      whichever toolchain is selected.
- [ ] The tool versions above (cargo-nextest, actionlint, shellcheck,
      zizmor, gh) can be overridden per repository with exact `x.y.z`
      pins validated against the catalog, and velnor's defaults track the
      latest releases.
- [ ] A config without a toolchain file or the new keys generates
      byte-identical output to 0.1.8 (same marker version aside).

### Report back for Problem C

- which source of truth wins (`rust-toolchain.toml`, the config key, or
  both with a conflict error), and the exact key names and defaults;
- how tool-version overrides are spelled and validated;
- the Rust and tool versions the new release defaults to.

## 9. Problem D: `Publish baseline` refuses the configured default branch

Seen on velnor-actions 0.1.8 (run
<https://github.com/tailrocks/terminal-components-claude/actions/runs/38943668040>):
every other job is green, `Publish baseline` fails.

- `.velnor/config.toml` sets `[workflow] default_branch =
  "design/interaction-polish"`; the repository's GitHub default branch is
  `main`.
- The generated job runs on `github.event_name == 'push' && github.ref ==
  'refs/heads/design/interaction-polish'` (the config value).
- Its `Publish baseline` step builds the request from the event payload
  (`write_publish_request`, `repository.default_branch` = `main`) and
  `publish_gate` refuses any ref other than `refs/heads/<that>`:
  `velnor-actions: internal request failed: internal:
  publish_refused:unprotected_ref`.
- `Upload baseline` then fails with `No files were found with the provided
  path: …/published-baseline/baseline.json` (`if-no-files-found: error`).

The renderer and the orchestrator disagree on which branch is the default.
A consumer cannot fix it in config: pointing `default_branch` at `main`
stops push CI on the working branch.

### Acceptance criteria

- [ ] The publish gate and the job's `if:` use one source of truth (the
      configured `default_branch`, carried into the request), or the job is
      not generated / skipped when they cannot agree.
- [ ] A push to the configured default branch publishes the baseline and
      uploads it; a push elsewhere never runs the job.
- [ ] A failed publish never leaves the upload step failing on a missing
      file as a second error.
