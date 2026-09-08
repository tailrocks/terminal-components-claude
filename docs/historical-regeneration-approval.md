# Reviewed historical rendering installation

Accepted scope: add the previously absent 499 HTML and 499 PNG historical
renderings. This implements the user's main-based integration task, the execution
plan's historical recovery work, and the independent review below. Architecture
§75 narrows the prior blanket frozen-addition refusal for these exact bytes only.
No baseline key was blessed and no application acceptance snapshot was approved.

## Immutable authority and classification

- Generator: `c051c8fa61c2fd67eb06e122dd91fd2553c8c810`.
- Source-equivalence provenance: `040a2726c6f8d48590bc5b787afae2a0839096a0`.
- Archive inputs: `c12cad8728755cd2d03eefdd8e02891143fca86d`.
- Historical renderer/application source: `d5e7075f436f0e437c7d12cf3d1e638e763b26f6`.
- Exact manifest: [pins.json](../tools/historical-render/pins.json), SHA-256
  `e2dcdb2952e5cbd283b841ce055962c4d961dfa967d43c879bdbc49a5abdb6cb`.
- Independent [installation review](historical-regeneration/independent-review.md),
  SHA-256 `c3898d76eea11f22051117788b5d6872d568227ce5619ddf63eaa91f0d9031dd`.
- Generated [provenance](historical-regeneration/PROVENANCE.json), SHA-256
  `001f4cf50addaff6c59a3500416c0b681904f7db0f1fd9436990c397746e1dfc`.
- Preserved [font license](historical-regeneration/FONT-OFL.txt), SHA-256
  `30f0c136e3c88e422d0791acd97238870f9054a9729bc34cf2ff0d4ed8cac4ad`.

The files are **regenerated-historical-renderings-not-originals**. The original
image bytes and executable hash remain unavailable; the latter remains null.
Independent review exhaustively checked all output hashes and visually inspected
three capture pairs. Historical Unicode/glyph and modifier limitations remain;
HTML font resolution depends on the browser environment. This installation makes
the historical inventory complete, not current applications visually correct.

## Binding gate

The historical-addition module is called by the existing baseline classification
gate. It verifies the pinned generator manifest, local manifest, independent
review, font license and generation provenance; requires a comparison base descended from the immutable archive; checks
all 1,497 original ANSI/text/cursor inputs and all 998 regenerated outputs; rejects
unreviewed HTML/PNG files, including ignored files; and grants an exception only
for a reviewed path absent from the base Git tree. Empty existing blobs still
count as existing. Every changed existing frozen artifact remains refused by the
original guard. Missing and unreadable artifacts fail rather than disappearing
from the discovered changed set.

Use an explicit comparison base and external target directory:

```sh
rtk proxy env CARGO_TARGET_DIR=/tmp/historical-xtask-target cargo +1.88.0 test --locked -p xtask historical_additions
rtk proxy env CARGO_TARGET_DIR=/tmp/historical-xtask-target BLESS_GUARD_BASE=1318e85 cargo +1.88.0 run --locked -p xtask -- bless-guard
rtk proxy env CARGO_TARGET_DIR=/tmp/historical-xtask-target cargo +1.88.0 run --locked -p xtask -- parity --dry-run
```

Reproduction remains the pinned command in `tools/historical-render/README.md`.
The generator refuses existing destinations and does not install baselines.
Installation copied only manifest-listed, independently hash-verified absent
HTML/PNG files and staged them explicitly with `git add -f`; no pre-existing
baseline file changed. License and provenance remain outside `baseline/before`.

## Installation verification

[The recorded gate ledger](historical-regeneration/installation-proof/gates.json)
binds 98 passing xtask tests, the real bless guard, the 499-recipe parity inventory,
strict xtask clippy and formatting of changed Rust sources. Rust 1.88.0 executes
the tests and binary; installed Rust 1.98.1 provides clippy/rustfmt because the
1.88.0 rustfmt component is unavailable. The CI matcher repair was independently
supplied by the integrator; it preserves the CI base assertions.

[Mutation results](historical-regeneration/installation-proof/mutation-results.json)
record twelve rejected mutations: malformed/tampered manifest, altered output,
altered original ANSI, altered review/provenance, missing output, root and nested
unreviewed ignored PNG, symlink output, invalid base and unrelated base. The exact
installation passed before and after restoration. The full current parity command
still fails for missing current acceptance evidence, as required.

[The installation inventory](historical-regeneration/installation-proof/installation.json)
records 998 exact additions against the immutable archive and zero modified or
deleted original artifacts. All published proof files have SHA-256 bindings in
the accompanying `SHA256SUMS`. Local paths in logs are historical execution
evidence, not reproduction dependencies.
