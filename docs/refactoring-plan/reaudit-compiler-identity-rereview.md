# Actual compiler identity — independent rereview

Bounded planning-evidence review, not whole-plan acceptance. The previous JSON transport report's completed runs used the old compiler-selection boundary; this report binds the subsequent repair. Read the new resolver, cached record, revalidation, environment, actual build path, source command wrapper and all 109 lines of compiler-identity-selftest.py before executing checks.

The enabling defect was recording a PATH launcher while separately invoking a selected compiler, compounded by cached identity without revalidation. The repaired helper resolves actual Cargo and rustc through one explicit rustup toolchain before entering source copies; stores resolved path, bytes, verbose version and toolchain; revalidates all four before and after compilation; pins RUSTC and clears both wrapper variables. Source-library/parser builds, metadata and external rustc commands use that same boundary. Candidate execution remains protected and cannot rewrite compiler records or privately observed outputs.

Independent normal and optimized compiler-identity-selftest runs passed all 21 controls, including actual tool execution, altered records/bytes/path/toolchain/version, defensive-copy isolation and post-invocation tamper rejection. A separate dependency-free real build in `/tmp/compiler-independent.NLJYsc` put `rust-toolchain.toml` at 1.88.0 and supplied `/usr/bin/false` for RUSTC, RUSTC_WRAPPER and RUSTC_WORKSPACE_WRAPPER. Both modes built/ran the actual tiny Rust executable successfully under the frozen host 1.98.0 compiler; all three poisoned variables were replaced, and identity still validated. This tests actual invocation, not just a mocked passing metric.

Current affected reruns completed in both modes: actual-TC 25 prepared cases, two positive/recovery executions, always-pass and zero/forged rejection; typed boolean and floating-point forgeries rejected with `forged observation bytes`; final transport attack corpus accepted three legitimate results and rejected all 13 malformed/forged paths, rejected global overreach on the permitted-only graph and detected all 13 shallow-graph counterexamples. Broker corpus separately completed 45 actual AST premises plus ten transport controls in both modes, explicitly not a submitted-checker acceptance.

Source 193-case normal session94550 and optimized session14066 both completed exit0 on these current hashes: 44 external-consumer, 33 Cargo-dependency, 114 source-policy and two executable-example cases; 153 negatives, three positive executions and individual always-pass/zero/forged/boolean-exit/floating-record-exit rejections. No affected run remains unfinished. Earlier successful source runs retain their old-boundary status only.

| Binding | SHA-256 |
|---|---|
| evidence/architecture-bootstrap-main-driver.py (484 lines) | fd6e1f240cd09367b8414041302fa926d6d939502577d2ff8f3269c9cde1cd49 |
| evidence/architecture-bootstrap-source-driver.py (558 lines) | b1a74b78f6a13d0ba2430e016aa25471314a31301b814adbec878428a38f9ed6 |
| evidence/compiler-identity-selftest.py (109 lines) | 034808f8046d80149d1571ef5831affddf47176dc871944628702d722dd0de28 |
| actual Cargo 1.98.0, commit797e8a9bc | 1de2e84c15443b70444eecfa959ff9099dd8c1a5606b6d9ef5bc0ea9c25bc7f9 |
| actual rustc 1.98.0, commit88d9e12ae | a11618eca0956a8aa4372c2bc898690b513cbdfa2cb9125b2a5301e360ed5b49 |

Both tools reside in `/Users/donbeave/.rustup/toolchains/1.98.0-aarch64-apple-darwin/bin/`; the MBX shell shim is no longer credited as the compiler. No production source or baseline changed. Immutable host tool storage/isolation remains a separate prerequisite: pre/post hashing does not claim resistance to a hostile privileged actor replacing and restoring a compiler between reads.
