/goal
# Repair the Termrock specifications and agent rules

## 1. Mission

Work in `tailrocks/terminal-components-claude`.
Use only the existing `termrock-implementation` branch.

Repair the documentation that governs the remaining refactor.
Define reusable Termrock components as the only way to build preview interfaces.
Require independent review for each work item and each commit.
Make the visual, interaction, API, and ownership requirements explicit.

This task changes documentation only.
Do not fix, revert, rename, or refactor source code in this task.
Do not regenerate or approve snapshots.
Record code defects as pending implementation work.

Use subagents for independent research, document writing, and review.
Do not specify or change subagent models in the new documents.

Read the supplied `termrock-doc-guardrails/` pack as the documentation input.
Use `FINDINGS.md`, the `proposal/` contracts, the recipe catalog, and the document checklist.
Merge these requirements into the existing canonical documents.
Do not install the proposals as a second competing documentation system.

## 2. Fixed authorities

The repository and implementation branch stay unchanged.
Do not create a new repository or another implementation branch.
Do not merge `main` or reset the branch.
Do not move the `visual-baseline` tag or release.

Use these references for different purposes:

| Reference | Purpose |
| --- | --- |
| Frozen `visual-baseline` tag | Visual and observable interaction authority. |
| `visual-baseline` branch | Compare the branch requested by the user with the frozen tag. |
| `termrock-refactor` | Recover the intended architecture and preparation requirements. |
| `termrock-implementation` | Inspect current APIs, implementation defects, and documentation gaps. |
| `main`, when needed | Structural reference only. Never a visual authority. |

The frozen tag commit is `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`.

The review for this prompt inspected these branch tips:

```text
visual-baseline:         209c88e7494cad301771bed55916316d4e890ec7
termrock-refactor:       15c1a0a73911584cf2339eaad2c0212b2661fbd8
termrock-implementation: 4d117b48092c5210c80070ac9bbeb6560973b2b0
```

Fetch the actual refs before work.
Record the exact commits used for research.
Do not discard later work to match these observations.

The inspected branch and tag have the same `src` tree.
Check manifests, dependencies, fixtures, and visual inputs too.
Do not infer complete execution equivalence from one tree hash.

## 3. Write scope

First make a path allowlist from the existing repository.
Permit only the following document classes:

```text
AGENTS.md
GOAL.md
README.md
SPECIFICATION.md
CRATES.md
CHECKLIST.md
checklist.json
crate-map.json
component-ownership.json
docs/**
refactoring-tasks/** documentation and non-executable planning records
existing or necessary scoped AGENTS.md and README.md files
.github/pull_request_template.md
CLAUDE.md symlinks adjacent to approved AGENTS.md files
```

Treat the list as document classes, not permission to change embedded execution assets.
Keep executable scripts, Rust files, baseline files, and test fixtures read-only.
Do not change task verifier commands to conceal a failure.
Do not change workflow YAML, Cargo manifests, Cargo.lock, Mise configuration, or Git settings.
Do not install hooks or change branch protection.
Do not change approved snapshots or required executable capture data.

Use Markdown fences for proposed Rust examples.
Do not add implementation `.rs` files to the repository.
Use temporary external harnesses only when you can test examples without changing the repository.
Report compilation failures. Do not fix source code to make a document example pass.

Preserve unrelated dirty files.
Compare this task's diff with its start commit, not with the visual tag.
Earlier implementation changes are not violations of this documentation-only task.

## 4. Research every reported deviation

Inspect each commit against its actual parent:

```text
9f3e04fe667aa704a03e7b70d6a1f4bae579165a
fbca27983a4f6ef980a2e24ec068ab57c33d6177
d6637639107d2046247d0a537946520960a6c719
087a1af433c0e99a32fd537d19d8418ebaac9c2a
34b28f6ebe63a0c968367d7bd822a6245c97647e
```

Read all changed files, not only the commit title.
If GitHub omits a patch, compare the parent and child blobs.
Trace the changed path to current callers.
Separate a historical defect from a defect that remains active.

Find related patterns in all four previews and the library.
Inspect direct paint calls, historical frame modules, fixture switches, and layer bypasses.
Inspect generic wrappers that only hide an application-specific painter.
Inspect generated files as code, not as automatic exceptions.

For each confirmed finding, record:

```text
finding ID
source commit, file, and symbol
observed mechanism
affected component and preview
violated requirement
required documentation change
future implementation owner
a test that must reject this mechanism
```

Do not infer intent from the code.
Do not revert these commits during this task.

## 5. Reconcile the document set

Read all current root specifications and every document under `docs/`.
Read the task catalog, capture plans, crate map, and both checklist views.
Read all scoped agent instructions.
Compare the preparation plan with the current result.

Retain correct requirements from the earlier specification packs.
Remove obsolete instructions and duplicate authorities.
Keep Git history as historical evidence.
Do not keep a second live specification because deletion feels risky.

