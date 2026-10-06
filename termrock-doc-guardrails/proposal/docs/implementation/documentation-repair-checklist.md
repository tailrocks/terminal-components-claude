# Documentation repair checklist

This checklist accepts the documentation task only. It does not certify the Rust implementation.

All rows start pending. Use the [JSON record](documentation-repair-checklist.json) as the status source.

Keep the existing root implementation checklist. Preserve its IDs and valid evidence. Reopen unsupported status claims with a reason.

## Authority and source analysis

- [ ] **DOC-001 — Record the actual branch tips and task start commit.** Record exact refs without changing or resetting the repository.

- [ ] **DOC-002 — Reconcile the baseline branch with the frozen tag.** Compare source, manifests, fixtures, and capture inputs. State the limits of each comparison.

- [ ] **DOC-003 — Inspect all five reported commits and their parents.** Read omitted patches through parent/child blobs. Record a finding or a justified disposition for each commit.

- [ ] **DOC-004 — Trace each reported mechanism to current consumers.** Separate a historical change from a currently reachable defect.

- [ ] **DOC-005 — Find related patterns in all four preview apps.** Audit raw painting, generated frames, fixture-specific dispatch, and normal input/layer bypasses.

- [ ] **DOC-006 — Read existing instructions and canonical planning documents.** Map root and scoped instructions, component contracts, task records, and checklist views.

## Composition and ownership rules

- [ ] **DOC-007 — Define the preview-app purpose.** Describe Showcase, Jackin Preview, Holla, and TablePro as complete UI previews built from reusable components.

- [ ] **DOC-008 — Define the conjunction of acceptance requirements.** Require pixels, interactions, ownership, public APIs, customization, and review together.

- [ ] **DOC-009 — Define the low-level author boundary.** Permit rendering inside approved reusable components. Prohibit its use as a preview escape hatch.

- [ ] **DOC-010 — Define the data-versus-screen distinction.** Permit fixture records and child terminal content. Prohibit exported full-screen answer data.

- [ ] **DOC-011 — Define normal and paused execution rules.** The motion policy cannot bypass component input, layer opening, or routing.

- [ ] **DOC-012 — Define the missing-component procedure.** Require a smallest reusable primitive contract and an implementation owner. Do not fix code now.

- [ ] **DOC-013 — Define legitimate responsive layout.** Separate constrained layout and semantic breakpoints from snapshot-size-specific painters.

- [ ] **DOC-014 — Preserve the current multi-crate responsibility model.** Resolve stale one-crate text. Do not count crates as evidence of reusable behavior.

## Component contracts and examples

- [ ] **DOC-015 — Map all 45 existing component contracts.** Every required surface retains one canonical owner and its detailed behaviors.

- [ ] **DOC-016 — Map all shared foundations and legacy dispositions.** Keep all declared shared mechanisms and historical-family obligations accounted for.

- [ ] **DOC-017 — Use one component document template.** Specify data, state, phases, responses, variants, parts, bounds, and verification requirements.

- [ ] **DOC-018 — Separate current and proposed API signatures.** Do not present the earlier proposed TextInput signature as current source.

- [ ] **DOC-019 — Write complete control and field usage examples.** Show phase separation, controlled state, typed responses, disabled and invalid cases.

- [ ] **DOC-020 — Write collection and layout examples.** Show keyed data, content descriptions, responsive regions, and state reconciliation.

- [ ] **DOC-021 — Write overlay, picker, and menu examples.** Show normal layer opening, cancellation, nesting, and target identity.

- [ ] **DOC-022 — Write data, output, and motion examples.** Show Grid capabilities, logs, code/diff, motion sampling, and prepared terminal content.

- [ ] **DOC-023 — Write all four preview composition examples.** Document each helper as component composition or a data reducer. No hidden painter is permitted.

- [ ] **DOC-024 — Write supported customization examples.** Show variants and semantic patches. Separate advanced authoring from normal consumption.

- [ ] **DOC-025 — Write negative examples from the findings.** Include historical frame dispatch, repainted controls, and disguised snapshot components.

