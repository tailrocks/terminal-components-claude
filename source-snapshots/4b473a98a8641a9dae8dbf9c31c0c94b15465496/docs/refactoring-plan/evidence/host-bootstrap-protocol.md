# Independent host qualification ABI, version 1

This is a planner-owned acceptance protocol for the future `tc-proof-host`. It defines test inputs and independently inspected effects. The companion [driver](host-bootstrap-driver.py), [protected observer](host-bootstrap-observer.py), [vectors](host-bootstrap-vectors.json), and [canonical task fixture](host-bootstrap-fixture/README.md) must be frozen outside the B-HARNESS writable checkout before dispatch. The observer is a test-only process supervisor over the existing Darwin sandbox facility; it is not a production host implementation or scheduler. The host implementation must satisfy these tests in addition to the [proof contract](../proof-contract.md) and comparator qualification.

The synthetic fixture is deliberately tiny: `src/payload.txt` changes from the exact bytes `pending\n` to `qualified\n`; `protected.txt` and `.proof/check.py` remain immutable. The checker contains planner-authored expected bytes. No application baseline is captured, approved, or fabricated. Synthetic acceptance receipts are independent test inputs and have no authority in a production campaign.

## Invocation and prerequisites

Python 3.10 or later and Git are required. Host qualification additionally requires macOS, `/usr/bin/sandbox-exec`, and the direct `/Library/Developer/CommandLineTools/usr/bin/git` executable; unsupported isolation fails closed. The observer uses a fixed PATH containing the selected Python directory, direct Command Line Tools directory, `/usr/bin`, and `/bin`. It disables global/system Git configuration and prompts; user shims and the `xcrun` Git launcher do not select the verifier. The current independently inspected taskfmt executable must report revision `52d9f1eb7721f409bc47beb9fced7997b5c13ede` and fingerprint `52c960db74b3b288ce93211c82e5703a338ba5ddfd40c92d6054ef93cfcd94e4`. Supply that source checkout because standalone taskfmt currently also requires an experiment configuration. No taskfmt run, dispatch, monitor, or promote command is used.

```sh
python3 docs/refactoring-plan/evidence/host-bootstrap-driver.py --self-test
python3 docs/refactoring-plan/evidence/host-bootstrap-driver.py \
  --taskfmt /absolute/taskfmt \
  --taskfmt-source /absolute/pinned/task-format
python3 docs/refactoring-plan/evidence/host-bootstrap-driver.py \
  --observer-test --taskfmt /absolute/taskfmt \
  --taskfmt-source /absolute/pinned/task-format
python3 docs/refactoring-plan/evidence/host-bootstrap-driver.py \
  --host /absolute/submitted/tc-proof-host \
  --taskfmt /absolute/taskfmt \
  --taskfmt-source /absolute/pinned/task-format
```

The last invocation is a future B-HARNESS acceptance command. There is no submitted real host executable during planning. `--self-test` runs intentionally incorrect external always-pass, always-fail, and malformed hosts. `--observer-test` executes the independent reviewer's stronger [forged host](host-bootstrap-fixture/forged-host.py), which fabricates denial reports and logs and integrates the correct tree while running zero workers and zero verification checks. That host must reject specifically because independent execution records are missing. The same command then proves actual observer IPC, real build/test/taskfmt execution, denial of host access to observer files/process authority/network, and replay rejection. These results qualify the preparation facility; they do not prove a real B-HARNESS implementation passes. Standalone taskfmt qualification verifies canonical lint, positive checking, out-of-scope rejection, a failed check, incomplete progress, `DONE` with nonzero exit, and absent trusted overlay rejection.

The driver invokes every submitted host process through the frozen observer's Darwin sandbox profile. File contents are unreadable by default. Exceptions are explicit fixture/tool inputs and bounded system/runtime roots (`/System`, `/usr/lib`, `/usr/bin`, `/bin`, Command Line Tools, and the resolved Python prefix). File metadata and the root directory itself are readable for Darwin loader/path checks; this grants no descendant file contents. `/dev/null` and random devices have explicit access. A synthetic secret outside the fixture and observer roots must be unreadable. Signals, other-process inspection/task ports, Mach registration, and network access are denied; only self-process inspection and the system log service are allowed.

