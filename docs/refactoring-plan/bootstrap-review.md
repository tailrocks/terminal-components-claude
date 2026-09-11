# Independent bootstrap review

Status: the comparator and Darwin host preparation are accepted for their explicitly bounded qualification profiles. BR-01 through BR-05, including the late-freeze and surviving-child evidence problems, are resolved in the exact revisions recorded below. VF-02 aggregation and the host portions of INT-03/VF-03 are independently re-tested. BR-06's finite synthetic runner checks pass; its actual Rust preparation remains a separate independent review and is not approved by this document. This review qualifies planning fixtures, not a production harness. No terminal-components production source was changed, no oracle was blessed, and no real integration ref was updated. All adversarial executions used temporary synthetic repositories.

## Scope and acceptance

The review examined `proof-contract.md`, both bootstrap drivers, both vector corpora, both machine protocols, and the canonical host fixture. Acceptance requires that the independent drivers enforce their stated result schemas, that advertised isolation and execution evidence comes from outside the submitted implementation, and that independently prepared qualification cases cover the bootstrap requirements. Driver self-tests alone do not satisfy production harness acceptance.

The review used the verification-only skill to keep work within evidence collection and stopped short of implementing the future comparator or host. Evidence repairs belong to the original authors.

## Executed evidence

- Comparator self-test passed: 57 fixture constructions and four deliberately incorrect runners rejected. No real comparator was supplied.
- Host self-test passed: seven tests, including actual external always-pass, always-fail, and malformed executables.
- Pinned standalone taskfmt qualification passed all six expected outcomes: positive accepted; out-of-scope edit, failed check, incomplete progress, `DONE` with nonzero exit, and missing trusted checker rejected. The tested executable SHA-256 was `34acc170d7d4f6b7141c04c02a959e196f7031baa2fc978ee421629990a2aff2`.
- An independent comparator liar passed all 57 cases and all 113 invocations without comparing any artifacts. It read the exposed case name from `context.run_id`, looked up its expected category in the public vector file, and emitted the expected verdict. Every rejection omitted the mandatory `checked_count` and `results` fields. Qualification still returned `failures: []` and exit 0.
- An independent fake host passed the complete positive `HostFixture.execute()` path while running zero candidate workers and zero `taskfmt verify` checks. It copied its own executable, initialized progress, reported the actual fixture tree, fabricated denied-access observations and hashed logs, and created a commit with the required tree, parent, and trailers. The independent observer accepted that fabricated worker evidence. This was a positive-path probe, not a claim that this fake passed all host negative cases.

The adversarial probes are retained in `/tmp/tc-bootstrap-review.slC97f/`. They are intentionally incorrect test programs, not production implementations. Their commands were:

```sh
python3 docs/refactoring-plan/evidence/proof-comparator-bootstrap.py --self-test
python3 docs/refactoring-plan/evidence/host-bootstrap-driver.py --self-test
python3 docs/refactoring-plan/evidence/host-bootstrap-driver.py --taskfmt /Users/donbeave/.cargo/bin/taskfmt --taskfmt-source /Users/donbeave/Projects/donbeave/task-format
python3 docs/refactoring-plan/evidence/proof-comparator-bootstrap.py --runner /tmp/tc-bootstrap-review.slC97f/comparator_liar.py
python3 /tmp/tc-bootstrap-review.slC97f/run_host_liar.py
```

The comparator false-positive occurred with driver SHA-256 `68a4f58de886c7d820aa47a64614a07d7c87e19c9d5bbb7013717c97857edbb2`, vector SHA-256 `ec880dfd96b01c2b4d5adf3d4be0ff60eb82979bef0c530c2177a6cabd967315`, and liar SHA-256 `8800fa6deb961850c082fd8d05fcb6c5db9cbe4ad0ce01c3356062a6f7763c04`. Later repairs require a new evidence identity.

## Findings

### BR-01: Rejected comparator reports did not enforce the complete schema

The original `invoke` negative branch checked a failure category and zero passing scenarios, but defaulted a missing `results` array to empty and never checked `checked_count`. It also used Python equality where `True == 1` and `False == 0`, without exact integer type validation. This admitted incomplete reports despite the protocol requiring every invocation to write every field.