The current workspace has the planned multi-crate structure.
Do not describe that structure as absent.
Do not certify component adoption from crate counts.
Resolve the old one-library wording without starting another crate redesign.

Use one owner for each contract:

| Contract | Canonical owner |
| --- | --- |
| Agent entry rules | Root `AGENTS.md`. |
| Current documentation task | Root `GOAL.md`. |
| Component-only composition | `docs/architecture/component-composition.md`. |
| Public signatures and ownership | Existing `docs/api/public-api.md` and `types.md`. |
| Consumer examples | `docs/api/consumer-recipes.md` and its example catalog. |
| Per-work and per-commit review | `docs/process/per-commit-review.md`. |
| Visual and ownership verification | `docs/verification/ownership-and-parity.md`. |
| Individual component contracts | Existing `docs/components/` files. |
| Current implementation status | Existing authoritative `checklist.json`. |

Make root summary documents link to these owners.
Do not duplicate full rules in several files.

## 6. Define the central contract

Write this rule prominently:

> A preview screen is a composition of reusable Termrock components.
> The component owns its pixels, geometry, interaction, and state rules.
> The preview supplies data, layout choices, component state, and typed application actions.

Apply this rule to Showcase, Jackin Preview, Holla, and TablePro.
They are preview UI applications in this repository.
They show how a real interface uses the library.
They are not separate product redevelopment projects.

Keep their existing fixture worlds and interaction flows.
Do not simplify a flow because the preview uses mock data.
A mock operation must still exercise the same component and action path.

Require all of these outcomes together:

```text
visual parity
AND observable interaction parity
AND reusable component ownership
AND usable public APIs
AND customization through supported contracts
AND valid review evidence
```

A matching screenshot cannot override a failed ownership check.
An import from `termrock` cannot prove reusable composition.
A large crate inventory cannot prove that the code has correct boundaries.

## 7. Define permitted and prohibited code patterns

Preview code may construct components and supply fixture data.
Preview code may choose routes and documented responsive layouts.
Preview code may handle typed application actions.
Preview code may map domain records to borrowed component models.
Preview code must not recreate component rendering or generic interaction.

Low-level cell painting belongs inside reusable component implementations.
Permit it only in the documented component-author boundary.
Do not ban painting inside the actual renderer that must draw the component.
Do not expose that permission as a preview escape hatch.

Explicitly prohibit these patterns in preview drawing paths:

```text
historical full-screen painters
snapshot files or exported frame data used as application output
hardcoded answer tables for 72x20, 80x24, 100x30, 120x40, or 160x50
fixture/scenario IDs that select a different painter
paused mode that bypasses component updates or layers
raw buffer access through termrock::author or an alias
direct ui.paint_str, ui.fill, set_string, or equivalent rendering of controls
padded strings that simulate columns, selection, fields, menus, or buttons
manual borders, focus gutters, scrollbars, or widget cursors
normal component drawing followed by a historical overlay
app-specific screens moved into a termrock crate under a generic name
fake state flags that display success without the normal component action
```

Distinguish legitimate layout constants from test-size answer selection.
Distinguish fixture text from pre-rendered screen text.
Distinguish terminal content from a stored picture of the enclosing application.
Do not use a `Canvas`, `TerminalView`, slot, or helper to bypass these distinctions.

If a required primitive is missing, specify the smallest reusable primitive.
Give it an owner, public data contract, use case, and test requirement.
Do not implement it now.

## 8. Write usable API examples

Supply examples for the cases listed in the attached recipe catalog.
At minimum, cover controls, fields, validation, lists, trees, tabs, dialogs, menus, pickers, grids, output, motion, and terminal panes.
Include the four preview compositions and theme customization.

For each example, show its domain data and persistent component state.
Show update handling and typed results.
Show drawing through reusable components.
Show layout and customization through public APIs.
Show the failure or cancellation path when applicable.

Use ordinary Rust functions and existing component concepts.
Do not invent a new UI language or mix update and drawing in a `show()` shortcut.
Do not reduce the code count by hiding direct paint calls in a helper.

Classify every example:

```text
current_source_checked
current_compiled
proposed_target
forbidden_example
```

Record the source SHA and declarations for a current example.
Record the exact compile command and result before using `current_compiled`.
For a proposed example, identify every new helper or signature.
Mark its implementation and compilation as pending.
Do not present the old proposed TextInput constructor as an existing API.

Each component document must use the component contract template.
Preserve all 45 component/API surfaces and shared foundations.
Add missing generic primitives only with source-backed need.
Do not replace detailed contracts with this summary.

## 9. Define customization without loss of ownership

Keep exact baseline appearance as the default recipe.
Allow other themes and parts through a separate extension contract.
Do not use a custom theme to excuse default-theme drift.

Specify instance, subtree, variant, and part customization.
Specify stable content slots and row/cell content descriptions.
State which component owns clipping, hit regions, focus, capture, and disabled behavior.
A slot does not become another input router or screen painter.