Original candidate/probe/package/config/receipt/oracle bytes are kernel-readonly from the first submitted-host invocation. `install` may write only its destination; `prepare`, `freeze`, `verify`, and rejected `seal` may write only the run; `integrate` additionally may write the target repository's `.git` and ledger. Prior immutable receipt/ledger inputs remain protected. This prevents a submitted host or surviving child from replacing a probe before the independent source snapshot. The driver also compares candidate bytes at every host-call boundary. The retained [substituting host](host-bootstrap-fixture/substituting-host.py) must fail at its attempted `src/worker.py` write during prepare. Final-byte hashes alone are not the isolation mechanism.

The observer remains outside submitted-process sandboxes. Candidate workers can read only their own source/output within the observer root and can write only their output. The independently tested interpreter, sandbox executable, observer, and taskfmt hashes are returned by observer qualification. All Git repositories and ref changes are temporary synthetic fixtures; the real repository is never a freeze or integration target. No arbitrary Linux environment or generic unconfigured worker provider is qualified.

## Independently owned execution authority

Submitted host-written reports cannot prove that a process ran or that an access was denied. The host therefore requests actual execution from the already frozen planner observer. This is an external worker-provider interface; B-HARNESS implements its adapter, not its authority. Production worker providers require separately accepted identity and isolation evidence and must preserve the same separation.

The observer supplies three host-only environment values: `TC_PROOF_OBSERVER_REQUEST_FD`, `TC_PROOF_OBSERVER_RESPONSE_FD`, and unpredictable per-fixture `TC_PROOF_OBSERVER_NONCE`. The first two are inherited anonymous pipe descriptors. They are not filenames, and the submitted host cannot replace the observer through an environment path. Worker subprocesses inherit none of these descriptors or variables.

During `verify`, the host writes one newline-terminated JSON request to the request descriptor, flushes, and reads one newline-terminated JSON response from the response descriptor. The exact ordered requests are:

```json
{"schema":"tc-proof-observer-request/v1","nonce":"HOST_SUPPLIED_NONCE","step":"build"}
{"schema":"tc-proof-observer-request/v1","nonce":"HOST_SUPPLIED_NONCE","step":"test"}
{"schema":"tc-proof-observer-request/v1","nonce":"HOST_SUPPLIED_NONCE","step":"taskfmt"}
```

The nonce is a placeholder for the observer-supplied value, not a fixture constant. Requests are bounded to 4096 bytes, require a terminal newline, are one-shot, and permit only these keys and this order. Duplicate keys, unknown fields, wrong nonce, unarmed execution, reordering, and replay fail. The host cannot supply source paths, a command, expected bytes, progress, or a substitute executable. Original inputs are protected before the first host call; the observer copies the driver-owned candidate/package/config/progress into its private root before freeze. It launches fixed worker argv or pinned standalone taskfmt against those copies, using the recorded overlay scope base. Neither a host context nor a host-written tree ID can change these inputs.

A response uses `tc-proof-observation/v1`, with `step`, `tree`, actual `argv`, integer process `exit`, base64 `stdout`/`stderr`, and `files` mapping relative output/log names to base64 bytes. The observer retains this actual evidence in its own memory before sending any response. The host copies worker files into the normal run layout and uses the actual taskfmt check logs in its verdict. The driver compares those copies byte-for-byte against retained observer evidence, requires all three actual successful launches, validates the taskfmt terminal `DONE`, and requires every check log. A forged success string, correct Git tree, or fabricated denial JSON cannot replace an observer event.

`tc-proof-observer-error/v1` indicates a protocol violation. A failed subprocess remains a real failed observation and cannot authorize a passing verdict. Once a request has been made it cannot be reset or replayed in that fixture. The observer conformance gate also attacks its own file boundary, signal boundary, Mach task-port boundary, and network denial through a submitted-process sandbox; unsupported or failed protection stops qualification.

