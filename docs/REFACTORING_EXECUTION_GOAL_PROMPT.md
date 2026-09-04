# Goal Execution Prompt: Finish the Refactor

Copy the block below into /goal.

~~~text
Read docs/REFACTORING_AUDIT_REPORT.md first, then complete every remaining requirement in GOAL.md and REFACTORING_GOAL.md. Finish the repository, not merely the tui-next library.

Use the current code as truth. Treat markdown state as a checklist only. Do not declare work done from docs, TODO movement, existing pre-refactor baselines, or a passing vacuous check.

Model policy

- Spawn subagents aggressively.
- Every subagent must use model gpt-5.6-luna with max reasoning.
- Do not silently substitute another model. If that model is unavailable, report the blocker.
- Start with parallel read-only scouts, then run independent implementation lanes in parallel.
- Use separate git worktrees/branches or strict disjoint file ownership. One integrator owns merges. Never have two agents edit the same file concurrently.
- Keep a live evidence ledger: completed, in progress, blocked, command/capture evidence, and exact file/line references.
- Preserve existing user changes. Never reset, checkout, clean, or overwrite unrelated work.

Priority rule

Start from the unfinished end of the goal. Prioritize the application migration, package boundary, product behavior, and legacy removal in the later goal sections before broad new foundation refactors.

Use the existing tui-next foundations. If an API gap blocks migration, assign a narrow unblocker. Do not reopen or redesign the foundation without concrete migration evidence.

Initial parallel audit

Spawn at least these read-only scouts in parallel. Each returns path:line evidence, affected files, dependencies, and a bounded implementation plan:

1. GOAL.md and REFACTORING_GOAL.md definition-of-done crosswalk.
2. Cargo/package/binary boundary and legacy dependency graph.
3. Showcase migration inventory and product-semantics preservation plan.
4. TablePro migration inventory and generic-grid/adapter boundary plan.
5. Jackin Preview migration inventory and simulation/control preservation plan.
6. New-library API gaps that concretely block app migration.
7. Security/secret-state and redaction audit.
8. Visual/capture/baseline and interaction QA audit.
9. CI/xtask/test/doc gate audit.

After scouts return, freeze file ownership and launch implementation lanes. Do not spend the main execution budget re-reading the same files.

Parallel implementation lanes

Lane A — application package boundary

- Create or complete the required apps/showcase, apps/tablepro, and apps/jackin-preview package/lib/bin layout.
- Move binary ownership out of the root package as required by the goal.
- Make boundary checks scan real application source instead of zero files.
- Keep product-facing binary names and launch behavior stable.

Lane B — Showcase migration

- Migrate Showcase to the new public tui-next facade, runtime, event, focus, layer, theme, and component APIs.
- Remove app-owned focus, hit, hover, pressed, cursor, modal, and scroll plumbing.
- Preserve the full catalog, navigation, keyboard/mouse behavior, custom themes, Junie theme, overrides, color downgrade, overlays, and responsive behavior.
- Replace legacy imports and direct legacy widget construction.

Lane C — TablePro migration

- Migrate TablePro to tui-next and the new runtime/foundation APIs.
- Preserve query safety, pending edits, SQL preview, filters, tabs, history, safe mode, grid behavior, keyboard/mouse behavior, and modal behavior.
- Keep database/product code outside the generic UI library.
- Implement the generic grid surface and a narrow TablePro adapter where required. Do not put DB/domain types into tui-next.
- Remove app-owned interaction routing.

Lane D — Jackin Preview migration

- Migrate Jackin Preview to tui-next.
- Preserve simulation/control semantics, screens, status, dialogs, keyboard/mouse behavior, and resize/scroll behavior.
- Remove legacy imports and app-owned interaction state/routing.

Lane E — API and legacy cleanup

- Work from actual consumer search results.
- Add only narrow compatibility shims needed to land app migrations.
- After all consumers move, remove old src/core, src/ui, src/widgets, old flat theme paths, duplicate APIs, and compatibility code.
- Prove zero production references to the legacy API.

Lane F — security and state hardening

