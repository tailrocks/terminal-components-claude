# Conformance driver corrections — 2026-10-21

This increment corrects three conformance-fixture mismatches, not product
behavior. The starting source was `5b3097acab788ed4e189bc7e6aab27f1b6a6ee65`;
it was normally merged with `origin/parity` at `79f8990630e129bf35103dd0b3a860dc27eae87f`
before the final focused gate.
The existing before-fix diagnostic is preserved at
`/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/conformance-68-cause-review-52e1572/review.json`
(SHA-256 `4d591a1959b2203664042c225b3991630fb797a534633f54d414005416523732`).
That packet records the original failing probes against source
`52e1572caaf7d9effd0265bbfdf6edcd0e57533b` and must remain historical evidence;
it is not a baseline approval.

The test driver now compares `Theme` values semantically rather than their
fingerprints. `Theme` equality excludes the generated recipe revision while
preserving the resolved patch-effect and sibling-isolation assertions. Dialog
activation explicitly follows its actual route: focus the visible opener, open
the modal, then focus/click the action inside it. The CodeEditor binding audit
uses the real `FOCUSED | EDITING` state, because focusing this editor enters its
immediate editing mode; it no longer evaluates Ctrl+F against the navigation
table while the live editor is editing. Default binding-state samples for all
other cases are unchanged. The local-override check snapshots the exact
runtime's semantic theme before the patched draw and compares that same runtime
afterward, so a local mutation remains detectable.

Focused post-change nextest runs passed: Brand and Completion semantic-theme
checks, Dialog keyboard/mouse activation, and CodeEditor handled-key bindings
(4/4; merged-source run `d023eeef-8885-4215-925e-5ce6d84c30d6`); all
activation equivalence cases (44/44; merged-source run
`991542d1-a1bd-4fc7-a704-7797f4bb873d`); formatting passed with
`cargo fmt --check --all`. A wider binding scan is not green: 13 of the first
18 tests passed and five failed—Dialog dynamic F6 not consumed, HelpOverlay
unable to focus its control, FilterList consuming Ctrl+A outside its declared
bindings, and Form and Field consuming Shift+F10 outside their declared
bindings. Nextest fail-fast left 26 cases unrun. These failures are outside the
changed setup paths; they remain unresolved and no complete conformance-suite
result is claimed. Full output is preserved with the focused results under
[`evidence/2026-10-21-conformance-driver-corrections`](evidence/2026-10-21-conformance-driver-corrections/).
The prior targeted red packet and current unrelated failures remain visible;
no assertions or expected values were suppressed or edited.

The committed post-merge logs are pinned by SHA-256: focused
`3c05f79219ef11ddf7ff6f22b9ce68a461d2cf615bc83e6776617856fd123179`, activation
`158392b55acddd86e67dd1d23c13ca3019c45cb9c1515f5c6debd1eddf1ef21a`, and
broad binding scan
`cf07858cf59d4f88f3a21e4b39e31aa28076dcf016adec7cc9cf5faddee1d52c`.

This evidence does not establish full 68-case conformance, application parity,
or lifecycle acceptance. The independently confirmed Grid and ScrollRegion
rendering failures from the earlier packet remain real and untouched. No
Grid/Scroll source, production application behavior, snapshots, or expected
outputs were changed.

Independent review of exact WIP commit `22366ab816b95ed71be970b4ea541fc332cb8e86`
is CLEAR for this bounded increment, with no new findings. It closed the stale
`K_OPEN` acceptance and semantic-only Theme comparison findings. The reviewer
reran 47 focused native cases (47 pass, 888 unrelated skipped), all six
unchanged external negative controls, strict scoped Clippy and formatting. Its
review packet is retained at
[`independent-review.json`](evidence/2026-10-21-conformance-driver-corrections/independent-review.json),
SHA-256 `3078d53970161597ffdd02bc9012cc77f0c137fb6eba3966795e228d14df429f`.
This clearance is limited to those probes and source; full conformance remains
incomplete.

Integration verification on merge candidate `b64420ffaa39d30a1cc2ee7a4afbf23e9f244617`
reran the unchanged 47-case focused selection (Nextest run
`12b0520f-d1f3-4b45-8fb7-78b02e10d844`: 47 passed, 888 unrelated skipped) and
the six external negative controls (run `0888a2bc-f5d4-4fd9-a22b-864ff98f8e47`:
6 passed). The focused test binary SHA-256 is
`2c4d1c474f9e4e7c79b6dfc7fcc4360025f2b2546325e78e6edbd52b982e7674`; the
negative-control binary SHA-256 is
`59e700e416a95188baa0a757cf5fa4c8112156d25493c3e8bfeacb2db4854103`.
`cargo clippy -p termrock-test-support --lib -- -D warnings`,
`cargo fmt --all --check`, and `git diff --check` passed. Seven protected
expectation/assertion files match the bytes at `52e1572caaf7d9effd0265bbfdf6edcd0e57533b`;
their paths and SHA-256 values are retained in the integration work evidence.
These checks apply only to this merged source and do not clear the broader
conformance failures, terminal/PTY coverage gaps, or application acceptance.

The branch advanced with the independently integrated Form pointer-reveal
change. After normal-merging PR #38 tip `1b9bd580e5f46dff23630a8c15c85e7f14cf7c4e`,
candidate `c29b2f7f547776769cc581e0456746b9e5dbd7c2` again passed the focused
47-case selection (Nextest run `fca1057f-cdab-4eea-b092-4fcb03f9dfdc`, 47
passed, 888 unrelated skipped) and the six external negative controls (run
`7eebfbac-7cce-4912-94c4-f66770c1de57`, 6 passed). Strict scoped Clippy,
workspace formatting, and whitespace checks passed. All seven held
expectation/assertion files still match `52e1572caaf7d9effd0265bbfdf6edcd0e57533b`;
their SHA-256 values are listed in the prior verification paragraph. This
revalidation supersedes the immediately preceding `b64420f` local candidate
receipt without changing the bounded review scope.
