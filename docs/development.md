# Development

## Workspace commands

Install the checked-out workspace's pinned developer tools with `mise install`, then use its declared Rust toolchain. Before a gate, inspect `AGENTS.md`, `Cargo.toml`, `rust-toolchain.toml`, and CI configuration because package layout and profiles differ. T37 retires the legacy suite; T38 must approve any replacement tests before test edits or execution.

| Workspace | Location | Non-test command policy |
|---|---|---|
| `main` | Workspace packages, with default members for facade and applications | Use the pinned toolchain and guarded commands for `cargo check --workspace --all-targets`, `cargo fmt --all -- --check`, and `cargo clippy --workspace --all-targets -- -D warnings`. |
| `design/interaction-polish` and historical `wip/*` branches | Earlier `junie-tui` workspace | These branches have separate source, suites, and instructions. Their historical command lists are not current procedures for this workspace. |

Format, check, and lint the affected workspace with its configured commands. T37 runs no tests. Do not run old Nextest or workspace test commands after cleanup. One GPT-6.1-Sol agent at medium effort must design the T38 test architecture. A different GPT-6.1-Sol agent at medium effort must review it. Before a later task edits or executes tests, an independent GPT-6.1-Sol reviewer at medium effort must approve that task's test plan. A compile, format, or lint result proves only its recorded command and source revision; it does not prove product behavior.

At source input `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`, the facade's `testing` feature gated the [`ex01_button`, `ex02_text_input`, and consumer targets](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock/Cargo.toml#L8-L75). T37 retires those targets. The pinned manifest is historical; T05 will classify the remaining feature surface.

