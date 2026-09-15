# Campaign executor adaptation

The package AGENTS.md files preserve the byte-canonical task-format template at `52d9f1eb7721f409bc47beb9fced7997b5c13ede`, with task-ID substitution only. This campaign adaptation is an explicit package requirement. It supersedes the template's executor-authoritative ordered-check/full-verification sequence and its `--progress ""` invocation, not task/v5, verify/v2, task-meta/v1 or final progress validation.

Operator-facing execution: [Campaign execution prompt](campaign-execution-prompt.md) (frozen task catalog handoff) and [Campaign iteration guide](campaign-iteration-guide.md) (visual gate tiers and targeted filters).

The executor owns candidate edits and claimed progress, not trusted contexts, result authority, source freeze, taskfmt configuration or integration. It may run ordinary compiler/tests as advisory feedback using its own temporary outputs. Such local checks must not use production expected bundles, impersonate `/proof/bin`, create accepted receipts, or count as the canonical checks. No progress-disabled invocation authorizes completion or integration.

## Initial bootstrap exception

TASK-001 cannot require its own future accepted host executable. For that task only, the trusted operator performs the initial checkout/freeze/progress handling outside the executor, invokes the currently available pinned standalone taskfmt with explicit configuration/base/nonempty progress, and runs the planner-frozen comparator and host bootstrap drivers with their independent observer. The submitted `tc-proof-host` is the untrusted subject of those fixtures, not authority over its own real campaign checkout or receipt. The operator independently inspects the allowed filesystem map and immutable verification checkout, records actual driver/taskfmt results and source/trust hashes, and rebuilds/accepts the first host product only after complete qualification. This is the initial manually controlled trust root, not an undocumented existing host command. The subsequent workflow below applies only once that host receipt exists; TASK-001's README/verify argv remain the executable independent acceptance contract.

## Operator bootstrap checklist (IW-03)

The manual bootstrap exception above is not operator discretion prose. Before the **first** `tc-proof-host` receipt (`qualified-harness`) may enter the protected ledger, the trusted operator must complete every checklist step, record every required evidence field, avoid every forbidden shortcut, and obtain coordinator sign-off at every gate below. A receipt issued without this record poisons downstream dependency resolution.

Machine authority (schema `tc-proof-operator-bootstrap-checklist/v1`):