Required repair: validate exact required fields, supported schema, field types, nonnegative consistent counts, result membership and failure structure for both success and rejection. Add self-tests for missing negative fields, boolean counts, malformed arrays, and inconsistent counts. The protocol must define the meaning of checked counts on early input rejection.

### BR-02: Comparator test identity exposed the expected verdict

The original context used `run_id = "bootstrap-" + case["id"]`, and temporary directory names also exposed mutation labels. A runner could return the fixture answer key without inspecting input artifacts. This does not make all black-box testing impossible; it shows that this driver unnecessarily provided an answer-key selector.

Required repair: generate opaque run IDs and invocation directory names, separate the private case-to-verdict mapping from submitted inputs, randomize invocation order, and reject the independent label-lookup liar. Continue source review and independent rebuild: a finite public suite never proves a submitted program free of deliberate test-specific behavior.

### BR-03: Corrupt required-set input left required count unspecified

The original context provided only `required_sha256`, not trusted required IDs or count. The result protocol nevertheless demanded `required_count = 1` even when `required.json` was missing or corrupt, stating that the context contained sufficient membership authority. A general comparator cannot derive a count from a SHA-256 digest.

Required repair: include protected required IDs/count in the context and validate their consistency when the payload is readable, or explicitly define a fail-closed unknown-count rejection schema. Do not hardcode the one-scenario fixture count into a production implementation.

### BR-04: Host observations did not independently prove worker execution or isolation

`HostFixture.execute()` read logs and denial observations from paths written by the submitted host. Hashing those files proved their reported bytes, not that a worker ran or an access attempt was denied. The fake host demonstrated the distinction by passing without running a worker or verification check. The `authority_env_present` field had the same circular authority problem.

Required repair: put actual process execution and isolation observations behind an independently trusted observer or supervisor that the submitted host cannot impersonate, modify, or replay. Bind its observations to fresh invocation identity, exact executable/tree, actual process status, worker launch and attempted accesses. Require this evidence at the acceptance entrypoint. If a separate observer is not yet implemented, the present host suite must explicitly report only filesystem/protocol plumbing and must not claim independent execution/isolation qualification. Merely adding more host-authored JSON fields does not fix the root cause.

### BR-05: Host protected-state coverage omitted declared targets and successful integration side effects

The original `protected()` snapshot omitted `campaign/secret` and `campaign/other-run`, although both were declared worker protection targets. Positive integration checked the selected ref's tree/parent but did not ensure unrelated refs and accepted artifacts stayed unchanged. Initial installation accepted a symlink because `is_file()` followed it.

Required repair: snapshot every declared protected target, reject nonregular installed executables, and compare all protected state across successful integration while permitting only the explicitly authorized ref and ledger update. Add deliberate mutants for each omitted effect. Final-byte preservation remains distinct from denied temporary mutation and cannot replace BR-04's external observation.

### BR-06: Required bootstrap case coverage remained incomplete

The main proof contract requires independent qualification for partial test execution, dropped or renamed required tests, unknown failures, regressions of previously closed scenarios, candidate-modified allowed-failure maps, and incomplete subordinate results. The reviewed comparator and host suites do not exercise `account-tests` or the historical inventory/stage-map rules. The comparator corpus also does not directly exercise every stated parser/provenance requirement, including duplicate JSON keys and manifest paths, accepted adapter binding, and task/tree replay. The host protocol acknowledges ignored-source/index/config and environment/toolchain gaps but does not supply independent fixtures for them.

Required disposition: prepare the missing finite independent cases or name a bounded prerequisite preparation owner with an independently frozen acceptance contract. Candidate-authored unit tests cannot silently become the sole acceptance authority for the harness implementing these rules. Keep complete application inventories and actual oracle capture as later baseline obligations; they are not required to exist during planning.

## Re-review state

The comparator and host authors received the concrete reproducers and repair requests. A real `tc-proof` or `tc-proof-host` remains a future B-HARNESS deliverable and has not passed qualification.

### First repaired comparator

