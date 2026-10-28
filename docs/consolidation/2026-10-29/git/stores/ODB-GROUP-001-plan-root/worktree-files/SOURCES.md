# Sources and technical terms

## 1. Review limits

This package separates observed source from proposed work.
A source finding does not prove a runtime failure by itself.
A remote CI result applies only to its recorded commit.

The review used source copies of both branches.
It inspected manifests, API owners, component families, application paths, tests, CI results, and selected images.
It did not inspect every source line or every image.
T01 must complete the full requirement and public API inventory.

No local Rust compiler or Cargo executable was available.
No local Rust build, Nextest run, Clippy run, benchmark, package test, or live terminal capture ran.
A Python test did run and failed before it selected any test.
See [VALIDATION.md](VALIDATION.md) for the exact evidence.

Never use any Astra model under any circumstances or in any scenario.
The source-review scopes were crate and API design, UI and reference parity, and performance and validation.

The model names in [AGENT-RUNBOOK.md](AGENT-RUNBOOK.md) apply to future implementation.
They are instructions, not evidence that those future tasks ran.

The adopted testing and comment policy is in [TESTING-RULES.md](TESTING-RULES.md).
Its source review and limits are in [PRACTICE-REVIEW.md](PRACTICE-REVIEW.md).

## 2. Fixed source records