## Host entrypoint

The exact CLI is the host-only six-operation CLI in [proof-contract.md](../proof-contract.md). There are no extra qualification-only host operations or permissive flags. Unknown options fail. `--campaign` names a directory containing `campaign.json`. `install --destination` creates `bin/tc-proof-host` with exactly the accepted executable bytes. Later operations run that installed executable.

The host operator supplies `TC_PROOF_AUTHORITY_FILE=/absolute/authority.json` to the initial and installed host executable. This variable is an authority selection input only at the protected host entrypoint. Never copy it to a candidate, accept a candidate-defined replacement, or resolve it from a task/context. Production may provision the same authority through a fixed service configuration instead; this CLI qualification entrypoint remains available only to the operator. Candidate processes must be unable to read or write the authority file.

Every host command emits exactly one JSON object on stdout. Diagnostics go to stderr. Duplicate keys, invalid JSON, and unknown schema versions fail driver validation. The object contains:

```json
{"schema":"tc-proof-host-result/v1","operation":"freeze","status":"passed"}
```

A rejection has `status: "rejected"`, nonzero process exit, and the exact `category` named by the applicable vector. A passed result requires process exit zero. Result text never substitutes for filesystem evidence. A positive install requires an exact installed executable hash; positive freeze, verify, and integration require the independently inspected artifacts below. Infrastructure failure or unsupported isolation cannot be reported as a passing/rejected test case: the driver fails if the required positive case cannot run.

## Operator-authored authority and campaign

The driver's `HostFixture` class materializes the normative version-1 JSON fixture. It creates the receipt and campaign bytes itself before starting the submitted executable. Absolute paths in these local invocation records locate independently pinned artifacts; they are not portable content-addressed bundle manifests. The portable fixture package is hashed separately before dispatch.

`authority.json` uses `tc-proof-host-authority/v1` and contains `campaign` (absolute campaign.json path), `campaign_sha256`, and `accepted_receipts` (mapping of exact SHA-256 to absolute receipt file). This allowlist is the external bootstrap root. A receipt with changed bytes is rejected even if its fields still look valid. A well-formed independently accepted receipt from the wrong producer is rejected by dependency resolution. Production acceptance must additionally bind the complete provenance and review evidence required by the proof contract; the synthetic root grants authority only inside its fresh temporary campaign.

Receipts use `tc-proof-host-receipt/v1`: `producer`, `product`, `source_commit`, `source_tree`, and `dependencies`. The synthetic harness receipt additionally names `executable` and `executable_sha256`; its externally pinned bytes authorize only that binary installation. A prerequisite receipt must match the task's exact producer/product tuple, and its accepted commit must be an ancestor of the requested integrated parent. Neither a receipt's existence nor an ancestor somewhere on another branch suffices.

`campaign.json` uses `tc-proof-host-campaign/v1`:

- `repository` and `integration_ref` identify the only permitted local integration target.
- `catalog_root` and `catalog_sha256` bind the external canonical package directory.
- `receipt_root`, `ledger_root`, and `harness_receipt_sha256` identify protected authority state.
- `taskfmt` binds executable path/hash, source revision, fingerprint, and experiment config location/hash. The host validates its executable and environment independently before running it.
- `tasks` maps a concrete external task identity to its relative `package`, `package_sha256`, required dependency tuples/digests, allowed `seal_products`, exact `required_checks`, candidate `workers`, and `artifact_comparisons`.

The fixture package keeps canonical `task/v5`, `verify/v2`, and `task-meta/v1` unchanged. These host fields do not become taskfmt fields. `writable_paths` and forbidden paths come from immutable `verify.toml`, never candidate Git configuration. Every positive host fixture has an approved `.proof/overlay.txt` addition: `trusted_overlay` binds its exact path/hash, original `parent`, and resulting `scope_base`. That scope base differs from the integration parent. The overlay remains in the frozen, tested, and integrated tree; stripping it before integration fails exact-tree equality. Modifying it fails the forbidden-path gate.