```json
{
  "schema": "tc-proof-operator-bootstrap-checklist/v1",
  "repair_id": "IW-03",
  "task_id": "TASK-001",
  "exception": "manual_bootstrap_trust_root",
  "applies_before": "first_tc-proof-host_receipt",
  "evidence_fields": [
    {"id": "EV-001", "name": "ready_catalog_sha256", "required": true, "binds": "READY FOR REFACTORING EXECUTION catalog SHA"},
    {"id": "EV-002", "name": "architecture_source_commit", "required": true, "binds": "campaign worktree (.worktrees/campaign) HEAD on refactor/holla-parity at dispatch"},
    {"id": "EV-003", "name": "catalog_sha256", "required": true, "binds": "immutable terminal-components catalog root"},
    {"id": "EV-004", "name": "bootstrap_package_sha256", "required": true, "binds": "planner-frozen comparator/host driver and fixture package"},
    {"id": "EV-005", "name": "taskfmt_revision", "required": true, "binds": "52d9f1eb7721f409bc47beb9fced7997b5c13ede"},
    {"id": "EV-006", "name": "taskfmt_fingerprint", "required": true, "binds": "pinned standalone taskfmt compiled fingerprint"},
    {"id": "EV-007", "name": "taskfmt_config_sha256", "required": true, "binds": "/proof/bootstrap/experiment.toml bytes"},
    {"id": "EV-008", "name": "comparator_driver_command", "required": true, "binds": "proof-comparator-bootstrap.py against submitted tc-proof"},
    {"id": "EV-009", "name": "comparator_driver_exit", "required": true, "binds": "0"},
    {"id": "EV-010", "name": "comparator_vector_pass_count", "required": true, "binds": "72 authored vectors"},
    {"id": "EV-011", "name": "comparator_recovery_pass_count", "required": true, "binds": "69 fresh positive recoveries"},
    {"id": "EV-012", "name": "submitted_tc_proof_executable_sha256", "required": true, "binds": "candidate comparator bytes under test"},
    {"id": "EV-013", "name": "comparator_rebuild_executable_sha256", "required": true, "binds": "must equal EV-012 after independent rebuild"},
    {"id": "EV-014", "name": "host_driver_self_test_exit", "required": true, "binds": "host-bootstrap-driver.py --self-test"},
    {"id": "EV-015", "name": "host_driver_observer_test_exit", "required": true, "binds": "host-bootstrap-driver.py --observer-test"},
    {"id": "EV-016", "name": "host_matrix_exit", "required": true, "binds": "host-bootstrap-driver.py --host against submitted tc-proof-host"},
    {"id": "EV-017", "name": "submitted_tc_proof_host_executable_sha256", "required": true, "binds": "candidate host bytes under test"},
    {"id": "EV-018", "name": "host_rebuild_executable_sha256", "required": true, "binds": "must equal EV-017 after independent rebuild"},
    {"id": "EV-019", "name": "verification_checkout_tree_sha256", "required": true, "binds": "immutable standalone taskfmt verification checkout"},
    {"id": "EV-020", "name": "scope_base_commit", "required": true, "binds": "explicit taskfmt --base"},
    {"id": "EV-021", "name": "progress_sha256", "required": true, "binds": "nonempty frozen progress.md"},
    {"id": "EV-022", "name": "taskfmt_verify_exit", "required": true, "binds": "0"},
    {"id": "EV-023", "name": "taskfmt_verify_last_line", "required": true, "binds": "DONE"},
    {"id": "EV-024", "name": "check_log_sha256", "required": true, "repeat": "CHK-001..CHK-007", "binds": "actual per-check taskfmt logs"},
    {"id": "EV-025", "name": "allowed_filesystem_map_sha256", "required": true, "binds": "operator-inspected writable/readonly map"},
    {"id": "EV-026", "name": "trust_root_sha256", "required": true, "binds": "authority.json + campaign.json materialization used for first receipt"},
    {"id": "EV-027", "name": "first_receipt_sha256", "required": false, "binds": "populated only after SO-007; names the issued qualified-harness receipt"}
  ],
  "forbidden_shortcuts": [
    {"id": "FS-001", "forbidden": "use_submitted_host_as_own_gate", "reason": "tc-proof-host cannot authorize its own bootstrap"},
    {"id": "FS-002", "forbidden": "issue_receipt_before_complete_qualification", "reason": "false first receipt poisons ledger"},
    {"id": "FS-003", "forbidden": "taskfmt_verify_empty_progress", "reason": "AGENTS step 7 bypass; use campaign host handoff"},
    {"id": "FS-004", "forbidden": "executor_local_done_without_host_verify", "reason": "local taskfmt verify is advisory only"},
    {"id": "FS-005", "forbidden": "advisory_compiler_tests_as_completion", "reason": "cannot impersonate /proof/bin or create receipts"},
    {"id": "FS-006", "forbidden": "skip_comparator_bootstrap_driver", "reason": "comparator must pass independent 72+69 corpus"},
    {"id": "FS-007", "forbidden": "skip_host_observer_test", "reason": "observer IPC and replay denial are mandatory"},
    {"id": "FS-008", "forbidden": "skip_host_self_test", "reason": "lying-host rejection must pass before matrix"},
    {"id": "FS-009", "forbidden": "accept_without_rebuild_hash_match", "reason": "submitted bytes must match independent rebuild"},
    {"id": "FS-010", "forbidden": "copy_forged_or_stale_driver_output", "reason": "driver results must be from this dispatch session"},
    {"id": "FS-011", "forbidden": "use_taskfmt_run_monitor_promote", "reason": "forbidden lifecycle API per proof contract"},
    {"id": "FS-012", "forbidden": "candidate_authored_receipt", "reason": "only protected host/operator may create receipts"},
    {"id": "FS-013", "forbidden": "integrate_before_host_verdict", "reason": "integration requires protected verify record"}
  ],
  "coordinator_sign_off": [
    {"id": "SO-001", "gate": "planning_authorization", "requires": ["EV-001"], "actor": "coordinator", "before": "TASK-001 dispatch"},
    {"id": "SO-002", "gate": "architectural_worktree", "requires": ["EV-002"], "actor": "operator", "before": "candidate edits"},
    {"id": "SO-003", "gate": "bootstrap_inputs_frozen", "requires": ["EV-003", "EV-004", "EV-005", "EV-006", "EV-007"], "actor": "operator", "before": "driver invocation"},
    {"id": "SO-004", "gate": "independent_driver_qualification", "requires": ["EV-008", "EV-009", "EV-010", "EV-011", "EV-012", "EV-013", "EV-014", "EV-015", "EV-016", "EV-017", "EV-018"], "actor": "operator", "before": "standalone taskfmt gate"},
    {"id": "SO-005", "gate": "standalone_taskfmt_gate", "requires": ["EV-019", "EV-020", "EV-021", "EV-022", "EV-023", "EV-024"], "actor": "operator", "before": "filesystem inspection"},
    {"id": "SO-006", "gate": "filesystem_and_trust_inspection", "requires": ["EV-025", "EV-026"], "actor": "operator", "before": "first receipt issuance"},
    {"id": "SO-007", "gate": "first_receipt_authorized", "requires": ["EV-027"], "actor": "coordinator", "before": "TASK-002+ host handoff"}
  ],
  "checklist_steps": [
    {"id": "OB-001", "action": "Record READY FOR REFACTORING EXECUTION catalog SHA and operator dispatch authorization.", "evidence": ["EV-001"], "sign_off": "SO-001"},
    {"id": "OB-002", "action": "Create or verify .worktrees/campaign on refactor/holla-parity via campaign-init.sh; record HEAD.", "evidence": ["EV-002"], "sign_off": "SO-002"},
    {"id": "OB-003", "action": "Verify pinned catalog, bootstrap package, and taskfmt revision/fingerprint/config.", "evidence": ["EV-003", "EV-004", "EV-005", "EV-006", "EV-007"], "sign_off": "SO-003"},
    {"id": "OB-004", "action": "Run comparator bootstrap driver; record exit, vector counts, submitted and rebuilt comparator SHA-256.", "evidence": ["EV-008", "EV-009", "EV-010", "EV-011", "EV-012", "EV-013"], "sign_off": "SO-004"},
    {"id": "OB-005", "action": "Run host driver self-test, observer-test, and full host matrix; record exits and submitted/rebuilt host SHA-256.", "evidence": ["EV-014", "EV-015", "EV-016", "EV-017", "EV-018"], "sign_off": "SO-004"},
    {"id": "OB-006", "action": "Perform operator checkout/freeze/progress handling; run pinned standalone taskfmt with explicit base and nonempty progress; record tree, logs, exit, and DONE.", "evidence": ["EV-019", "EV-020", "EV-021", "EV-022", "EV-023", "EV-024"], "sign_off": "SO-005"},
    {"id": "OB-007", "action": "Independently inspect allowed filesystem map and trust-root materialization.", "evidence": ["EV-025", "EV-026"], "sign_off": "SO-006"},
    {"id": "OB-008", "action": "Coordinator authorizes and records first qualified-harness receipt SHA-256.", "evidence": ["EV-027"], "sign_off": "SO-007"}
  ]
}
```