| Record | Exact value or link |
| --- | --- |
| Repository | [tailrocks/terminal-components-claude](https://github.com/tailrocks/terminal-components-claude) |
| Target pull request | [PR #38](https://github.com/tailrocks/terminal-components-claude/pull/38) |
| Target branch | `parity/termrock-visual-baseline` |
| Target commit | [e08ac57d10ae54c87337eae0be08b1f5e8f4bda4](https://github.com/tailrocks/terminal-components-claude/commit/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4) |
| Reference pull request | [PR #26](https://github.com/tailrocks/terminal-components-claude/pull/26) |
| Reference branch | `design/interaction-polish` |
| Reference commit | [5b3dbd7ae799f325efbb4f4cc11b2f5518dec948](https://github.com/tailrocks/terminal-components-claude/commit/5b3dbd7ae799f325efbb4f4cc11b2f5518dec948) |
| Common ancestor | [cc14dd6beae526884aabdf897e309be837b4f504](https://github.com/tailrocks/terminal-components-claude/commit/cc14dd6beae526884aabdf897e309be837b4f504) |
| Historical visual tag | `visual-baseline`; annotated and unsigned |
| Historical tag object | `2718a6b8127a522a3d351d0b4bdb7542fc8da542` |
| Historical tag commit | [4169783ae883788186d161af09ba9c481bd84e5c](https://github.com/tailrocks/terminal-components-claude/commit/4169783ae883788186d161af09ba9c481bd84e5c) |
| tuiscotti pin | `a1ea27f757d75d07b05499276cd932e9f8e067ad` |
| Target CI run | [40222193598](https://github.com/tailrocks/terminal-components-claude/actions/runs/40222193598) |

Both pull requests were open drafts when their metadata was retrieved.
Do not treat that status as current for a later run.
T00 refreshes both heads and their status.

The target inventory contains 49 workspace members.
The production library policy names 20 crates.
Two additional crate manifests are outside workspace membership: `termrock-e2e` and `termrock-visibility-tests`.
Do not count all manifests as active workspace members.

## 3. Repository instructions and contracts

Use the current version of these documents during implementation.
The links below fix the review version.

| Subject | Source |
| --- | --- |
| Root agent rules | [AGENTS.md](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/AGENTS.md) |
| Project instructions | [CLAUDE.md](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/CLAUDE.md) |
| Workspace members, dependencies, and lints | [Cargo.toml](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/Cargo.toml) |
| Current architecture | [docs/architecture.md](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/docs/architecture.md) |
| Product rules | [docs/product.md](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/docs/product.md) |
| Component contracts | [docs/components-contracts.md](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/docs/components-contracts.md) |
| Retained decisions | [docs/decisions.md](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/docs/decisions.md) |
| Test, comment, and capture rules | [docs/development.md](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/docs/development.md) |
| CI ownership | [.github/AGENTS.md](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/.github/AGENTS.md) |
| CI generator input | [.velnor/config.toml](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/.velnor/config.toml) |
| Tool pins | [mise.toml](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/mise.toml) |
| Production package policy | [tools/acceptance_policy.json](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/tools/acceptance_policy.json) |
| Oracle state | [baselines/oracle/ORACLE.toml](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/baselines/oracle/ORACLE.toml) |

Each finding in [REVIEW.md](REVIEW.md) links its source.
Each component family in [COMPONENT-MATRIX.md](COMPONENT-MATRIX.md) has a required behavior record.
These records define implementation scope.
They do not replace the source checks in T00 and T01.

## 4. Image sample record

Four corresponding PNG pairs were retrieved.
The four distinct reference images were viewed.
Each target file had the same SHA-256 hash as its reference file.

The paths are relative to each fixed source commit.

| Sample | Path | SHA-256 |
| --- | --- | --- |
| 1 | `snapshots/showcase/audit/inputs/120x40/truecolor.png` | `baa5efb092af797cf66b824b5089edda7fa3c05ff93bdd4c3d92a1f6943db6d3` |
| 2 | `snapshots/showcase/flows/dialogs/open/120x40/truecolor.png` | `05f0da613e442a51c84f9c1e0fddfed6227d1ffda0c7562aad335f1ef6f2d47e` |
| 3 | `snapshots/showcase/flows/inspector/open/72x20/nocolor.png` | `8e1bf49144b6e344c77f29d97f1a08e75ffe3522863993aa87aade1f27fafa3d` |
| 4 | `snapshots/showcase/flows/inputs/selected/120x40/truecolor.png` | `78b6a13fa9eca9ad511a58970c5cf3a85712be5ab6e7cfbccedc9d45af7d2871` |

This sample supports the appearance review.
It does not prove that either current binary produces those images.
It does not prove keyboard, pointer, focus, copy, or validation behavior.
Use T04 and the final visual gates for that evidence.

## 5. Primary external references

| Source | Use in this plan |
| --- | --- |
| [ASD-STE100 overview](https://www.asd-ste100.org/about_STE.html) | Controlled technical language and consistent instructions. |
| [ASD-STE100 FAQ](https://www.asd-ste100.org/STE_faq.html) | Short sentences, active voice, and defined technical terms. |
| [ASD-STE100 Issue 9](https://www.asd-ste100.org/assets/files/ASD-STE100_ISSUE9.pdf) | The standard reference. Direct PDF access returned 403 during this review. |
| [Rust API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html) | Clear naming, predictable types, errors, and public API contracts. |
| [Rust API naming guidance](https://rust-lang.github.io/api-guidelines/naming.html) | Consistent constructor, conversion, and type names. |
| [Rust API interoperability guidance](https://rust-lang.github.io/api-guidelines/interoperability.html) | Standard traits and useful caller integration. |
| [Cargo feature reference](https://doc.rust-lang.org/cargo/reference/features.html) | Optional dependencies and explicit feature forwarding. |
| [Cargo resolver reference](https://doc.rust-lang.org/cargo/reference/resolver.html#features) | Separate feature builds and dependency graph checks. |
| [Cargo workspace lint rules](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table) | Explicit member lint inheritance. |
| [Cargo published path dependency rules](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#local-paths-in-published-crates) | Version requirements and package checks. |
| [Ratatui widget concepts](https://ratatui.rs/concepts/widgets/) | Existing Rust widget and state patterns. |
| [shadcn/ui documentation](https://ui.shadcn.com/docs) | Source ownership, composition, useful defaults, and direct customization. |

The plan uses these principles with the repository's actual contracts.
It does not import another UI framework's full architecture.
No external source was used to claim a measured Termrock performance result.

## 6. Writing rules

Use ASD-STE100 Simplified Technical English for new prose.
Use the active voice.
Use one instruction in each procedural sentence.
Use no more than 20 words in a procedural sentence.
Use no more than 25 words in a descriptive sentence.

Use a consistent technical term for each object.
Define a new technical term before use when its meaning is not clear.
Keep code identifiers, paths, commands, errors, and model IDs exact.
Do not change an identifier to simplify the surrounding English.

Use a noun group only when its meaning is clear.
Split long lists into parallel items.
State the condition before the required action.
State the expected result beside its test.

Sentence length is one review aid.
It does not certify full dictionary conformity.
The review did not obtain a complete machine-readable ASD-STE100 dictionary.

## 7. Project technical nouns

These terms have the meanings below in this package.
They are project terms for software work.
They are not claims about entries in the ASD dictionary.

| Term | Meaning in this package |
| --- | --- |
| acceptance gate | A required check that must pass before a task or goal can close. |
| action | A typed result that requests application work. |
| adapter | Code that maps an existing contract to another caller interface. |
| API | The public types and operations that a caller can use. |
| artifact | A file that records output or evidence. |
| backend | Code that connects the terminal runtime to platform input and output. |
| baseline | A fixed result used for comparison. |
| binding | A mapping from input to a named action. |
| block | A reusable composition of multiple components. |
| branch | A named Git line of development. |
| builder | A method that sets optional component configuration. |
| cache | Derived data retained to avoid repeated work. |
| callback | Caller-supplied code that a component invokes under a stated contract. |
| capability | A supported operation or terminal behavior. |
| cell | One position in a terminal output grid. |
| checkpoint | A named point in a deterministic input and output sequence. |
| chord | A key with its modifier set. |
| CI | Automated continuous integration checks. |
| clipping | Restriction of output and hit regions to a permitted area. |
| commit | A Git source revision, or a validated data change when the context states that meaning. |
| component | A reusable UI element with a defined state, input, and output contract. |
| composition | The arrangement of components through the shared layout and runtime APIs. |
| conformance | Evidence that an implementation follows its required contract. |
| consumer | An application or test program that uses a public API. |
| crate | A Rust compilation and dependency unit. |
| cursor | A text position, collection navigation position, or terminal cursor, as explicitly named. |
| data revision | A value that changes when the source data contract changes. |
| descriptor | A value that describes content or behavior for another component. |
| domain | Application data and decisions that do not depend on UI mechanics. |
| facade | The public `termrock` crate that exposes the supported library families. |
| feature | A named Cargo option that changes compiled dependencies or APIs. |
| fixture | Fixed input or support code for a test. |
| focus | The current owner of applicable keyboard input. |
| frame | Output and registration data prepared for one presentation. |
| generation | An identity value for a specific state, opening, or prepared output. |
| grapheme | A user-perceived character that can contain multiple Unicode code points. |
| harness | Code that drives tests and collects their results. |
| hit region | An area that maps pointer input to an owner. |
| identity | A stable key that names an object independently of its current position. |
| invalidation | A request to repeat paint or layout because earlier output is no longer current. |
| layer | A runtime-owned input and paint level, such as a modal or popup. |
| leaf | An atomic composition item under the declared validation contract. |
| lineage | Source identity used to distinguish related derived cache generations. |
| lint | A compiler or tool rule that detects a source issue. |
| manifest | A file that declares package, case, or result metadata. |
| metadata | Data that identifies or describes another object. |
| migration | A documented move from an old API or owner to its replacement. |
| modal | A layer that restricts input to its permitted contents. |
| MSRV | The minimum supported Rust version. |
| normal dependency | A production Cargo dependency, as distinct from a build or dev dependency. |
| oracle | The accepted reference data and rules for a comparison. |
| owner | The crate, runtime object, task, or agent responsible for a stated behavior or file. |
| package | The distributed Cargo archive and its declared metadata. |
| parity | Equality with the accepted reference for the named behavior or output. |
| patch | A source diff, or a partial style change when the context states that meaning. |
| placeholder | A route with substitute content that does not implement its required demonstration. |
| projection | Output derived from current source state under a defined view. |
| protocol | The ordered operations and values that two owners must follow. |
| PTY | A pseudo-terminal used to drive and observe terminal behavior. |
| receipt | Evidence that a specific prepared frame was presented. |
| reconciliation | The update of cached indices and fallback state after data changes. |
| registry | A declared set of routes or packages, as explicitly named. |
| replay | A repeated execution of a recorded input sequence. |
| response | The typed action, input flow, invalidation, and owner state returned by a component. |
| route | A stable application page identity and its navigation path. |
| runtime | The shared owner of frame state, input routing, focus, layers, and explicit time. |
| schema | The declared shape and identity of data, such as grid columns. |
| semantic state | User or application meaning, separate from derived paint and frame data. |
| session | One controlled lifetime of terminal input, output, and platform modes. |
| SHA | A hash that identifies a Git object; file hashes state their algorithm separately. |
| shell | Shared application chrome, slots, navigation, hints, and leave policy. |
| slot | A named place for caller-supplied content or style. |
| snapshot | A fixed capture of output or state. |
| source tree | The files at a stated repository revision. |
| stale result | A result whose owner, source, request, or generation is no longer current. |
| task | A bounded unit of work with predecessors, owners, tests, and acceptance. |
| test inventory | The exact selected set of required tests or cases. |
| theme | Named style roles, metrics, and capability-specific colors. |
| token | A typed identity or style value whose exact meaning is stated by its API. |
| tooltip | A small non-modal explanation associated with an owner. |
| transport | The operation that writes prepared output to the terminal. |
| TUI | A text-based user interface in a terminal. |
| viewport | The visible region of larger content. |
| widget | A component type in the existing terminal UI model. |
| worktree | A separate Git checkout that shares repository history. |

## 8. Project technical verbs

| Verb | Required meaning |
| --- | --- |
| acknowledge | Confirm presentation of the exact prepared frame. |
| borrow | Use a Rust reference under its declared lifetime. |
| compile | Translate source through the selected compiler. |
| dispatch | Send a task or event to its declared owner. |
| invalidate | Mark derived output or data as no longer current. |
| reconcile | Update state through the shared identity and fallback rules. |
| refactor | Change internal structure while preserving the required behavior. |
| remap | Change the input assigned to an action. |
| render | Produce terminal cells from current state. |
| re-export | Expose the same Rust item through another public path. |
| serialize | Convert a record to its declared byte or text format. |
| stage | Prepare package or evidence bytes before final acceptance or publication. |

Keep technical verbs limited to these software meanings.
Use a simpler ordinary verb when it expresses the same instruction clearly.

## 9. Testing and comment policy sources

The following pinned commits provide the adopted project rules.
The review did not audit every changed file or every review thread.

| Source | Use in this plan |
| --- | --- |
| [Velnor PR #391 at `29d56212bdd8dfecfc84b1ecf3c8cd565f757c4e`](https://github.com/tailrocks/velnor-new/commit/29d56212bdd8dfecfc84b1ecf3c8cd565f757c4e) | This commit records plan-first tests, observable behavior, red-green evidence, deterministic runs, and functional comment exceptions. |
| [Jackin PR #1231 at `7302c4c0d67a97c59d1439213f34a9c8ff3d22f6`](https://github.com/jackin-project/jackin/commit/7302c4c0d67a97c59d1439213f34a9c8ff3d22f6) | Its shared rules require an architect review after an escape and integration-first test plans. |
| [Google Inquiry Method](https://testing.googleblog.com/2016/06/the-inquiry-method-for-test-planning.html) | It asks teams to consider testability during design and select plan questions by risk. |
| [Google Change-Detector Tests](https://testing.googleblog.com/2015/01/testing-on-toilet-change-detector-tests.html) | It explains why a test that repeats implementation details may not prove correct behavior. |
| [Google Test Fidelity](https://testing.googleblog.com/2024/02/increase-test-fidelity-by-avoiding-mocks.html) | It recommends real dependencies when practical and explains fidelity limits of fakes and mocks. |
| [Google Coverage Best Practices](https://testing.googleblog.com/2020/08/code-coverage-best-practices.html) | It treats coverage as an indirect measure and rejects a universal quality target. |
| [Playwright Best Practices](https://playwright.dev/docs/best-practices) | It recommends user-visible behavior and isolated test state. |
| [Playwright API mocking](https://playwright.dev/docs/mock) | It documents controlled network routes for tests that need external API responses. |
| [Anthropic agent evaluations](https://www.anthropic.com/research/demystifying-evals-for-ai-agents) | It defines evaluation tasks, success criteria, graders, and outcomes. |
| [SWE-bench harness](https://www.swebench.com/SWE-bench/api/harness/) | It records fail-to-pass and pass-to-pass results as separate measures. |

These sources support selected practices.
They do not establish a universal post-AI testing consensus.
The no-owned-mocks rule, T37 cleanup, and independent review gates are Termrock policy.