Each worker has an ID, an argv array, and an exact `required_outputs` list. The host requests build then test through the external observer as arbitrary candidate code, from the independently frozen source copy. `TC_PROOF_WORKER_OUTPUT` names an empty worker-owned output directory. The host validates and copies the observer-returned regular output files into `run/workers/<worker-id>/`; missing, duplicate, escaping, or extra output identities fail. The driver checks these files against the observer's retained bytes. The test worker emits `payload.txt`; the host compares its bytes against the independently pinned expected file. The candidate receives no expected file or authority file. These simple artifact comparisons qualify host plumbing; the independent schema-3 frame comparator remains a separate required gate.

The temporary catalog's worker program deliberately attempts to open protected files for writing, change bytes, and restore the original bytes. A successful temporary write must fail qualification even if final hashes match. It also attempts to read expected bytes and signing/authority material. Separate build and test observations must report denial for every target and absence of the authority environment variable. The observer checks those observations and preserved protected state. The hostile program is a planner-authored input, not a submitted host test or claimed pass marker.

For this fixture, a directory identity is SHA-256 of UTF-8 JSON with sorted keys and compact separators, with no terminal newline, produced by the driver's `snapshot` function. Every relative path maps to `["file", SHA256]`, `["directory"]`, or `["link", target]`. Symlinks in accepted input directories are still forbidden by the main proof contract; the snapshot representation permits the observer to detect their introduction. This qualification identity does not replace the production bundle manifest format.

## Independently checked artifacts and effects

`prepare` creates administrative `run/preparation.json` containing exact `task` and `parent`, and initializes canonical `run/progress.md` using pinned taskfmt. This administrative record is never an operation context. The driver appends a valid complete synthetic event stream. Production executors maintain their own progress; a completion claim is never proof.

`freeze` creates `run/freeze.json` containing exact `tree`, `parent`, and `scope_base`. Every positive candidate intentionally retains its unchanged initial index while containing an unstaged tracked payload change, deletion of tracked `src/obsolete.txt`, and allowed untracked source/probe files. The driver computes the expected tree using a separate private repository/index; it never stages the executor checkout to manufacture a passing case. The expected tree must differ from the candidate's stale index tree. The stale index must still list `src/obsolete.txt`, while the independently expected and integrated trees must omit it. The host reconstructs actual source bytes into its own object database. Out-of-scope files, modified checker/overlay, symlink or hard-link escape, ignored source, hidden index flags, changed Git configuration, and submodule substitution reject. Original executor mutations after freeze must not reach verification or integration.

The same freeze writes `context-index.json` and exactly five `contexts/CHK-NNN.json` files. The shared index schema is `tc-proof-context-index/v1`: `run_id`, `task_id`, `tree`, `trust_sha256`, and `members`. Each member has `check_id`, relative `context_path`, `context_sha256`, `schema`, `operation`, `lane`, `namespace`, `required_ids`, and `output_id`. `run_id` is the observer nonce also pinned in campaign configuration. `trust_sha256` hashes canonical JSON containing `catalog_sha256`, `harness_receipt_sha256`, and ordered dependency receipt hashes. The independently supplied task `check_context_templates` declares every child field except run/task/tree bindings; no host hardcoding of fixture operations is required.

Synthetic children use `tc-host-fixture-check-context/v1`, with exactly their template fields plus `run_id`, `task_id`, and `tree`. Comparator/runner children retain their own strict schemas in production. Canonical JSON means sorted keys, compact separators, UTF-8, and one terminal newline. The driver independently constructs and validates every child and index before verification. It then protects those files, freeze record, and progress against host writes and supplies `TC_PROOF_CONTEXT_INDEX` plus `TC_PROOF_CONTEXT_INDEX_SHA256`. Six negative fixtures tamper with the index, mutate/substitute a child, omit/add/swap members, or replay another run; all must reject. The runner companion qualifies actual multi-operation selection and close against this same index schema.

