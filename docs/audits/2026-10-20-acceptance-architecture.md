# Bounded architecture acceptance gate, 2026-10-20

## Purpose

`tools/acceptance_architecture.py` checks a narrow, source-bound set of architecture facts independently of `termrock-xtask` ownership exceptions. `tools/acceptance_policy.json` fixes the expected 19 production library package names, four application targets, and prohibited library dependency classes. The checker resolves locked Cargo metadata, rejects missing or duplicate roles, traverses normal and target-specific workspace dependency edges (including renamed dependencies), checks each binary's library dependency closure, hashes tracked and present source inputs, follows literal includes and `#[path]` modules, and flags known coordinate-run frame stamps and ANSI snapshot replay in both libraries and application packages.

Its report always sets `acceptance_complete` to `false`. A clean bounded architecture result cannot establish thin application composition, complete workflows, exact real-CLI parity, complete generated-input accounting, or independent semantic review. No exception inventory is loaded. Existing ownership exceptions cannot suppress these findings.

## Evidence

The same checker and policy bytes were used against both source revisions. Reports are retained outside the repository at `/Users/donbeave/Projects/termrock-cli-evidence-20261020/architecture-check-pr-final.json` and `architecture-check-design-final.json`.

| Source | Architecture check | Evidence |
| --- | --- | --- |
| PR branch source `08d88b8059a0e319962fe3e282e2e10b52371d9e` | **Red**: 88 captured-frame/replay findings; 0 dependency violations; 0 source read errors or unaccounted inputs | 395 hashed inputs; 19 expected library roles and 4 binary targets found; every application binary reaches a shared library package. Source manifest SHA-256 `6411220da0c9c4affbb34deaf8fb661a9ff1dc0e9314200018381806c7bc70bb`. |
| Design source `f4867e9b5f8a08a3bb61ce80d644198a29f51984` | **Red**: 19 production library roles missing; all four application binaries lack a reusable termrock dependency closure | 1,482 hashed inputs. Source manifest SHA-256 `4485f73b73f8a17a363bd31b782c55cb9927a7cc8415bb4c4a3080de19285c68`. |

The policy SHA-256 was `4b7641d76546448660cba8bb2252d488a1c23c5f8818f379ca7f43d23bc47982`; checker SHA-256 was `891243dc4d42729c9cba90fbe61bbaf8895cd1a88d79d4fbb168cd84df0aa158`. Both reports have `acceptance_complete: false`. The PR's 88 findings break down to 10 frame markers, 43 coordinate-run stamp types, and 35 ANSI frame replays. The design source also has one dynamic include input that the checker cannot enumerate.

Commands:

```sh
python3 tools/acceptance_architecture.py --root . --output /tmp/architecture-report.json
python3 -m unittest discover -s tools -p test_acceptance_architecture.py
```

The regression suite covers the monolithic workspace, missing roles, transitive renamed target-specific application dependencies, out-of-root source symlinks, renamed/moved coordinate stamps in a library, ignored exception files, ordinary ANSI controls, dynamic generated includes, build-script unresolved inputs, and application dependency closure. Result: 11 tests passed.

## Limits and next checks

This is a lexical and Cargo-graph guard, not proof that a screen is composed from reusable widgets. A future aggregator must keep overall acceptance false until full frozen real-CLI workflows, paired frame and functional comparisons, full workspace checks, isolated production-library checks, complete generated-input accounting, and a current independent semantic review all pass. Production stamp detection should expand when review finds new mechanisms; zero lexical findings alone must never be treated as architecture completion.