Prose binding: comparator and host driver commands, observer requirements, and receipt schema remain in [host-bootstrap-protocol.md](evidence/host-bootstrap-protocol.md), [proof-comparator-protocol.md](evidence/proof-comparator-protocol.md), and [proof-contract.md](proof-contract.md). TASK-001 package README D-006 requires this checklist before any first receipt claim.

## Executable handoff

1. The operator invokes the existing future host command `tc-proof-host prepare --campaign /absolute/campaign --task terminal-components/completion/NNN --parent INTEGRATION_PARENT_COMMIT --run /absolute/run`. Those are host-resolved paths/identities. The pinned `taskfmt --config /proof/bootstrap/experiment.toml progress-init /absolute/catalog/terminal-components/completion/NNN --out /absolute/run/progress.md` supplies initial canonical progress. The executor receives a writable progress copy, the task and allowed candidate checkout; never host credentials.
2. The executor finishes scoped edits and writes the canonical progress completion claim, including actual advisory commands/results. It places its progress file and a plain request naming the task and candidate checkout in the operator-visible outbound directory. This request is untrusted data, not a CLI, queue service, approval or proof. The operator reads it and explicitly invokes the host commands below. There is no new automatic scheduler.
3. The operator installs the claimed progress into this run's untrusted input location and invokes `tc-proof-host freeze --run /absolute/run --candidate /absolute/executor-checkout`, then `tc-proof-host verify --run /absolute/run`. Freeze revokes executor mutation access and records the source filesystem map, exact tested tree, scope base and frozen progress. Verify uses the pinned taskfmt configuration, explicit base, nonempty frozen progress and immutable per-check contexts; it executes all canonical checks with isolated subordinate workers. The host independently requires exit 0 and final `DONE`, complete check evidence and unchanged trust/source trees.
4. The operator returns read-only copies of the actual host verdict and check logs to the executor. They bind run/task/tree/context-index and progress hashes. The executor reports that host verdict faithfully, or uses its failure details to revise the candidate and submit a new freeze/verification request. Every revision requires a fresh full frozen gate. A copied, forged, stale or cross-run report cannot authorize integration: the host resolves its own protected run record, never the executor's returned copy.
5. Only after the protected record passes does the operator invoke the existing host integration command with the recorded expected parent. Canonical completion remains a worker claim until this host evidence exists. The host verifies complete progress at the authoritative gate; it never accepts an empty progress bypass or synthesizes taskfmt monitor lifecycle state.

TASK-001 owns this thin host workflow and its independent hostile-index/overlay/context/forged-record qualification. TASK-070 owns operation evidence and closure joins. This document describes future implementation contracts, not commands available during the planning goal.