BR-01 through BR-03 are repaired in comparator driver `c3a646b1759024521ab650051bc76af37bdf54bc8067d21c157c5465053bfda4` and vector file `cd2a9689d47bbad3b0c00ddcff70d47f581cbee4a5103ed6f762bdb4a3a49c67`. The context now supplies protected membership/count, case identities and paths are opaque, order and inert payload salts vary, every result uses strict required fields and integer types, and explicit guards survive Python optimization. The independent `python3 -B -O ... --self-test` run passed 62 fixture constructions, 744 malformed-report rejections, and seven lying runners. The original label-lookup liar was rejected across all 123 invocations. New cases cover duplicate JSON keys, duplicate manifest paths, adapter binding, and task/run replay. This accepts the repaired finite comparator preparation, not a production comparator.

### First protected host observer

The new host observer gate independently passed with observer SHA-256 `913de7d7044cae68cd4e9d229b8bc5d2f0b63df33c6b153f8a897c62d7bd986a`: zero-execution forgery rejected; real build/test/taskfmt launches observed; observer read/write, signal, task-port and network access denied; request replay rejected. This is a Darwin-specific preparation result. The interpreter hash was `6c9d4000c3acc266f080e6abacaef321fb0393778e12889abee8b39c3ed6e0c9`, and `/usr/bin/sandbox-exec` hash was `abc5bb136d6b5cce8fa85d789f78e3326c51ca60cae637b2064adfb67a1dcd9a`. No Linux qualification is claimed.

However, BR-04 remains open for a different confirmed path. The submitted host's `prepare` operation can modify the shared synthetic candidate before `HostFixture.execute()` computes the expected tree and calls `observer.arm()`. An independent revised liar replaces `src/worker.py` with a denial-report generator during `prepare`, then requests genuine observer build/test/taskfmt launches and copies their genuine outputs. The observer executes this substituted probe, which attempts no protected read or write, and the positive fixture accepts it. The successful repro prints `ACCEPTED host-substituted worker that attempted zero protected reads/writes`. The current temporary `host_liar.py` and `run_host_liar.py` reproduce this later bypass; the original zero-execution liar is preserved in the canonical fixture's `forged-host.py`.

The root correction is to freeze all independently authored candidate/probe/package/config inputs before the first submitted-host invocation and forbid or independently reject changes at every command boundary. Computing a new expected tree after a submitted process edits the probe promotes submitted bytes into independent authority. Keep this substitution mutant as a regression case.

BR-05's previously missing secret/other-run snapshots, regular executable requirement, unrelated-ref checks and prior-receipt preservation are repaired. Positive integration still allows arbitrary new ledger/receipt entries without checking their exact authorization, producer/product and tested-tree binding. This requires an exact permitted append contract, not only preservation of existing entries.

BR-06 now has a separate runner protocol and executable fixture preparation for source, expansion, actual direct/PTY execution, test accounting, monotonic contributions and architecture path mutants. An independent `python3 -B -O .../runner-bootstrap-driver.py --self-test` run passed 51 cases, 27 actual observer cases and 204 malformed-result rejections; its external zero-worker liar was rejected. Its private source repository and configuration exist before the first submitted runner invocation, avoiding the host fixture's late-freeze substitution. Driver identity at this check was `604d221865fc7984691ecbe1db6c4f81d87060fe693bce2e8390eac9189624fa`; worker identity `a1ff1fa52f212a53b6c6176a4d76b67a1ad61388a52c5f6d44ad67ad7425f09e`; application fixture identity `682d8e14c480b85175f78748ca25dd163f501a63b9751257007147b679e2fd2c`. Actual Rust source-probe preparation is assigned separately to the planning team, not TASK-072's implementation. Its exact corpus and protocol still need review and a frozen cross-reference; the synthetic suite does not prove production-wide Rust obligations.

The repaired comparator positive frame also round-tripped exactly through the qualified tuisnap executable (`b22ab076a287c910e6bc75238063a22290e06f1079f8b6c3735e683e8e9aefbc`) while running its Python-optimized self-test. Both driver self-test groups remain green.

The observer profile currently begins with `allow default` and denies reads only for named protected roots. A separate temporary synthetic secret outside those roots was readable by a submitted-process probe. No real credential or user-private file was accessed. This profile must not be described as generic credential-free host isolation. Either narrow its read permissions or establish and verify the required credential-free disposable operating environment separately. Named-root protection and production worker-provider qualification are different claims.

