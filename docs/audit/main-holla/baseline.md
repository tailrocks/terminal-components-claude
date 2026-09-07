# Independent baseline and gate audit

Pinned reference: `794b095c196562d38f1b6f7ce379c128af2a023d`. Reference source never edited; isolated target-holla. Commands and raw logs: holla-commands.json. Test identity inventory: holla-test-identities.json. Initial candidate artifact hash inventory: artifact-inventory.json (before integration changes).

## Verified findings

- Historical recipes: 499 unique; Showcase 208, TablePro 108, Jackin 183. No Holla namespace. All 499 each TXT/ANSI/cursor exist; all 499 each HTML/PNG absent. All current trace/provenance absent. Exact missing filenames: recipe-availability.json.
- No reachable Git log entries for baseline/before PNG/HTML; manifest first introduced at cefc4b8af27226d6b6494b49dd5224ba2bd48f6f. Manifest header says generated at 0cd1bf7; scope says binaries at d5e7075. Neither source tree contains baseline captures. Regeneration requires resolving source provenance, separate immutable worktree, reconstructed renderer/environment; cannot claim new images are recovered originals.
- perf.yml lacks explicit shell, falsely claims implicit shell enables pipefail. Reproducer `bash -e -c 'false | tee /dev/null'` returns 0; explicit `bash --noprofile --norc -eo pipefail -c ...` returns 1. Set workflow defaults.run.shell: bash or each step shell: bash, then retain mutation test.
- CI uses unlocked Cargo commands; identities only checked by nonzero render-prefix grep, which cannot detect missing cases. Enumerate targets and per-target identities in an explicit required inventory; compare exact set before running. Do not equate a zero-match filter with a test.
- Existing parity gate has useful strict fingerprints/artifact hashes/review binding. Preserve these. `xtask/src/parity.rs` hardcodes 499 historical recipes and maps only showcase/tablepro/jackin; extend distinct pinned-holla namespace, do not alter historical count or relabel historical captures.
- Historical baseline_capture.sh uses fixed sleeps and `target/debug` binaries; source provenance must not assume caller's working directory proves executable identity. Existing newer capture_provenance.py is reusable and checks hashes; qualify it with stale-binary/capture-failure mutations rather than replace blindly.

## Tool qualification plan (not executed; no installation)

1. Inspect current donbeave/tui-snap and microsoft/tui-test primary repositories and actual manifests/API, pin full source SHA plus lockfiles/toolchains externally. Build modern tooling outside Rust 1.88 workspace and retain executable hashes.
2. Reuse existing production-view Rust harness for canonical cell/cursor output. Feed a known ANSI fixture through each engine: default/truecolor/256/16/mono, DIM/bold/reverse combinations, blank colored cells, wide and combining Unicode, clipping at both edges, cursor positions/visibility.
3. Compare exact expected cells/styles/widths/cursor against tool output; document unsupported attributes as tool limits, never normalize away differences. Independently inspect rendered image with pinned font/profile.
4. Use a controlled executable fixture to echo keyboard/paste/pointer/wheel/drag/resize events and expose ready/settled markers. Prove zero/one-based pointer mapping, frame settling, shutdown restoration, exit statuses, bounded timeouts, isolated session names and failure artifacts.
5. Run deliberate mutations (one-cell hitbox, dropped DIM, wrong binary hash, missing reference, stale output after failure, duplicate/missing recipe, empty filter) and require each qualification gate fail before tools consume product snapshots.
6. Store separate historical, pinned-holla reference, and candidate roots. Manifest includes source SHA, executable SHA, argv/env, viewport, motion/time, color/theme, engine/font versions, hashes for canonical/PNG/trace and reviewer identity. Candidate snapshots never become reference automatically.
