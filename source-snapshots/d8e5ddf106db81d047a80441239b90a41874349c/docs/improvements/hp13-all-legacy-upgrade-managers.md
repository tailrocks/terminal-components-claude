# HP13 — All legacy upgrade managers

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Complete every mapped manager alone and in aggregate with exact stages, platform availability, failure barriers and simulated version changes. Fix the standalone mise loop.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real manager discovery, installation/upgrade commands and host mutation.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve Brew packages/casks, mise, Amp, Oh My Zsh and upgrade-all. Current apt+mise graph misses macOS workflows; standalone mise upgrade loops.

**Source evidence and mandatory scope:** [matrix HP13](../parity/holla-parity-matrix.md#hp13--all-legacy-upgrade-managers) — `OP51`, `OP52`, `OP55`, `OP53`, `OP54`, `OP56`, `OP57`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Build host-specific plan from detected managers, and keep each manager directly searchable. Preserve Brew update, greedy --yes upgrade, cleanup, autoremove, doctor; cask variant only macOS. Preserve amp update, mise upgrade and sh <resolved ZSH>/tools/upgrade.sh. Show $ZSH override exactly. Aggregate independently runnable managers; review before apply and keep per-stage output.

**Architecture / reusable components:** Use shared typed stage specifications for standalone and aggregate variants; remove shell-chain duplication. Explicitly block dependent Brew stages after prerequisite failure while retaining independent managers. Refresh availability before execution and invalidate changed reviewed plans. Reuse existing upgrade Plan/Activity.

**Required deterministic fixture:** `parity-upgrade-managers` — every manager alone, all managers, none, Linuxbrew versus macOS casks, ZSH override/fallback, disappeared executable, failed update/doctor, independent successful manager, standalone mise follow-up.

**Acceptance / automated verification:** Assert every command/flag/order/cwd and exact preview, correct availability, parallel manager branches and blocked dependent stages. Standalone mise Upgrade must reach execution and change versions. Report partial upgrade and verification failure without generic success. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture macOS aggregate plan, cask-only route, custom ZSH path, failed Brew prerequisite, independent completion and mise upgrade result.

**Dependencies:** HP01/HP14/HP15/HP17/HP23; F23.

## Evidence

**Slice status:** current simulated slice complete for the journeyed variants; gap: the `parity-upgrade-managers` fixture is one macOS host and does not carry the Linuxbrew, no-manager, `$ZSH` fallback, disappeared-executable or failed-update variants named in the required fixture, and the standalone cask and amp argv are not asserted by any test · Later operational clauses open.

**Scenario and journey:** `parity-upgrade-managers` (fixture fn `upgrade_managers` in src/bin/holla/domain/parity.rs) · `hp13_upgrade_managers_run_exact_stages_and_the_plan_branches` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/plan.rs: failure_blocks_dependents_and_retry_restores_them`, `src/bin/holla/domain/plan.rs: parallel_eligibility_respects_ancestry_and_locks`, `src/bin/holla/app_tests_parity.rs: hp23_platform_capabilities_are_stated_and_never_faked` (cask availability by platform). · row proofs in src/bin/holla/app_tests_rows.rs: `op54_op56_op57_standalone_upgrade_rows_are_exact`.

**What the journey asserts:**
- `upgrade.brew-packages` argv is exactly `brew update @/Users/alex`, `brew upgrade --greedy --yes @/Users/alex`, `brew cleanup @/Users/alex`, `brew autoremove @/Users/alex`, `brew doctor @/Users/alex`.
- `upgrade.oh-my-zsh` argv is `sh /Users/alex/.config/omz/tools/upgrade.sh @/Users/alex` ("$ZSH wins over ~/.oh-my-zsh").
- Running "Upgrade Homebrew packages" creates a batch with `members.len() == 5`; member 1 settles `Succeeded`, member 4 settles `Failed` ("brew doctor fails on this host"); the effect landed: `world.upgrade.brew_outdated.is_empty()`.
- "Upgrade mise-managed tools" argv is `mise upgrade @/Users/alex`, settles `Succeeded`, and every `mise.global_tools` entry has `active == latest`.
- "Upgrade everything" opens a plan page whose text contains `Upgrade everything · mbp` and `9 steps`; plan `upgrade-all` contains steps `brew-update`, `brew-upgrade`, `brew-casks`, `mise-upgrade`, `amp-update`, `omz-upgrade`, `verify`; step `brew-doctor` has `fails == true`; step `mise-upgrade` has empty `deps` ("managers run beside the brew chain").
- `hp23_platform_capabilities_are_stated_and_never_faked`: `upgrade.brew-casks` is present in the macOS catalogue ("casks are a macOS action") and absent in the Linux catalogue.
- `plan.rs: failure_blocks_dependents_and_retry_restores_them`: after step 1 fails, steps 2, 3 and 7 are `Blocked` while step 4 stays `Ready` ("the mise branch keeps going"); `retry(1)` restores them and succeeded work is not repeated.

**Captures:** `shots/h_hp13_brew_batch` (first member of the sequential brew batch, `brew update · succeeded`, output following) · `shots/h_hp13_plan` (the "Upgrade everything" plan, 9 steps in lanes a to d, `brew upgrade --cask` beside the brew chain, status line reports the earlier batch as `4 ok · 1 failed · 0 cancelled`) · base matrix `shots/h_parity_upgrade-managers_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| OP51 | `hp13_…`: `upgrade.oh-my-zsh` argv resolves `$ZSH` (`/Users/alex/.config/omz/tools/upgrade.sh`); `hp23_…`: `upgrade.brew-casks` present on macOS, absent on Linux. The `~/.oh-my-zsh` fallback and the no-manager host are not journeyed. | Real PATH/directory probing at execution time. |
| OP52 | `hp13_…`: plan `upgrade-all` has `9 steps`, contains `brew-update`, `brew-upgrade`, `brew-casks`, `mise-upgrade`, `amp-update`, `omz-upgrade`, `verify`; `mise-upgrade.deps` empty; `plan.rs: failure_blocks_dependents_and_retry_restores_them` blocks dependents and keeps the independent branch `Ready`. The plan is opened, not confirmed; execution of the aggregate is not asserted. The legacy single `&&` chain is redesigned into typed steps. | Re-probe at execution and real parallel child processes. |
| OP55 | `hp13_…`: argv `mise upgrade @/Users/alex`, `Succeeded`, every global tool `active == latest` (standalone mise no longer loops to the snapshot). | Real `mise upgrade`. |
| OP53 | `hp13_…`: five exact stage argv with cwd `/Users/alex`; batch of 5; member 1 `Succeeded`, member 4 (`brew doctor`) `Failed`; `brew_outdated` emptied. Availability on Linux with Brew is not journeyed. | Real Homebrew commands and host mutation. |
| OP54 | `op54_op56_op57_standalone_upgrade_rows_are_exact`: the `upgrade.brew-casks` batch carries `brew update` and `brew upgrade --cask --greedy --yes`; `hp23_…`: the item exists on macOS and not on Linux. Stage results of the standalone cask batch are not run in a journey. | Real cask upgrade; compile-time macOS gating. |
| OP56 | `op54_op56_op57_standalone_upgrade_rows_are_exact`: `upgrade.amp` argv is `amp update`; `hp13_…`: plan step `amp-update` present. | Real `amp update`. |
| OP57 | `hp13_…`: argv `sh /Users/alex/.config/omz/tools/upgrade.sh @/Users/alex` with the `$ZSH` override; `op54_op56_op57_standalone_upgrade_rows_are_exact`: the standalone row is `sh <dir>/tools/upgrade.sh`. Fallback path not journeyed. | Real script execution. |

**Deferred remainder:**
- Real manager discovery, installation/upgrade commands and host mutation (Later).
- Not proven by tests: aggregate plan execution with a failed Brew prerequisite blocking dependents while independent managers complete (only the plan.rs unit test proves the blocking rule on the generic upgrade plan); Linuxbrew versus macOS cask route; `$ZSH` fallback; disappeared executable; failed `brew update`; standalone cask and amp argv; partial-upgrade and verification-failure reporting text.
- Captures named in the contract but absent from tools/holla_parity_flows.sh: cask-only route, custom ZSH path, failed Brew prerequisite, independent completion, mise upgrade result.

**Limits:**
- The virtual clock and scripted outcomes cannot prove real command timing, exit codes or PATH state on a host.
- Availability by platform is a fixture flag, not a compile-time or runtime probe.