### VF-02 aggregate-comparison repair

The earlier single-member comparator preparation did not establish complete aggregate behavior. The repaired protocol now defines an ordered, nonempty, duplicate-free required set of arbitrary size. Successful results contain every identity. Scenario failures retain passing members and continue through later checkpoints; whole-bundle structural rejection has zero checked members. Exact failed identities and categories must match the independently expected set. A reported count cannot substitute for required identities.

The independently reviewed repair has these SHA-256 identities:

- Driver: `3ecb9c6ea3a37da22a864c0c016f6ea58a0ec52973742782d5f73c3e4c7c8dae`.
- Vectors: `84f4b35d92acc39bd5feddfb639920b8484aaff12f9de1f71807612d753046d0`.
- Protocol: `a055cc1d286aac5dbca457bc76015eee62406b9660dd3f4a2d862d85f3e15017`.

The Python-optimized self-test passed 72 vectors, 888 malformed-schema/aggregation probes, nine lying runners and the qualified tuisnap frame round-trip. The external comparator acceptance protocol now requires 72 cases plus 69 positive recoveries, or 141 invocations. No production comparator has performed those invocations successfully.

An additional independent review program, `/tmp/tc-bootstrap-review.slC97f/review_comparator_aggregate.py`, derived expected aggregate results directly from the materialized manifest, frame and semantic-state bytes. It did not call the driver's `expected_outcome` helper or use vector-declared failure indices to construct expected results. It covered the ten multi-member vectors plus an independently introduced two-ID positive fixture. All 62 assertions passed, including the original two-ID/one-result omission pattern, count-only claims, first-only reporting after a failure, duplicate final identities, reordered results, missing failure identities and fabricated extra failures.

The three-checkpoint fixtures actually contain distinct pressed, released and settled states, ordered actions, timer values and cell contents. The four-member fixtures add a separate editor identity; mutations in that final member must appear in the complete result. First/middle/final mismatch, two failures separated by a passing checkpoint, malformed final frame, missing final artifact and dropped final required member are independently distinguishable. This resolves VF-02 for the finite comparator preparation without treating a helper-generated self-test as independent evidence or promoting these fixtures into application parity proof.

### Second host repair review

Host driver `6152b57a89268aa1f5261cab513b4851940d42b4519bb504440aa6d7f7689b4a`, observer `0e95a95c17163d269533a67a062fe3aa900dc75b9e27dae19f1ca28bd5c9e5d1`, vectors `9c86f0535bd0f6b31d4c56497b7631e05a9c3f446b322c6b636e28b019e2f77f`, and protocol `c00f78387d5c39de20d07579f4ff17cfa80bad5205265ad01f6ba36cb9b9fb5b` resolve BR-04's original and late-freeze circular evidence paths. The original substitution reproducer now fails at the exact attempted `src/worker.py` write with `PermissionError`. The candidate and independent probe inputs are kernel-readonly before the first host call, not merely checked by hashes afterward.

The independent optimized observer gate passed all six outcomes: probe substitution denied, zero-execution forgery rejected, real build/test/taskfmt launches observed, process/file/network boundary enforced, ambient secret read denied and replay rejected. It also ran the six real standalone taskfmt cases. The direct Command Line Tools Git executable used by that gate has SHA-256 `be4afb2b003904725826250de9fb76567bbacf82323457b5a1ec26706b66bcae`. The eight host self-tests pass with Python optimization enabled.

An independently authored `/tmp/tc-bootstrap-review.slC97f/review_host_boundaries.py` passed 14 additional probes. It verifies that the private expected tree contains allowed untracked and unstaged payload changes, preserves the approved overlay with a separate scope base, and leaves the original executor index untouched. It accepts the complete context index but rejects a rehashed cross-run child, reordered index, output alias, child symlink, child hardlink, extra child and duplicate JSON key. Separate sandbox calls deny both the original candidate write and a synthetic ambient secret read.