- Ensure secret-bearing controls cannot expose raw values through Debug, Display, Clone, equality, logs, captures, snapshots, diagnostics, or retained dialog state.
- Keep masking/fingerprints useful without storing unnecessary raw secrets.
- Add regression tests for secret input, acknowledgement dialogs, capture redaction, logging, and state transitions.

Lane G — visual and interaction QA

- Build and run each migrated application.
- Capture current evidence for required screens and interactions: Paper, Junie, custom overrides, 256-color, 16-color/mono, responsive sizes, dialogs/overlays, keyboard, mouse, wheel, drag, focus, and resize.
- Compare against approved references and classify every difference as intentional or a bug.
- Generate/update the current capture manifest and findings. Do not bless from before-refactor artifacts.

Lane H — gates and documentation

- Keep fmt, tests, clippy, build, docs, examples, boundary, conformance, baseline, secret-redaction, and named-test gates green.
- Remove diagnostic instrumentation and dead code.
- Replace unjustified allowlists with implemented references, or document a real narrowly scoped deferral with owner and proof.
- Update state/report docs only after fresh evidence exists.

Integration protocol

1. Scouts report first; the integrator records ownership and dependency order.
2. App lanes proceed in parallel after the public contract is identified. Do not wait for unrelated foundation polish.
3. Merge lanes one at a time. After each merge run affected package tests and compile checks.
4. If two lanes need the same foundation file, stop the overlap and split the change into a small contract lane owned by one agent.
5. Every implementation agent returns files changed, behavior preserved, tests run, failures, and remaining TODOs.
6. Every blocker includes the smallest concrete missing contract. Route it to one unblocker; continue all independent lanes.
7. Never weaken a test, broaden an allowlist, delete a check, or mark a baseline blessed merely to get green output.

Required completion proof

Do not stop at an estimated percentage. Stop only when all applicable requirements are evidenced:

- All three applications build and run through the new package/library boundary.
- No application imports or constructs legacy core/ui/widgets/theme APIs.
- No app owns duplicate focus, hit, hover, pressed, cursor, modal, or scroll routing that belongs to the new runtime.
- Generic tui-next remains domain-free, backend-free where required, and independent of database/product types.
- All public components have conformance coverage and an explicit legacy disposition.
- Showcase, TablePro, and Jackin product semantics remain covered by tests and live/capture evidence.
- Secret-bearing state is non-leaking across debug/display/clone/equality/logging/capture/snapshot paths.
- Current visual captures cover required themes, color modes, sizes, overlays, keyboard, mouse, scroll, and resize cases.
- No unresolved accidental TODO/unimplemented/unsafe/dead-code/diagnostic instrumentation remains in required scope.
- Boundary checks inspect real application files and real package dependencies.
- The old API is removed or proven unused, not merely hidden behind a compatibility path.

Run and pass, using the repository rtk wrapper where available:

- rtk cargo fmt --all -- --check
- rtk git diff --check
- rtk cargo check --workspace --all-features
- rtk cargo build --workspace --all-targets --all-features
- rtk cargo test --workspace --all-targets --all-features
- rtk cargo clippy --workspace --all-targets --all-features -- -D warnings
- RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
- rtk cargo test --workspace --doc --all-features
- cargo run -p xtask -- boundary
- cargo run -p xtask -- doc-check
- BLESS_GUARD_BASE=<validated-pre-refactor-base> cargo run -p xtask -- bless-guard

Run the actual application and capture checks. Compile-only results are insufficient. Use a validated pre-refactor base for the baseline guard; never guess or compare against the current dirty tree.

Final response format

Return a terse evidence-backed handoff:

- exact completion percentage and calculation;
- DONE / IN PROGRESS / TODO with file and line evidence;
- every command run and result;
- every application run and capture result;
- remaining blockers, if any;
- worktree status and intentional uncommitted changes;
- no complete claim unless every required gate and migration criterion passes.
~~~

## Execution principle

The first major deliverable should be the three real applications running on the new library. Foundation changes are supporting work, not the main track. Parallelize independent app lanes; serialize only shared-contract changes, merges, and final verification.
