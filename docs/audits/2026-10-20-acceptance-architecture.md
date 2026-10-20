# Bounded architecture acceptance gate, 2026-10-20

## Purpose

`tools/acceptance_architecture.py` checks a narrow, source-bound set of architecture facts independently of `termrock-xtask` ownership exceptions. `tools/acceptance_policy.json` fixes the expected 19 production library package names, four application targets, and prohibited library dependency classes. The checker resolves locked Cargo metadata, rejects missing or duplicate roles, traverses normal and target-specific workspace dependency edges (including renamed dependencies), checks each binary's library dependency closure, and scans Cargo target source paths plus conservative package source inputs. Literal includes and `#[path]` modules are followed to a fixed point; cycles and unreadable/out-of-root inputs fail closed. It flags known coordinate-run frame stamps and ANSI snapshot replay in libraries and every workspace package in production dependency closures.

Its report always sets `acceptance_complete` to `false`. A clean bounded architecture result cannot establish thin application composition, complete workflows, exact real-CLI parity, complete generated-input accounting, or independent semantic review. No exception inventory is loaded. Existing ownership exceptions cannot suppress these findings.

## Evidence

The same checker and policy bytes were used against both source revisions. Reports are retained outside the repository at `/Users/donbeave/Projects/termrock-cli-evidence-20261020/architecture-check-p1-review-pr.json` and `architecture-check-p1-review-design.json`.

| Source | Architecture check | Evidence |
| --- | --- | --- |
| PR worktree revision `c6f1ee62465845459e815a228ee541571ae548c9` plus the uncommitted P1 fix under review | **Red**: 88 captured-frame/replay findings; 0 dependency violations; 0 source read errors or unaccounted inputs | 395 hashed inputs; 19 expected library roles and 4 binary targets found; every application binary reaches a shared library package. Source manifest SHA-256 `56c4039ed316fa89ca705519e3602171940d35a87f7727752b178d4a5cfb2b4c1`. |
| Frozen design source `f4867e9b5f8a08a3bb61ce80d644198a29f51984` | **Red**: 19 production library roles missing; all four application binaries lack a reusable termrock dependency closure | 393 hashed inputs; one dynamic include is explicitly unaccounted; all four application closures are missing. Source manifest SHA-256 `c5ddabe9ae3ef4f1319ef8ab346bd40c2033c8513e98f4cc19a827de625d6569`. |

The policy SHA-256 was `4b7641d76546448660cba8bb2252d488a1c23c5f8818f379ca7f43d23bc47982`; checker SHA-256 was `f420be2f252a6fa9f721b360f87064ad6d5822d5fcb167723ecf0be7fd1d2b7f`. Both reports have `acceptance_complete: false`. The PR's 88 findings break down to 10 frame markers, 43 coordinate-run stamp types, and 35 ANSI frame replays. The design source also has one dynamic include input that the checker cannot enumerate.

Commands:

```sh
python3 tools/acceptance_architecture.py --root . --output /tmp/architecture-report.json
python3 -m unittest discover -s tools -p test_acceptance_architecture.py
```

The regression suite covers the monolithic workspace, missing roles, transitive renamed target-specific dependencies (including helper source scanning), out-of-root source symlinks, renamed/moved coordinate stamps in a library, ignored exception files, ordinary ANSI controls, dynamic generated includes, build-script unresolved inputs, custom library target paths, custom build targets, nested include/path traversal, recursive include cycles, and application dependency closure. Result: 15 tests passed. `py_compile` and `git diff --check` also passed.

## Limits and next checks

This is a lexical and Cargo-graph guard, not proof that a screen is composed from reusable widgets. Source enumeration is conservative, but generated inputs cannot be fully derived from static Cargo metadata; build scripts remain explicitly unresolved. A future aggregator must keep overall acceptance false until full frozen real-CLI workflows, paired frame and functional comparisons, full workspace checks, isolated production-library checks, complete generated-input accounting, and a current independent semantic review all pass. Production stamp detection should expand when review finds new mechanisms; zero lexical findings alone must never be treated as architecture completion.