The shared `tc-proof-context-index/v1` now has exact independently constructed child/member binding, immutable index/member files, and verdict linkage. This resolves the host-side INT-03 membership ambiguity at the prepared fixture level; actual runner multi-operation dispatch remains its separate gate. The filesystem-based expected tree resolves the original index-only freeze weakness, but the full future-host positive vector still needs a tracked deletion, in addition to unstaged changes and untracked additions, to complete VF-03's stated coverage.

BR-05's unrestricted append problem is structurally narrowed: receipts cannot be added, unrelated refs stay unchanged, and exactly one task/tree/parent/scope-base/dependency-bound ledger record is permitted. A remaining review finding prevents acceptance of this revision: `run/verdict.json` is still writable during integration, and the expected ledger hash is calculated from that file after integration. This lets a host rewrite the verified verdict and bind the record to its replacement. The verified verdict must become kernel-readonly after its independent checks, its original digest must remain in driver memory, and integration must validate against that retained digest. The host author received this finding and the missing tracked-deletion coverage request.

The proposed write protection needs an additional authority distinction. The independent `/tmp/tc-bootstrap-review.slC97f/review_host_surviving_child.py` probe confirms that a detached child of an earlier host invocation can write a run file after the observer adds that path to the immutable list for future host invocations. Updating a future sandbox profile does not revoke an existing process's rights. The probe touched only a synthetic temporary verdict. Therefore accepted verdict/log/worker bytes must be copied from independently validated observations into the observer's private authority root and bound to the retained digest. External run files remain untrusted copies and must be revalidated against that authority; they cannot define the accepted digest after integration. The host author agreed to this correction and to avoid claiming blanket retroactive kernel revocation.

### Final host preparation acceptance

The final independently reviewed host preparation has these SHA-256 identities:

- Driver: `f1b2c8319824a65f08c5ebef69c164d48f275c44a1d680e8fa3f1ff9870f9c1c`.
- Observer: `0e95a95c17163d269533a67a062fe3aa900dc75b9e27dae19f1ca28bd5c9e5d1`.
- Vectors: `3bfd68251995a3e3115aca081d9789f101beca0f1b1d3c40cecd7ed312d77098`.
- Protocol: `cbc76b1feeddc87abaadb63b8fb72b0f2b6cac97478199dec72c81b4e9f68e10`.

`protect_verified_run` receives the exact verdict bytes read once and independently validated. It retains those bytes and the observer's actual check logs and worker outputs under private `accepted-proof/`. `validate_accepted_copy` rejects drift in either the private authority or public copies. Integration uses the retained `accepted_verdict_sha256`, checks the copies before and after its operation, permits exactly one authorized ledger append, and permits no receipt additions. Public run files are explicitly not retroactively protected from an already surviving child; such a child cannot rewrite the private authority or select a new accepted digest.

Independent normal and Python-optimized observer gates each passed all eight outcomes, including an actual detached old-rights child corrupting the public verdict after sealing. That corruption was rejected, private bytes and digest stayed unchanged, and restoring the publication copy restored the valid case. The direct verdict/log/private-proof write-denial probe also passed. Eight host self-tests pass, including stale-index, overlay-base separation, tracked deletion and seven context-index/member mutations. Each observer gate also ran the six real standalone taskfmt cases with their expected outcomes.

The independent boundary review program now passes 21 probes against these exact bytes. In addition to the previously described context and sandbox attacks, it runs real build, test and taskfmt processes through the observer, seals their actual output, then independently corrupts the public verdict, one check log and one worker output. Every corruption is rejected without changing private authority; every restoration passes. Host writes to the private proof and public sealed verdict are denied. The positive fixture's stale index still tracks `src/obsolete.txt`, while the independently reconstructed tree omits it. The expected and integrated tree assertions also require that deletion to survive. This closes the requested host-side VF-03 deletion gap.

BR-04/BR-04b and BR-05 are therefore resolved for this prepared Darwin observer/host ABI. The shared context-index host surface of INT-03 and filesystem freeze/overlay host surface of VF-03 are accepted at the fixture level. The finite future-host corpus contains 31 negative vectors, each requiring a fresh positive recovery when a real host is supplied. No real `tc-proof-host` exists here, so this review does not report that a submitted production host passed that matrix. Actual runner multi-operation dispatch, actual Rust architecture preparation, complete application baselines and final campaign acceptance retain their separate gates.
