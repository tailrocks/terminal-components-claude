# Oracle and provenance

This document defines what may be used as expected output and how it is bound
to the frozen repository. The candidate implementation never owns the
definition of success.

## Authority lanes

Every scenario and checkpoint has exactly one authority lane:

| Lane | Meaning | Acceptance use |
| --- | --- | --- |
| `ExistingOracle` | Approved output already present in the frozen repository's snapshot/test corpus, bound to the pinned source and renderer facts | Required for unchanged baseline behavior |
| `ExtractedOracle` | New trusted capture of an applicable state from unchanged pinned source, made through a reviewed fixture/adapter that exposes state or clock seams only | May close a baseline state gap after independent review and artifact binding |
| `Extension` | New robustness, adapter, theme, API, or capability behavior with no equivalent frozen output | Proved separately; never satisfies an old parity case |

An `ExtractedOracle` adapter may expose fixture data, state setup, geometry,
or deterministic clock entry points. It must not change painting, layout,
event handling, hit testing, fixture results, or product behavior. Record its
patch/diff and digest. Source code and expected output remain outside candidate
authority.

The four applications are preserved consumers in every lane. Their current
fixtures and outputs are visual and interaction references, not permission to
develop new product routes or services.

## Immutable source and tools

- Visual source repository: `donbeave/terminal-components-claude`.
- Exact visual source commit: `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`.
- Annotated tag object observed for `visual-baseline`:
  `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5`.
- Baseline renderer/tool pin: `tuiscotti`
  `a47c9aaefb34e4c00026f99d8a8dd7ee5916b274` (`TUISCOTTI_SHA`).
- Legacy renderer/tool pin (historical provenance): `tui-snap`
  `2d43458ad2bc37d76653c22d56e61ee74512d893`.
- Reference application authority: `7bd6a331721737514a2477c894d922cb262ef07b` (`REFERENCE_APP_SHA`).
- CI generator identity: `velnor-actions-0.1.0` (`VELNOR_GENERATOR_ID`).


Always verify the commit independently with:

```sh
git fetch --tags --prune
test "$(git rev-parse visual-baseline^{commit})" = \
  4a79c0a2d40fca46fc406b77157ce3b3f12ec16b
git cat-file -t visual-baseline^{tag}
git cat-file -p visual-baseline^{tag}
```

The tag, approved snapshots, and frozen application/test sources are immutable
references. A renderer qualification does not move the tag or rewrite the
expected corpus. Product changes and tool changes are separate approvals.

## Artifact authority and write boundaries

The approved grouped store under [`../../baselines/tuiscotti-v1/`](../../baselines/tuiscotti-v1/) is
expected input. The current frozen checkout contains 7,550 artifacts of each
kind (`.ansi`, `.txt`, `.png`, `.html`), 30,200 files total. Scratch actuals,
diffs, and reports belong under `target/` and never become expected output by
test execution.

The frozen PTY driver set is [`../../tests/visual_baseline/`](../../tests/visual_baseline/),
including its audit, pointer, application, and shared-support drivers. The
future harness may read it or use a separately reviewed adapter. It may not
modify it as part of candidate capture.

Expected artifacts, scenario manifests, source/tool digests, comparator
configuration, and approval receipts are materialized by a trusted runner or
reviewer-owned bundle. Candidate output is written to a separate directory.
Reject symlinks, substituted paths, duplicate IDs, unexpected files, stale
source/binary hashes, changed profile/font pins, and any attempt to write an
approval. Filesystem read-only flags alone are not a trust boundary when the
executor owns the files.

The only explicit approval operation belongs to the trusted reference-capture
workflow after review. Never run a candidate-driven `tuisnap accept` or an
environment-controlled bless operation. A candidate cannot create its own
expected baseline and then compare itself successfully.

## Provenance record

For each oracle bundle, scenario, and checkpoint record:

- repository, commit, tag resolution, source paths, and available Git blob IDs;
- authority lane and, for extraction, adapter patch/diff hash;
- Rust/toolchain and lockfile digests, target/OS image, capture-tool commit,
  toolkit revision, comparator version, profile and font/raster digests;
- fixture and scenario-manifest revision/digest, argv, environment, locale,
  timezone, initial dimensions, every resize, and capability/motion mode;
- sealed input program, decoded events, logical time samples, checkpoint IDs;
- canonical frame/semantic observation hashes and all `.ansi`, `.txt`, `.png`,
  and `.html` artifact hashes;
- independent review identity and approval receipt, if the lane is approved.

Pointer descriptions are resolved to absolute numeric events against the
trusted oracle, then sealed. Record both the semantic target description and
the coordinates/bytes actually sent.

## Capture-plan status

The imported [`../reference/capture-plans/`](../reference/capture-plans/)
metadata contains 45 plans and 222 required case descriptions. Every plan
pins the exact baseline commit, uses the five canonical dimensions and five
capability modes, has status `planned_not_captured`, and declares
`expected_artifacts: null`. Its source URLs and snapshot roots are discovery
references, not proof of state coverage. Before implementation acceptance,
each required case must be bound to `ExistingOracle` or a reviewed
`ExtractedOracle`; an `Extension` is recorded separately.

No plan counts as passing merely because its JSON validates. Missing required
capture, missing approval, unavailable fixture, or unavailable tool reports a
blocked/failing case and keeps the denominator intact.

## Known limits

- The source pack is a document specification. Its validation status does not
  claim Rust compilation, implementation, new captures, candidate comparison,
  PTY execution, performance measurement, or independent review.
- The pinned `tui-snap` schema normalizes or drops some terminal attributes,
  including blink and some underline/overline information. Record exactly what
  the version preserves. Prove dropped attributes through direct model,
  protocol, or cursor observations; never call them screenshot-proven.
- Canonical pixel rendering depends on the qualified raster profile and fonts;
  it does not represent every user's terminal/font system.
- Pure component captures do not prove PTY transport, decoding, terminal
  cleanup, or executable routing. PTY captures do not prove hidden state or
  exact component ownership. Both lanes are required where applicable.
- A source directory link, a snapshot filename, or a frame digest alone does
  not establish all-state coverage or artifact integrity.
- Secret values and markers must not appear in cell/text/HTML/PNG artifacts,
  debug traces, logs, provenance, or reports.