CLI option discovery is application-specific. The [Showcase parser](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/showcase-ui/src/app.rs#L2889-L2981) parses `--page`, `--theme`, `--color`, `--motion`, and `--frame`; that parser has no `--help` branch. Holla, Jackin Preview, and TablePro each have explicit help outcomes in their [parser](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/holla-ui/src/cli.rs#L100-L108), [binary entry](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/holla/src/main.rs#L16-L21), [parser](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/jackin-preview-app/src/cli.rs#L99-L106), [binary entry](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/jackin-preview/src/main.rs#L5-L21), [parser](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/tablepro-ui/src/cli.rs#L143-L151), [runner](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/tablepro-ui/src/cli.rs#L44-L55), and [binary entry](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/tablepro/src/main.rs#L1-L13). These links describe the frozen parser snapshot, not current launch results. T37 preserves explicit CLI help metadata and production diagnostics; current source and help must be checked again after cleanup.

## Backend-free dependency checks

A backend-free consumer claim needs an isolated consumer's resolved dependency graph without workspace feature unification; an in-workspace compile or an optional dependency declaration is not enough. The current `termrock-core` manifest enables its optional Crossterm adapter by default ([feature declaration](https://github.com/tailrocks/terminal-components-claude/blob/f4611b1fe98a98b78c9e2c32a6c9522fe72fd863/crates/termrock-core/Cargo.toml#L8-L16)). Bind the check to the current owning gate before publishing an invocation; historical commands and tool pins are not current instructions.

## Retired capture and frozen CLI history

At frozen product input `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`, the former [`tools/frozen_baseline.py`](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/tools/frozen_baseline.py) had source SHA-256 `fab5c2bbd40858d3fd13eb421411e7af065406edf46d8276d8d88918270f365e`. Sol high decision T37-FROZEN-01 retires it as a legacy PTY Nextest runner. T37 also retires `tools/test_frozen_baseline.py`, the paired-runner patch and extensions, and the exclusive frame helper.

Sol high decision T37-CAPTURE-01 retires `tools/capture.sh`, `tools/capture_exec.sh`, `tools/baseline_capture.sh`, `tools/capture_provenance.py`, `tools/parity_replay.py`, and `tools/parity_approve.py` as the legacy 499-recipe execution and approval closure. The review found no established production diagnostic owner or caller. These files and their commands are not current capture, replay, or approval procedures. T37 does not provide a replacement runner or fresh reference approval. T04 may propose a replacement only after T38 review.

Do not run the old prepare, build, candidate-build, compare, live-compare, parity replay, capture, approval, or runner-guard procedures. Their commands, recipes, scenario counts, reports, and results are source-pinned historical evidence only. Keep baseline and parity data, recipes, replays, manifests, receipts, reports, hashes, and review records unchanged. Keep annotated, unsigned `visual-baseline` tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542` and peeled commit `4169783ae883788186d161af09ba9c481bd84e5c`, with its oracle, snapshots, receipts, and reference decisions, immutable. This page does not define a replacement capture gate. Product behavior and visual parity remain `NOT VERIFIED`.

## Consumer slots

`Ui::layer_slot(id, |slot| ...)` draws into an already-open layer's resolved rectangle and focus scope. The consumer-facing `PickerSlotExt::picker` adapter delegates to `Picker::draw` once; applications provide their picker, state, and domain items without receiving the rectangle. Open the layer with `Picker::layer(cx, items)`, which measures from the current viewport and design tokens, then keep calling `Picker::update` so resize updates the same layer size. Measurement remains in the existing Picker contract; the slot does not run input handlers or application logic. Component authors needing a raw renderer callback can import `LayerSlotAuthor` from `termrock::author`.

## Rust comments

Remove nonfunctional source comments, Rust doc comments, and docstrings. Move required safety rationale, API rules, caller obligations, examples, and design rationale to maintained Markdown. Do not replace removed prose with `#[doc = "..."]`. Preserve legal and SPDX notices, parser and linter directives, shell shebangs, generated-required bytes, and structural compiler attributes such as `#[doc(hidden)]`. Keep user-visible command help as explicit runtime metadata. Use Clap `#[command]` or `#[arg]` attributes, or an explicit parser description string. Do not use Rust or Python docstrings as command help. Keep frozen Rust fixtures whose exact bytes are validation inputs unchanged.

## Workspace lint rationale

The root `Cargo.toml` at work input `c817b99e9e43abbbf3519d56b2f5ee86924243d8` has SHA-256 `d2a15fe80af76877bf5307d7a8beb3f882c5823c46af14ef437a729c2ca92f98`. Its line references below are source-pinned. T37 keeps the lint levels and lint set unchanged.

| Source lines | Maintained rationale |
|---:|---|
| 139–140 | The workspace denies `unsafe_code`. The source comment names `crates/tui`; that is an earlier path and is historical. It does not describe a current workspace crate. |
| 147–149 | Clippy's `all` group is denied for correctness, suspicious-code, style, complexity, and performance findings. The broader `pedantic` group warns. |
| 151–161 | The separately denied lints protect stated rules: no panics during normal interaction; structural handling of ragged rows instead of index panics; no material TODO or stub; no library output to stdout; and documented unsafe operations. Other listed denies remain as written in the manifest. |
| 164–165 | `arithmetic_side_effects` warns because it provides a useful signal but is too noisy to deny. Saturating arithmetic is the stated rule; denying the lint would produce too many findings. |
| 166–167 | `missing_docs`, `missing_panics_doc`, and `missing_errors_doc` are allowed as documentation-specific lints during the minimal-comment migration. This does not permit unsafe-code or correctness-lint changes. |
| 169–175 | `must_use_candidate` is allowed because it is noisy; `Response` is marked `#[must_use]` deliberately. The cast allowances cover terminal coordinates and match the pinned Ratatui 0.1.2 policy recorded at `ratatui-core-0.1.2/Cargo.toml:179–182`. |

The full lint configuration remains the source of truth. This table preserves the intent of its removed explanatory comments. It does not authorize a lint waiver, a broader allowance, or a change to an existing level.

## Safety evidence

Any unsafe operation requires its applicable invariant to be understandable at the operation and its callers. `pre_exec` runs in a post-fork child context and must use only operations permitted there; the PTY helper's `setpgid` setup and parent-side race handling require review. Nextest process isolation does not prove that a test process has no threads or environment readers. Zero-initializing `termios` before `tcgetattr` does not prove libc initializes padding, nor by itself justify reading every byte. Do not claim these concerns are resolved without a specific review and evidence.

The top-level [SAFETY.md](../SAFETY.md) preserves source-pinned rationales for particular unsafe operations. It is historical context, not a current audit; inspect each present operation and its callers before relying on a rationale.

## Documentation compatibility

On main, visibility source validation does not require a checked-in prose snapshot: use `python3 tools/visibility/status.py --validate` and `python3 tools/visibility/queue.py validate`. Generated reports are operational artifacts under ignored `target/visibility/STATUS.md` and `target/visibility/WORK_QUEUE.md`. Explicit generation (`status.py --write` or `queue.py repair-render`) precedes output checking (`status.py --check` or `queue.py check`). Output checking still rejects missing, stale, or tampered reports. An explicit output override is confined to the visibility artifact directory; it cannot replace maintained documentation or authoritative inputs. These local CLI workflows do not recreate tracked status or work-queue documentation.

The repository publisher retains a separate historical `STATUS.md` cursor/wire contract. Connecting that writer to this documentation system requires an intentional protocol migration; local artifact generation does not change or certify that public publishing contract.

Keep the historical source, archive, receipt, measurement, and binary identities. T37-FROZEN-01 retires the mixed frozen runner. T37-QUALIFIED-01 retires the `tools/qualified-capture` acquisition and qualification harness. T37 also retires the legacy deferred runner and its test suite. Claims from their integrity tests are historical; T37 does not verify packaging behavior. Preserve operational inputs and diagnostics under their named owners. A future gate needs current source evidence and an approved T38 plan.

The [`tools/qualified-capture/pins.json`](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/tools/qualified-capture/pins.json#L1-L2) at the frozen product input records a schema-2 pin set and [a retired documentation digest](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/tools/qualified-capture/pins.json#L83-L90). Keep the pins and their proof outputs, receipts, source hashes, licenses, and repair provenance unchanged. The acquisition and qualification harness is retired; do not repin it or treat these records as current qualification. The [README at immutable source revision `b0edda802`](https://github.com/tailrocks/terminal-components-claude/blob/b0edda802880473f5541dc71d18d5cfe5ecd991c/tools/qualified-capture/README.md) remains historical provenance, not a current procedure or capture authority.

CI ownership is also source-defined: [`.github/AGENTS.md`](../.github/AGENTS.md) marks generated workflow paths and directs maintainers to change generator inputs such as [`.velnor/config.toml`](../.velnor/config.toml), rather than hand-editing managed workflow output. This records the repository's existing generator boundary; it does not authorize a generator run or a workflow policy change.

Main's minimal-comment migration changes three documentation-specific lint settings: `missing_docs`, `clippy::missing_errors_doc`, and `clippy::missing_panics_doc` are allowed. Safety lints and the remaining lint policy stay unchanged. Formatting and behavior-preserving style fixes must still pass the workspace gates. This policy does not authorize restoring redundant prose to conceal a lint mismatch.

The design reference and WIP workspaces can still contain older generated-component documentation, task/hash/bootstrap validators, and report consumers expecting retired Markdown or historical text. Inspect the checked-out implementation before invoking those workflows. Their remaining protocol migrations must preserve operational evidence and rejection behavior; do not regenerate measured hashes or restore legacy guidance merely to make a gate green. A passing main gate does not certify another branch's tooling.
