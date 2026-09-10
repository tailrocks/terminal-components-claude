# HP17 — Custom actions, configuration and trust

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Use fixture configurations and in-memory versioned trust to show exact argv/cwd/source/digest, diagnostics, risk/confirm rules and review invalidation. Simulate edits, relocation, restart and save failures without live config/history I/O.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Live global/project config reads, persistent approvals, filesystem migration and trust CLI operations.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve global/project custom actions, declared risk/confirm, exact argv, scoped execution and durable content trust. Session path trust is insufficient.

**Source evidence and mandatory scope:** [matrix HP17](../holla-parity-matrix.md#hp17--custom-actions-configuration-and-trust) — `E11`, `E12`, `E13`, `E15`, `E16`, `E17`, `E18`, `E19`, `E14`, `E20`, `E21`, `E22`, `E23`, `E24`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Read XDG_CONFIG_HOME/holla/actions.toml or ~/.config/holla/actions.toml plus cwd .holla.toml; retain every required/optional [[action]] field and default. Present configuration resources, diagnostics with source/index, valid siblings, grouping and keywords. Reserve IDs against actual registry rather than stale hardcoded names. Show program/each argv/cwd/source/trust/risk before execution; explicit sh -c remains possible and clearly identified.

**Architecture / reusable components:** One validated action specification drives preview and execution; never interpolate argument strings. Bind trust to reviewed whole-file digest, origin/path and effective cwd/argv; re-read on execution so edits revoke review. Legacy digest-only trusted.json may be read as migration evidence, but broader path binding requires renewed review. Persist sorted/versioned approvals atomically; corruption untrusted, save failure no launch. Global config remains user-owned trusted input; inherited environment has no invented TOML overrides.

**Required deterministic fixture:** `parity-custom-actions` — minimal/full schema, invalid ID/type/blank field, duplicate global/project/builtin ID, malformed sibling/file, XDG/fallback, spaces/newlines/metacharacters, comment edit, identical bytes moved, config edited during review, corrupt store/write failure.

**Acceptance / automated verification:** Assert accepted argv preserved exactly, safe/mutating/destructive plus confirm combinations, precise cwd and no implicit shell. Review displays whole trust scope; cancellation changes nothing; unchanged authorized content survives restart, changed content/path re-prompts. --yes authorizes one run only; explicit durable approval supports automation and revocation without alias bypass. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture custom provenance, exact argv facts, source-index diagnostics, Cancel-default trust review and changed-definition rejection.

**Dependencies:** HP01/HP14/HP16/HP21; F01/F08/F09/F23.

## Evidence

**Slice status:** current simulated slice complete for the journeyed variants; gap: the XDG versus fallback path choice, a comment-only edit, a corrupt store through the UI and `--yes` one-run authorisation are not journeyed (schema, diagnostics, corrupt store and save failure are unit-tested) · Later operational clauses open.

**Scenario and journey:** `parity-custom-actions` (fixture fn `custom_actions` in src/bin/holla/domain/parity.rs) · `hp17_custom_actions_are_exact_argv_reviewed_by_digest_and_never_run_unsaved` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/custom.rs: parses_the_schema_with_defaults_and_argv_only_commands`, `diagnostics_are_indexed_and_valid_siblings_survive`, `trust_binds_digest_path_and_cwd_and_survives_persistence`. · row proofs in src/bin/holla/app_tests_rows.rs: `e20_unreviewed_project_actions_are_labelled_before_trust`.

**What the journey asserts:**
- The catalogue contains `notes`, `shared`, `deploy.preview`, `wipe`, `multi`, `custom.config` and not `Bad Id`; `shared` provenance contains `global` ("the global definition wins"); project diagnostics mention `shared` and `Bad Id`.
- `deploy.preview` argv is exactly `tools/deploy-preview.sh --env 'preview stack' '$(whoami)' @/Users/alex/work/team`; `multi` args are `["%s\n", "a\nb c"]`.
- Trust status for the project digest at its current path is `Moved` or `LegacyDigestOnly` (same bytes trusted elsewhere, plus a legacy digest-only record).
- Opening "Deploy preview" shows a page containing `Trust`, `sha256:`, `argv[3]` and `$(whoami)`.
- Editing the file during review then approving shows `changed during review`, returns to Here, and the status is not `Trusted`.
- Reviewing again approves: the activity argv is the exact string above, status is `Trusted`, and `persisted.trust` contains the digest and `/Users/alex/work/team/.holla.toml`.
- With `trust.save_failure = Some("EROFS")` approval shows `nothing was run` and stays on Here.
- "Wipe local caches" confirms once and runs `sh -c 'rm -rf .cache && echo done' @/Users/alex/work/team`.
- `custom.rs: parses_the_schema_with_defaults_and_argv_only_commands`: defaults `confirm == false`, project group `Current folder`, global group `Custom`, custom group `Maintenance`, keywords kept, `index == 1`, digest length 64.
- `custom.rs: diagnostics_are_indexed_and_valid_siblings_survive`: `/p/.holla.toml action[0]: invalid id`, `action[1]` needs an `argv array`, `action[3]` `duplicate`, `action[4]` `unknown danger`, `action[5]` `blank`/`nonempty program`/`confirm`; `built-in` and `global configuration` collisions; syntax error `/p/.holla.toml: line 2` with no index; `array of tables`; `# nothing` is zero actions and zero diagnostics; argv `["echo", "a b", "x\ny", "$(rm)"]` preserved.
- `custom.rs: trust_binds_digest_path_and_cwd_and_survives_persistence`: `Untrusted` before approval, `Trusted` after, `Moved` at another path, `Untrusted` for a new digest, survives `load`, legacy `{"v":1,"hashes":[...]}` is `LegacyDigestOnly`, `{nope` and `{"v":9}` are corrupt and untrusted, a save failure refuses approval, entries serialize sorted.

**Captures:** `shots/h_hp17_trust` (`Trust ~/work/team/.holla.toml?`, `not trusted yet`, digest, trust scope, `argv[0]` to `argv[3]` debug-quoted, two `action[index]` diagnostics, exact file content) · `shots/h_hp17_trusted_running` (the action launched with the exact argv; in this fixture no script is defined for `tools/deploy-preview.sh`, so the frame shows `exit 127` with `no simulated outcome`) · `shots/h_hp17_config` (`Custom actions` page: origin, digest, `Trust  Trusted`, `3 · 2 diagnostics`, per-action danger and argv) · base matrix `shots/h_parity_custom-actions_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| E11 | Fixture reads `/Users/alex/.config/holla/actions.toml`; `hp17_…` lists `notes` and `shared` from it; `diagnostics_are_indexed_…`: `# nothing` is a successful zero configuration. `XDG_CONFIG_HOME` precedence is not asserted. | Live global config read. |
| E12 | Fixture project file is `<cwd>/.holla.toml`; `hp17_…`: project ids present, `shared` resolved global-first ("the global definition wins"). No-ancestor-traversal not asserted. | Live project config read. |
| E13 | `parses_the_schema_with_defaults_and_argv_only_commands`: id/label/command/danger required, description/keywords/group/confirm optional with `confirm == false` default. | none |
| E15 | `hp17_…`: `Bad Id` never becomes an action, project `shared` diagnosed; `diagnostics_are_indexed_…`: `invalid id`, `duplicate`, `built-in`, `global configuration`, `blank`. | none |
| E16 | `diagnostics_are_indexed_…`: `/p/.holla.toml action[0]`, valid sibling `ok` survives, whole-file error `line 2` with `index.is_none()`, `array of tables`; `h_hp17_trust` shows `action[2]`/`action[3]` diagnostics. | none |
| E17 | `parses_the_schema_…`: `Current folder`, `Custom`, `Maintenance`, keywords retained. Alphabetical group order is not asserted. | none |
| E18 | `hp17_…`: project argv carries `@/Users/alex/work/team` (config parent) into the activity. Global cwd equals invocation cwd is not asserted. | Environment inheritance to a real child. |
| E19 | `hp17_…`: trust page contains `argv[3]` and `$(whoami)`; `h_hp17_trust` shows `argv[0]` to `argv[3]` as debug-quoted strings. | none |
| E14 | `hp17_…`: literal `$(whoami)` and `preview stack` stay single arguments; `multi` args `["%s\n", "a\nb c"]`; explicit `sh -c '...'` runs as argv; `diagnostics_are_indexed_…`: string `command` rejected (`argv array`). | Real `Command::new(program).args(args)`. |
| E20 | `e20_unreviewed_project_actions_are_labelled_before_trust`: the finder row for "Deploy preview" reads `⚠ unreviewed` before trust; `hp17_…`: opening it lands on the trust page (`h_hp17_trust` shows `not trusted yet`); global `shared` provenance `global`. | none |
| E21 | `hp17_…`: review shows exact argv and digest; approval is Right then Enter (`confirm` helper documents Cancel as the default); the changed-during-review path runs nothing. Cancel/Escape running nothing is not asserted directly. | Terminal restoration. |
| E22 | `hp17_…`: file edited during review gives `changed during review` and status not `Trusted`; re-review with current bytes is `Trusted`; `trust_binds_digest_…`: new digest `Untrusted`, same digest elsewhere `Moved`. A comment-only edit is not journeyed (the edit changes a label). | Re-read from disk at execution. |
| E23 | `hp17_…`: `persisted.trust` holds digest and path; `EROFS` save failure gives `nothing was run`; `trust_binds_digest_…`: `{nope` and `{"v":9}` corrupt and untrusted, legacy `v:1` is migration evidence, sorted serialization. | Cache-dir path, temp file plus rename, directory creation. |
| E24 | Not represented; no test models `--yes`. | Headless `--yes` acceptance and persistence. |

**Deferred remainder:**
- Live global/project config reads, persistent approvals, filesystem migration and trust CLI operations (Later).
- Not proven by tests: `XDG_CONFIG_HOME` versus fallback; cancellation (Escape) changes nothing; unchanged authorised content surviving a restart (persistence is proven only by `TrustStore::load` of a serialized store); `--yes` authorises one run; durable approval and revocation without alias bypass through a CLI.
- Captures named in the contract but absent from tools/holla_parity_flows.sh: Cancel-default trust review as a decisive frame and changed-definition rejection.

**Limits:**
- File bytes, digests and the trust store live in the in-memory `World`; no real file I/O, atomic rename or cache directory is exercised.
- `tools/deploy-preview.sh` has no scripted outcome in the fixture, so the launched action fails with `exit 127` in the capture; the journey asserts argv and trust state, not the run result.