Document simple use first.
Put advanced component-author APIs in a separate section.
A normal preview must not construct runtime registries or publish fake geometry.

## 10. Require independent review for every work item and commit

Apply this procedure to this documentation task too.
Assign a reviewer who did not author the submitted change.

Before work, review the plan, owner, scope, examples, and required evidence.
Before each commit, freeze the staged change.
Record its parent commit, candidate tree, and diff digest.
Have the reviewer inspect every changed line and relevant callers.
Include generated source and data provenance in that review.

The reviewer must check architecture before accepting a pixel result.
Reject special-case painters, weak evidence, and changed requirements that hide a failure.
Review fixes again before approval.
Any staged change after review invalidates that review.

Create the commit only after approval.
Confirm that its parent and tree match the reviewed subject.
Bind the review receipt to the actual commit SHA outside that commit.
Do not create a self-referential receipt that claims to include its own commit hash.

Review the complete work item again after integration.
A pull request approval is not a substitute for per-commit review.
A test function named `review` is not an independent review.
Do not fabricate reviewers or approval records.

Keep commits small and frequent.
Request review as soon as a coherent change is ready.
Do not bypass review to meet a commit-frequency goal.
Do not wait for the whole document rewrite before committing.

Preserve the exact sign-off identity:
`Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>`.
Preserve `CLAUDE.md` symlinks to adjacent `AGENTS.md` files.

## 11. Specify proof that rejects these failures

Define separate gates for:

1. Source and baseline identity.
2. Public API compilation and ordinary consumer use.
3. Real component ownership.
4. Exact cells, cursor, and pixels.
5. Input, state transitions, and typed actions.
6. General behavior beyond captured fixture sizes.
7. Artifact integrity and admission.
8. Independent work and commit review.

Use live reference and candidate execution for behavior proof.
Keep ANSI replay in a conversion-evidence category.
Keep all six formats: ANSI, HTML, PNG, ASCII, TXT, and canonical frame JSON.
Keep the screen/substep hierarchy and stable case identities.

Specify negative tests for known failure patterns.
Change a library painter and require the actual preview result to change.
Suppress an action and require the real journey to fail.
Change fixture labels and item order to expose stored answers.
Test neighboring widths and nonzero origins to expose fixed coordinates.
Test paused input to prove that layers and controls still operate.

Require actual case and checkpoint sets, not only counts.
Do not equate a store-integrity run with live application parity.
Do not treat a script exit code as evidence for unexecuted cases.
Keep unknown, failed, skipped, and not-run states distinct.

Describe future CI, hook, and static-analysis requirements without installing them now.
Record existing enforcement only when source and execution evidence prove it exists.
Documentation cannot create a technical barrier by itself.

## 12. Repair checklists and stale claims

Keep existing requirement IDs and accepted crate ownership where still valid.
Do not delete an obligation because its implementation failed.
Separate documentation acceptance from implementation acceptance.

Add the documentation repair rows from this pack.
Link each new rule to its source finding, example, and future verifier.
Keep future code and enforcement tasks pending.

Reopen a checked row when its evidence is missing, stale, or unrelated.
Record the reason and preserve its prior receipt.
Do not reset all verified work without review.
Do not mark architecture complete merely because a snapshot matches.

Give every known historical painter and bypass a future removal task.
Give every missing reusable API a component-owned task.
Give every unimplemented guard a test/tooling task.
This task writes those contracts. It does not execute those repairs.

## 13. Subagent work plan

Use separate workstreams for the five-commit audit and the three-branch comparison.
Assign component/API writers by shared mechanism.
Assign one reviewer to each preview composition.
Assign separate owners for governance, verification, and checklist reconciliation.

The final reviewer must not author the integrated specification.
Ask the reviewer to find contradictions and unsupported completion claims.
Require an explicit pass on this documentation task's acceptance criteria.
Do not ask the reviewer to approve known-unfixed implementation code.

Use an available ASD-STE100 skill or authorized standard as a writing reference.
Use short active sentences and one instruction per sentence.
Keep technical names unchanged.
Use the project glossary consistently.
Do not claim formal compliance without the required language checks.

## 14. Documentation acceptance and stop line

Accept this task only when all documentation checklist rows have current evidence.
Verify the complete task diff against the document allowlist.
Verify that source, tests, build inputs, workflows, and baseline bytes are unchanged.
Check local links, JSON records, duplicate rule IDs, and snippet statuses.

Confirm that every reported commit has a finding or a justified disposition.
Confirm that every component has one canonical contract owner.
Confirm that all four previews use the same documented composition rules.
Confirm that work review and every-commit review have an exact-subject procedure.

Commit and push the reviewed document changes on `termrock-implementation`.
Do not merge the pull request.
Do not begin code repairs after the documentation gate passes.

Report the start and final commits, changed documents, and source identities.
Report reviewer receipts for each new commit.
Report document validation results and example compilation status.
List the remaining code and enforcement tasks as pending.
Do not call the Termrock implementation complete.
