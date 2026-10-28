# Development

## Workspace commands

Install the checked-out workspace's pinned developer tools with `mise install`, then use its declared Rust toolchain. Use `cargo nextest` for tests, as required by the design-reference and WIP instructions. Before running a gate, inspect `AGENTS.md`, `Cargo.toml`, `rust-toolchain.toml`, and CI configuration because package layout and profiles differ.

| Workspace | Location | Test command policy |
|---|---|---|
| `main` | Workspace packages, with default members for facade/apps/conformance | `cargo run -p showcase`, `cargo run -p tablepro`, `cargo run -p holla`, `cargo run -p jackin-preview`; `cargo check --workspace --all-targets`; `cargo nextest run --workspace`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`. |
| `design/interaction-polish` and six `wip/*` branches | `junie-tui` package with `xtask` workspace member | `cargo run -p junie-tui --bin showcase` (substitute `tablepro`, `holla`, or `jackin-preview`); `cargo check --workspace --all-targets`; `cargo nextest run --workspace`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`. Follow branch-specific instructions for visual and repository-specific gates. |

The commands above are starting points verified against the package manifests. Format, check, lint, and test the affected workspace using its configured commands. Run focused tests first, then required integration and visual gates. Record exact command, branch, revision, and result. A passing local command is evidence only for that command and revision; old visual and nextest results are not current verification.

## Common functional suite

The standalone suite in [`tools/functional-suite/`](../tools/functional-suite/README.md) has one `functional` Nextest target. Use its README to export source and prepare the pinned engines. From the exported suite root, after building the four application binaries from one source snapshot and profile, use a fresh output directory for each run:

```sh
BIN_DIR="/absolute/path/to/showcase-holla-jackin-tablepro"
OUT_DIR="/absolute/path/to/new-output"
CARGO_BUILD_JOBS=2 TERMROCK_PARITY_BIN_DIR="$BIN_DIR" \
TERMROCK_PARITY_OUT_DIR="$OUT_DIR" \
"$HOME/.local/bin/rust-build-guard" -- mise exec -- cargo nextest run \
  --locked --offline --test functional --no-tests fail --retries 0 \
  --no-fail-fast --test-threads 2 -E 'test(/^first_delivery_/)'
```

This filter selects the four first-delivery cases. Keep source, engines, binaries, and profile fixed across a paired run; change only `OUT_DIR`. The broader case inventory and current evidence are in the suite README and [migration record](./architecture/termrock-migration-record.md#common-functional-suite-current-results-2026-10-28).

The facade's named headless journey integration tests require its `testing` feature: [`crates/termrock/Cargo.toml` declares the feature](https://github.com/tailrocks/terminal-components-claude/blob/c8a31dd3fa6c65cf926228ca660bc7785dacb295/crates/termrock/Cargo.toml#L8-L10), and its [`ex01_button`, `ex02_text_input`, and consumer test targets](https://github.com/tailrocks/terminal-components-claude/blob/c8a31dd3fa6c65cf926228ca660bc7785dacb295/crates/termrock/Cargo.toml#L35-L75) mark that requirement. The manifest explains that these example journeys use the testing harness surface. This is a target prerequisite, not evidence that an example or test was run.

CLI option discovery is application-specific. The [Showcase parser](https://github.com/tailrocks/terminal-components-claude/blob/c8a31dd3fa6c65cf926228ca660bc7785dacb295/crates/showcase-ui/src/app.rs#L2889-L2981) parses `--page`, `--theme`, `--color`, `--motion`, and `--frame`; that parser has no `--help` branch. Holla, Jackin Preview, and TablePro each have explicit help outcomes in their [parser](https://github.com/tailrocks/terminal-components-claude/blob/c8a31dd3fa6c65cf926228ca660bc7785dacb295/crates/holla-ui/src/cli.rs#L100-L108), [binary entry](https://github.com/tailrocks/terminal-components-claude/blob/c8a31dd3fa6c65cf926228ca660bc7785dacb295/crates/holla/src/main.rs#L16-L21), [parser](https://github.com/tailrocks/terminal-components-claude/blob/c8a31dd3fa6c65cf926228ca660bc7785dacb295/crates/jackin-preview-app/src/cli.rs#L99-L106), [binary entry](https://github.com/tailrocks/terminal-components-claude/blob/c8a31dd3fa6c65cf926228ca660bc7785dacb295/crates/jackin-preview/src/main.rs#L5-L21), [parser](https://github.com/tailrocks/terminal-components-claude/blob/c8a31dd3fa6c65cf926228ca660bc7785dacb295/crates/tablepro-ui/src/cli.rs#L143-L151), [runner](https://github.com/tailrocks/terminal-components-claude/blob/c8a31dd3fa6c65cf926228ca660bc7785dacb295/crates/tablepro-ui/src/cli.rs#L44-L55), and [binary entry](https://github.com/tailrocks/terminal-components-claude/blob/c8a31dd3fa6c65cf926228ca660bc7785dacb295/crates/tablepro/src/main.rs#L1-L13). These are source entry-point facts, not fresh launch results or a promise that every flag combination is valid.

## Backend-free dependency checks

A backend-free consumer claim needs an isolated consumer's resolved dependency graph without workspace feature unification; an in-workspace compile or an optional dependency declaration is not enough. The current `termrock-core` manifest enables its optional Crossterm adapter by default ([feature declaration](https://github.com/tailrocks/terminal-components-claude/blob/f4611b1fe98a98b78c9e2c32a6c9522fe72fd863/crates/termrock-core/Cargo.toml#L8-L16)). Bind the check to the current owning gate before publishing an invocation; historical commands and tool pins are not current instructions.

## Frozen CLI parity captures

`tools/frozen_baseline.py` extracts the signed `visual-baseline` tag into an isolated directory, builds its four CLI binaries, then runs the frozen `visual_baseline` Nextest target against both frozen and candidate binary directories. The frozen target uses real `tuiscotti::Tui` processes (`tests/visual_baseline/scenarios/mod.rs`, `tests/visual_baseline/support.rs`) and `GroupedStore::check_with(..., 1.0)` with `ArtifactSet::AnsiCursor`; it checks the full frame, including cursor and attributes. It pins `tuiscotti` to `a1ea27f757d75d07b05499276cd932e9f8e067ad`.

Run from either candidate checkout. Keep the evidence root outside the checkout if comparing multiple branches:

```sh
python3 tools/frozen_baseline.py --repo "$PWD" --root /tmp/termrock-frozen-pair prepare
python3 tools/frozen_baseline.py --repo "$PWD" --root /tmp/termrock-frozen-pair build
python3 tools/frozen_baseline.py --repo "$PWD" --root /tmp/termrock-frozen-pair build-candidate
python3 tools/frozen_baseline.py --repo "$PWD" --root /tmp/termrock-frozen-pair compare \
  --candidate-bin-dir /tmp/termrock-frozen-pair/candidate-target/debug \
  --case holla::parity::files::120x40::truecolor
```

Repeat `build-candidate` and `compare` in the other checkout with its own evidence root. `--case` requires one exact ignored test name. Runner sets `VISUAL_FULL=1`, strips `VISUAL_TOOL` and update controls, requires that selected frozen `.ansi` snapshot, and refuses empty selection. Same frozen test executable and case run first against isolated frozen binaries, then candidate binaries. Each phase gets separate `actual/` and `diff/` artifacts under `<root>/paired/<run-id>/`; failed phases save artifacts and `report.json` before returning nonzero. The report records tag object, source commit/tree, tool pin, selected expected snapshot hash, harness/source hashes, candidate commit and dirty source fingerprint, and all binary hashes. Existing phase evidence cannot be overwritten.

The runner keeps the original frozen source and snapshots immutable. `prepare-live` makes a separately hashed copy, applies only `tools/frozen_cli_pair.patch`, and copies the versioned frame helper into that copy. The harness patch routes its central frame gate through the helper and starts a thread-local checkpoint recorder for each scenario. The original snapshot gate still runs first whenever that frame already has an approved snapshot.

Use `live-compare --all` for the immutable 28,895 frozen scenarios plus the separately hashed 174-case Table sort extension. The 28,895-case inventory and 20 ordinary checks remain pinned independently; the extension manifest must match its pinned hash, form the complete declared journey/size/color matrix, and register every extension ID exactly once as an ignored CLI test. It runs three ordered phases against real CLI processes: frozen binary capture into a new runtime expectation store, frozen binary replay against that store, then candidate binary comparison against the same store. It compares each scenario's ordered `(case, ordinal, logical frame, ANSI SHA-256, semantic Frame SHA-256)` receipts; semantic hashes include every cell and hidden cursor fields, and the helper writes a provenance-normalized actual Frame sidecar into a separately sealed inventory across all phases, and extension receipts must exactly match every checkpoint in their journey, with the single declared reveal checkpoint inserted after `initial` below 120 columns; omitted, reordered, or extra stages are rejected by exact list equality. Candidate phase must leave the expectation tree byte and path-type identical. Every case must report all required checkpoints; a missing, duplicate, unexpected, skipped, or failed case leaves registered CLI coverage incomplete. Deterministic chunks default to 32 cases and stay within the runner's 230-second command limit. Reports expose `registered_cli_complete` for this test scope and keep both `acceptance_complete` and `overall_acceptance_complete` false; these CLI checks alone do not certify full architecture or every application flow. `--case` may be repeated for a partial pilot.

The 20 non-ignored checks run separately with all `TERMROCK_PAIR_*` flags absent. Eleven source-only checks run once without `VISUAL_BIN_DIR`; nine CLI checks, including `isolation::env_isolation` which indirectly launches Showcase through `VISUAL_TOOL=capture`, run independently against both frozen and candidate binary directories. The report calls coverage complete only after all registered scenarios, all three receipt inventories, and all three ordinary-check groups pass. Original frozen snapshots are hashed before and after each phase and never used as runtime expectations for pending scenarios. Reports, per-chunk stdout/stderr, receipts, and Tuiscotti artifacts stay under `<root>/live-paired/<run-id>/`. Run runner guards with `python3 tools/test_frozen_baseline.py`.

`tools/frozen_cli_references.json` contains one evidence-bound exception for the reviewed `showcase::resize::overview_grown::120x40::truecolor` checkpoint, whose existing approval was independently repeated as stale against the frozen binary. The approval is immutable and its original gate remains red. The exception applies only during baseline capture, in an isolated one-case invocation, and only when the frozen source, test inventory, all four application binary hashes, exact checkpoint identity, reviewed ANSI and semantic frame hashes, and decision-file digest match. Replay and candidate checks remain strict against the newly captured runtime expectation. No other scenario inherits this reference, including `overview_shrunk`. The harness uses a quiet-settle wait for resize cases so a no-op resize can still produce a required frame; readiness assertions and checkpoint requirements remain in place.

## Consumer slots

`Ui::layer_slot(id, |slot| ...)` draws into an already-open layer's resolved rectangle and focus scope. The consumer-facing `PickerSlotExt::picker` adapter delegates to `Picker::draw` once; applications provide their picker, state, and domain items without receiving the rectangle. Open the layer with `Picker::layer(cx, items)`, which measures from the current viewport and design tokens, then keep calling `Picker::update` so resize updates the same layer size. Measurement remains in the existing Picker contract; the slot does not run input handlers or application logic. Component authors needing a raw renderer callback can import `LayerSlotAuthor` from `termrock::author`.

## Rust comments

Keep Rust comments only for a safety invariant, legally required notice, or non-obvious behavior constraint that cannot be recovered from code or tests and is necessary to maintain correctness. Do not replace removed prose with `#[doc = "..."]`. Preserve structural compiler attributes such as `#[doc(hidden)]` and actual runtime metadata such as `#[arg(help = "...")]`. Existing Clap derive documentation consumed as CLI help metadata is retained where removing it would change user-visible help; that narrow use does not make general API documentation in source the policy. Frozen Rust fixtures whose exact bytes are validation inputs remain unchanged. Put contracts, API guidance, examples, and design rationale in maintained Markdown.

## Safety evidence

Any unsafe operation requires its applicable invariant to be understandable at the operation and its callers. `pre_exec` runs in a post-fork child context and must use only operations permitted there; the PTY helper's `setpgid` setup and parent-side race handling require review. Nextest process isolation does not prove that a test process has no threads or environment readers. Zero-initializing `termios` before `tcgetattr` does not prove libc initializes padding, nor by itself justify reading every byte. Do not claim these concerns are resolved without a specific review and evidence.

The top-level [SAFETY.md](../SAFETY.md) preserves source-pinned rationales for particular unsafe operations. It is historical context, not a current audit; inspect each present operation and its callers before relying on a rationale.

## Documentation compatibility

On main, visibility source validation does not require a checked-in prose snapshot: use `python3 tools/visibility/status.py --validate` and `python3 tools/visibility/queue.py validate`. Generated reports are operational artifacts under ignored `target/visibility/STATUS.md` and `target/visibility/WORK_QUEUE.md`. Explicit generation (`status.py --write` or `queue.py repair-render`) precedes output checking (`status.py --check` or `queue.py check`). Output checking still rejects missing, stale, or tampered reports. An explicit output override is confined to the visibility artifact directory; it cannot replace maintained documentation or authoritative inputs. These local CLI workflows do not recreate tracked status or work-queue documentation.

The repository publisher retains a separate historical `STATUS.md` cursor/wire contract. Connecting that writer to this documentation system requires an intentional protocol migration; local artifact generation does not change or certify that public publishing contract.

Main's capture and deferred-evidence packaging records retired documentation pins explicitly while continuing to require every operational payload. Historical source, archive, receipt, measurement, and binary identities remain historical; changing packaging is not fresh qualification. Integrity tests reject operational deletion, corruption, unauthorized retirement, and invalid provenance. Disposable integration fixtures generate their report views from authoritative inputs rather than copying deleted documentation.

The current [`tools/qualified-capture/pins.json`](https://github.com/tailrocks/terminal-components-claude/blob/c8a31dd3fa6c65cf926228ca660bc7785dacb295/tools/qualified-capture/pins.json#L1-L2) uses schema 2 and [records a retired documentation digest](https://github.com/tailrocks/terminal-components-claude/blob/c8a31dd3fa6c65cf926228ca660bc7785dacb295/tools/qualified-capture/pins.json#L83-L90) for its old README. The [README at immutable source revision `b0edda802`](https://github.com/tailrocks/terminal-components-claude/blob/b0edda802880473f5541dc71d18d5cfe5ecd991c/tools/qualified-capture/README.md) is retained as historical provenance only; its commands and pinned tool versions are not current procedure, fresh tool qualification, capture authority, or proof about today's frozen-CLI process.

CI ownership is also source-defined: [`.github/AGENTS.md`](../.github/AGENTS.md) marks generated workflow paths and directs maintainers to change generator inputs such as [`.velnor/config.toml`](../.velnor/config.toml), rather than hand-editing managed workflow output. This records the repository's existing generator boundary; it does not authorize a generator run or a workflow policy change.

Main's minimal-comment migration changes three documentation-specific lint settings: `missing_docs`, `clippy::missing_errors_doc`, and `clippy::missing_panics_doc` are allowed. Safety lints and the remaining lint policy stay unchanged. Formatting and behavior-preserving style fixes must still pass the workspace gates. This policy does not authorize restoring redundant prose to conceal a lint mismatch.

The design reference and WIP workspaces can still contain older generated-component documentation, task/hash/bootstrap validators, and report consumers expecting retired Markdown or historical text. Inspect the checked-out implementation before invoking those workflows. Their remaining protocol migrations must preserve operational evidence and rejection behavior; do not regenerate measured hashes or restore legacy guidance merely to make a gate green. A passing main gate does not certify another branch's tooling.
