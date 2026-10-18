# Runner bootstrap preparation evidence

This records qualification of the independent preparation files, not acceptance of a production `tc-proof` implementation. No production runner was supplied, no terminal-components production source was changed, and no application baseline was created or blessed. All source commits belong to disposable synthetic Git repositories and use DCO signoff plus the required Codex co-author trailer.

The following combined command passed on 2026-09-11 after the final-review repairs:

```sh
python3 -B -O docs/refactoring-plan/evidence/runner-bootstrap-driver.py --self-test
```

Result: 58 base cases, 34 base protected-observer runtime cases, 232 malformed-result rejections, 14 immutable multi-check index cases, 58 extension observer cases, a five-stage accounting sequence, and rejection of an external executable which emits a complete successful result schema while launching zero workers. Extension execution includes complete qualified prepatch **and actual approved-postpatch** tests before candidate invocation, a register derived from those real postpatch outcomes, approved migration exposing a new oracle assertion failure, actual per-target production corrections with qualified unresolved counts 2 → 1 → 0, native fixed-row assertions plus source-derived resized pointer effects, and privately compiled Rust allocation/work/timing perturbations. Same-name target allowance/closure/owner borrowing, incomplete/duplicated/foreign proposed inventories, absent or forged postpatch evidence, and independently rehashed invalid child schemas/extra fields fail. The optimized-Python run proves rejection checks do not disappear under `-O`; the native worker runs with its original assertions enabled. `rustfmt --edition 2024 --check` also passed for the Rust fixture, whose actual observer compilation uses `-Dwarnings -Copt-level=3`.

The same final bytes also passed the complete command without `-O`, with identical case counts. Whitespace checks on the new source/protocol files emitted no whitespace diagnostics.

The successful preparation run used these SHA-256 identities:

| File | SHA-256 |
| --- | --- |
| `runner-bootstrap-driver.py` | `cb2e11775b155858c386e08c234de4daa66e3800803534c6f9fb86f7d29d18de` |
| `runner-bootstrap-app.py` | `682d8e14c480b85175f78748ca25dd163f501a63b9751257007147b679e2fd2c` |
| `runner-bootstrap-worker.py` | `e5bd488a947bd7fa43d46b9f3658016c3d6a586e60ccfba093aff24d2678198a` |
| `runner-bootstrap-index.py` | `6d1e6c717a396c57835ec12ed308691dc1eb5b710939c2268d1845cbd7704710` |
| `runner-bootstrap-extensions.py` | `fb7bb0c35006fdd7759601866f461c57921f1de2805c6f94c47a1631072d8795` |
| `runner-bootstrap-accounting.py` | `8468412179b55a5abba3c431ed1cc81d4e9955ef9ee0a914cf64dcd3cb7e5b14` |
| `runner-bootstrap-native.py` | `b425dfdba38c89b46db734c277f318cf5784d032624f7bd2f409a526869f7996` |
| `runner-bootstrap-performance.rs` | `7b9c3903da134f478a9d78994ea1a1cb465341e2d208b7d96065d3f4cd188fed` |
| `runner-bootstrap-protocol.md` | `13f16cefe80525f4229d69676a9826ef4098f344f50a9533c46d2c3133b4fb49` |
| `runner-bootstrap-extensions-protocol.md` | `e2c0379ff993ad50707c20c97116073c9d0909c6dcd5f7c95afbec7fecd63c2f` |
| `host-bootstrap-observer.py` | `0e95a95c17163d269533a67a062fe3aa900dc75b9e27dae19f1ca28bd5c9e5d1` |

The imported host observer is separately undergoing independent review. Its current default-deny-read policy was used by the combined run above. Any change to its sandbox behavior requires this preparation check to run again and its new identity to be recorded before freeze. TASK-070, TASK-071 and TASK-072 must later pass their respective actual submitted-executable groups, which automatically include their extension cases. The planning-owned actual-Rust architecture corpus is an additional prerequisite for TASK-072; this synthetic fixture does not replace it. The Rust policy influenced the allocation-free safety wrapper, optimized measurement and warning-denied standalone compilation. Independent review, not this author-authored evidence note, supplies the final acceptance decision.
