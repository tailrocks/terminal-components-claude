# Independent canonical JSON transport rereview

Subsequent actual-compiler repair and all affected normal/optimized reruns are independently completed in [reaudit-compiler-identity-rereview.md](reaudit-compiler-identity-rereview.md). The original byte bindings below remain historical records, not silently replaced results.

Bounded conclusion: the shared type-equality repair rejects the reproduced boolean/integer and float/integer observation substitutions. This is not whole-plan acceptance, compiler-identity qualification, or ADJ-13 qualification. No production changes occurred.

The reviewer fully read the shared runner (593 lines), extensions (523), Rust model driver (336), actual-TC driver (280), source driver (552), source policy (160), and the four independent scripts in `/tmp/source-qualification-rereview.IH2g4A`. Unavailable old sessions 47486/91532 and absent PIDs 29482/44400 supply no result; foundation retained no completion output. Fresh executions below replace those assumptions.

## Root condition and repair

Python structural equality conflates `False`, `0`, and `0.0`, including deeply nested lists/dictionaries. An attacker could preserve private observation digests while changing submitted typed payload values, if the consumer compared ordinary Python objects. `runner-bootstrap-driver.py:418` now compares canonical JSON bytes of submitted observations with the private observer event list; the existing digest check remains separate. Full positive output comparisons at extensions `:404`, model `:246`, actual `:195`, and source `:454` also use canonical bytes. Thus omitted/extra output fields fail, and changing either the output alone or output plus its digest cannot borrow an authentic private event. Reordering JSON object keys or insignificant input whitespace is intentionally normalized; typed values are not.

The remaining ordinary comparisons inspect host-owned event payloads, not a candidate-controlled replacement. Canonical JSON is not a universal external-data schema validator; the scope here is exact equality to independently produced finite observations. The candidate cannot introduce NaN, type-coercion aliases, or nested extras and still match those bytes.

## Fresh execution results

Each command used `python3 -B`, then independently `python3 -B -O`. Every completed result below exited zero in both modes.

| Command | Observed result |
|---|---|
| `runner-bootstrap-driver.py --self-test` | 58 fixture cases; 34 actual observer cases; 232 malformed-result rejections; 14 index cases; 58 extension cases; five accounting stages; zero-worker liar rejected |
| `architecture-bootstrap-driver.py --self-test` | 41 cases: 38 compiled, three compiler rejections, 16 runtime mutants, 15 source-only mutants; exact mutation reversal; zero observer rejected |
| `architecture-bootstrap-actual-driver.py --self-test` | 25 actual prepared cases; two IPC positive/recovery controls; always-pass rejected; zero and forged controls rejected; protected relocated execution |
| `/tmp/source-qualification-rereview.IH2g4A/typed_forgeries.py` | Both `bool-exit` and `float-record-exit` print `TYPED_FORGERY_REJECTED ... forged observation bytes`; no accepted attack. Its exit status alone would not prove rejection, so exact output was inspected |
| `/tmp/source-qualification-rereview.IH2g4A/final_attacks.py` | Three transport positives/recoveries accepted; 13 exact malformed/identity/replay/output attacks rejected. Global over-rejection caught on permitted-only graph; shallow checker caught on 13 targeted dependency mutations. Explicit FINAL_ATTACKS_PASS |

Fresh `architecture-bootstrap-source-driver.py --self-test` normal session 68533/PID25083 and optimized 66532/PID25365 subsequently both completed with exit0:193 source cases (44 external consumers,33 dependency cases,114 source-policy cases,two executable examples),153 negative controls,three positive transports/recoveries,and one exact rejection each for always-pass,zero,forged,bool-exit,and float-record-exit. These are actual final results, not inferred from preparation or active process state. All started reviewer qualification sessions have now completed; no running test is counted as passed.

## Immutable reviewed script bindings

Paths below are relative to `docs/refactoring-plan/evidence/`.

| Path | Lines | SHA-256 |
|---|---:|---|
| runner-bootstrap-driver.py | 593 | 4a481d42161963eb620d271c9fa8474d605c42bebd76f1b206a5e34de31619c4 |
| runner-bootstrap-extensions.py | 523 | e9083cde278a3bfe2d9deada1225b1657c4dda090e2ba2ee0f3a0faba3882ae6 |
| architecture-bootstrap-driver.py | 336 | e40d2b0fc3f8c83b597604fd493c5f767238a4cb8e3d280b40d81a2b12a051d0 |
| architecture-bootstrap-actual-driver.py | 280 | a3e03f8ed0227bbb9a2186bc16536b8b29ca64667f0b0eba69c992b84a232f57 |
| architecture-bootstrap-source-driver.py | 552 | 9084ba14f66493e378407b3abeca497ff1dcd11c82623247bd43d502725cabb8 |
| architecture-bootstrap-source-policy.py | 160 | ec20832058b6db26c589031bd1a332435d85ee95914df568d5dba98c5983585a |

Foundation separately identified selected-Cargo identity drift: `architecture-bootstrap-main-driver.py:166` hashes the PATH cargo shim, caches identity, and `compile_execute` selects PATH again. That is a material tool-binding issue under repair by its owner, not disproved by these transport tests. No toolchain freeze acceptance is claimed here; subsequent helper changes require their own current-byte reruns. Existing source corpus and new ADJ-13 corpus remain distinct. Style measurement/source policy and complete task acceptance are outside this narrow type-binding conclusion.
