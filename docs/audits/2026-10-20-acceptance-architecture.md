# Bounded architecture acceptance gate, 2026-10-20

## Purpose

`tools/acceptance_architecture.py` checks a narrow, source-bound set of architecture facts independently of `termrock-xtask` ownership exceptions. `tools/acceptance_policy.json` fixes the expected 19 production library package names, four application targets, and prohibited library dependency classes. The checker resolves locked Cargo metadata, rejects missing or duplicate roles, traverses normal and target-specific workspace dependency edges (including renamed dependencies), checks each binary's library dependency closure, and scans Cargo target source paths plus conservative package source inputs. Literal includes and `#[path]` modules are followed to a fixed point; cycles and unreadable/out-of-root inputs fail closed. It flags known coordinate-run frame stamps and ANSI snapshot replay in libraries and every workspace package in production dependency closures.

Its report always sets `acceptance_complete` to `false`. A clean bounded architecture result cannot establish thin application composition, complete workflows, exact real-CLI parity, complete generated-input accounting, or independent semantic review. No exception inventory is loaded. Existing ownership exceptions cannot suppress these findings.

## Evidence

The same checker and policy bytes were used against both source revisions. Reports are retained outside the repository at `/Users/donbeave/Projects/termrock-cli-evidence-20261020/architecture-check-module-path-pr.json` and `architecture-check-module-path-design.json`.

| Source | Architecture check | Evidence |
| --- | --- | --- |
| PR branch source `eebcbadccb183b43e314e21e6dcca3750e342539` | **Red**: 89 captured-frame/replay findings; 0 dependency violations; 0 source read errors or unaccounted inputs | 542 hashed inputs; 19 expected library roles and 4 binary targets found; every application binary reaches a shared library package. Source manifest SHA-256 `976d5bb450522a402755b989875bec182fe91461e5cf16c87f676b07a771071c`. |
| Frozen design source `631c8f33a5c97a4b622e855874493a4107ac3c04` | **Red**: 19 production library roles missing; all four application binaries lack a reusable termrock dependency closure; 4 frame/replay findings | 425 hashed inputs; one dynamic include is unaccounted and one symlinked target path is unreadable/out-of-root; all four application closures are missing. Source manifest SHA-256 `ea257932f8f5b638f301b08b98fac8c6e0a934e632d382e8215780b89ea1ff71`. |

The policy SHA-256 was `4b7641d76546448660cba8bb2252d488a1c23c5f8818f379ca7f43d23bc47982`; checker SHA-256 was `1f7a1089e688baaab67b00bb8a7125e4b67e43f8ffa003e29cac47b19868ba98`. Both reports have `acceptance_complete: false`. The PR's 89 findings break down to 10 frame markers, 44 coordinate-run stamp types, and 35 ANSI frame replays. The design source also has one dynamic include input that the checker cannot enumerate.

Commands:

```sh
python3 tools/acceptance_architecture.py --root . --output /tmp/architecture-report.json
python3 -m unittest discover -s tools -p test_acceptance_architecture.py
```

The regression suite covers the monolithic workspace, missing roles, transitive renamed target-specific dependencies (including helper source scanning), out-of-root source symlinks, in-root and out-of-root directory symlinks and cycles, renamed/moved coordinate stamps in a library, ignored exception files, ordinary ANSI controls, dynamic generated includes, build-script unresolved inputs, custom library target paths, explicit and untracked modules beneath `tests`, actual Cargo target-directory pruning, custom build targets, nested include/path traversal, recursive include cycles, and application dependency closure. Result: 21 tests passed. `py_compile` and `git diff --check` also passed.

## Limits and next checks

This is a lexical and Cargo-graph guard, not proof that a screen is composed from reusable widgets. To catch source modules beneath names such as `tests`, `examples`, or `benches`, it conservatively scans Rust files throughout each production package except the resolved Cargo target directory. This can produce findings in non-production test/example code; that is an intentional false-positive risk rather than a claim of semantic module ownership. Literal includes and `#[path]` are followed, while a full Rust module parser is out of scope. Generated inputs cannot be fully derived from static Cargo metadata; build scripts remain explicitly unresolved. Directory symlinks are followed inside the workspace, rejected when they escape, and reported as errors on cycles or unreadable targets. A future aggregator must keep overall acceptance false until full frozen real-CLI workflows, paired frame and functional comparisons, full workspace checks, isolated production-library checks, complete generated-input accounting, and a current independent semantic review all pass. Production stamp detection should expand when review finds new mechanisms; zero lexical findings alone must never be treated as architecture completion.