- [ ] **DOC-026 — Record every example status and unresolved helper.** Use the example catalog. Include source or compile evidence only when actually obtained.

## Work and commit review

- [ ] **DOC-027 — Require independent plan review for each work item.** Review scope, owner, needed component contracts, and evidence before implementation work.

- [ ] **DOC-028 — Require independent review before every commit.** Apply this rule to documentation, tests, generated code, metadata, and implementation.

- [ ] **DOC-029 — Bind review to an exact subject.** Record parent commit, candidate tree, binary diff digest, and all changed paths.

- [ ] **DOC-030 — Invalidate approval after any subject change.** Do not reuse an earlier approval after edits or a changed base.

- [ ] **DOC-031 — Bind approved subjects to actual commits.** Verify committed parent/tree and store the actual SHA in an external receipt.

- [ ] **DOC-032 — Require final work-item integration review.** Per-commit reviews do not replace the complete work-item check.

- [ ] **DOC-033 — Preserve frequent coherent commits.** Request review promptly. Frequency does not permit review bypass.

## Verification contracts

- [ ] **DOC-034 — Separate file integrity from live parity.** State what a store-integrity run does and does not establish.

- [ ] **DOC-035 — Specify component-ownership proof.** Trace calls and run targeted painter/action mutations in future implementation work.

- [ ] **DOC-036 — Specify fixture-independent behavior checks.** Change labels, order, dimensions, origins, and input paths to expose stored answers.

- [ ] **DOC-037 — Specify six-format capture and trust boundaries.** Retain ANSI, HTML, PNG, ASCII, TXT, JSON, companions, manifest lineage, and separate admission.

- [ ] **DOC-038 — Specify exact current-default parity.** Use independent reference/candidate captures, structural cells, cursor/state, and exact pixels under qualified profiles.

- [ ] **DOC-039 — Specify real interaction and state coverage.** Require events, transitions, targets, actions, and timing, not only assigned visual flags.

- [ ] **DOC-040 — Specify exact required-case/checkpoint aggregation.** Missing, stale, failed, cancelled, or required-skipped evidence must not count as success.

- [ ] **DOC-041 — Specify enforcement status honestly.** Mark checks as existing/source-checked, executed, or specified for later work. Do not claim hooks or CI were installed.

## Checklist, consistency, and language

- [ ] **DOC-042 — Preserve existing checklist IDs and valid evidence.** Reopen only unsupported or stale statuses with a recorded reason and prior receipt.

- [ ] **DOC-043 — Separate document acceptance from code-remediation tasks.** Known code defects remain pending. Their presence does not mean these document edits have fixed them.

- [ ] **DOC-044 — Update AGENTS, GOAL, summaries, and scoped instructions.** Make rules reachable at task start. Preserve sign-off and CLAUDE symlink conventions.

- [ ] **DOC-045 — Reconcile duplicate architecture and API authorities.** Use links to canonical contracts. Remove contradictory current instructions.

- [ ] **DOC-046 — Update task and PR review templates.** Require component ownership and exact-subject evidence. Do not alter executable verifier logic.

- [ ] **DOC-047 — Use controlled English and a project glossary.** Use active short instructions and consistent technical terms. Record language-check limitations.

- [ ] **DOC-048 — Check document links, JSON, IDs, and example labels.** Run deterministic document-pack validation without treating it as runtime validation.

- [ ] **DOC-049 — Prove the task changed only allowed documents.** Compare with the task start. Source, tests, build inputs, workflows, and approvals remain unchanged.

- [ ] **DOC-050 — Obtain a fresh final documentation review.** The reviewer did not author the integrated documents. Resolve confirmed findings.

- [ ] **DOC-051 — Publish a factual final documentation report.** List reviewed commits, document checks, example status, and pending code/enforcement work.

## Verification record

For a verified row, record the document path/section, source inputs, checked revision or tree,
validation command, result, reviewer, and exact reviewed subject.
A tool exit code alone does not prove the prose is correct.
Do not mark a code-remediation item complete when only its specification exists.