`verify` executes every candidate worker, exact artifact comparison, and pinned standalone taskfmt check. `run/verdict.json` contains `status: "passed"`, exact `tree`, `parent`, `scope_base`, `task_id`, `run_id`, `context_index_sha256`, and `checks`. Each of the five check IDs appears exactly once with integer `exit: 0`, relative `log`, and `log_sha256`. The driver compares every regular run-contained log against independently captured taskfmt bytes and revalidates the immutable index/members. Existing accepted receipt/catalog/expected/install/ledger state cannot change during verify; new run outputs stay inside the run.

After these postchecks, `protect_verified_run` retains the exact verdict bytes that were read once and validated, plus actual observer-captured logs and worker files, under the observer-private `accepted-proof/` root. This root is inaccessible to submitted processes from their first invocation. The accepted verdict digest comes from those retained validated bytes, never from a later read of public `run/verdict.json`. `validate_accepted_copy` compares public verdict/log/worker copies against that private authority before and after integration. New host invocations receive a read-only run, and direct attempted mutation-plus-restoration of verdict/log/private-proof files must be denied.

Changing a future sandbox profile does not revoke a surviving earlier child's write rights. Public run files therefore remain untrusted publication copies, even after verification. The observer gate explicitly starts an old-rights detached child before sealing, lets it mutate the public verdict afterward, and requires integrity rejection while the private accepted bytes/digest remain unchanged. Restoring the public copy must restore the valid case. No claim of descendant termination or retroactive kernel revocation is made; an old child cannot rebind the accepted proof or ledger digest.

`seal` rejects the fixture task at every stage because its immutable `seal_products` is empty. Successful sealing of a real oracle product additionally requires the independent comparator and source-rebuild/repeat/review evidence; this host fixture does not claim to qualify application oracle capture.

`integrate` accepts only the configured ref and exact prepared parent. It creates a commit whose sole parent and tree match the independently verified values, with DCO signoff and `Co-authored-by: Codex <codex@openai.com>`. The ref update must use compare-and-swap. A different requested parent, an already advanced ref, or `refs/heads/main` must reject without changing any ref or accepted artifact. The driver checks Git objects and refs directly after the operation.

This synthetic task has no seal products, so successful integration adds no receipt files. Its only permitted ledger addition is `<integrated-commit>.json`, exactly `tc-proof-host-acceptance/v1` with `task`, `commit`, `tree`, `parent`, `scope_base`, `verdict_sha256`, and `dependencies: [accepted predecessor receipt SHA256]`. `verdict_sha256` must equal the independently retained private digest. Every value must match independently checked state. Extra keys, arbitrary products, extra ledger entries, replacement prior entries, new receipt files, unrelated ref changes, or publication-copy drift fail. The driver preserves all prior ledger/receipt bytes and validates this exact single authorized append.

For each single negative, the driver snapshots protected state immediately before invoking the rejecting command and requires exact preservation afterward. It then creates a fresh untouched positive campaign and requires full successful integration. This prevents an always-reject implementation from passing a collection of negative tests. The ordered mutation corpus is [host-bootstrap-vectors.json](host-bootstrap-vectors.json).

## Current evidence and remaining acceptance

The planner ran the canonical fixture lint and standalone taskfmt matrix against the pinned executable. Driver self-tests reject simple lying executables. The protected observer gate rejects the reviewer's stronger zero-execution forgery, then positively executes real build/test/taskfmt processes and verifies the observer boundary and replay denial. No real `tc-proof-host` implementation has been qualified. B-HARNESS must implement this ABI and provide actual positive, negative, isolation, receipt, freeze, and CAS evidence before any downstream production task can use its receipts.

The host matrix covers the bounded synthetic plumbing above. It does not alone prove complete oracle provenance, arbitrary artifact parser safety, all production toolchain dependencies, source adaptation validity, or the complete historical test accounting inventory. Those remain explicit B-HARNESS acceptance obligations in the main proof contract and comparator/runner protocols. The concrete external observer gate is mandatory; no submitted-host self-attestation can waive it. No omitted gate is implicitly accepted.
