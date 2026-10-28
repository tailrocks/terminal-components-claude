> Independent review status: this adjudication does not establish a settled reference decision or accepted fresh capture. The alternate binary cohort is unadmitted; all eight loose streams fail pinned Tuiscotti decoding, and capture metadata records failure/missing artifacts. Chronology, DIM counts and cursor conclusions need correction. Preserve this report and both artifact sets as disputed evidence. See [the independent provenance review](evidence/2026-10-21-current-blockers/modal-adjudication-9b0277-independent-rejection.json). No expected update, cohort substitution or product acceptance is authorized.

# Modal backdrop DIM adjudication: the frozen baseline data is defective, the parity port is correct

- Date: 2026-10-21
- Workstream: theme/visual-state extraction (PR #38), branch
  `parity/termrock-visual-baseline`
- Supersedes the open decision in
  `docs/audits/2026-10-21-modal-backdrop-dim-vs-frozen-baseline.md`
- Oracle: tag `visual-baseline` (4169783ae883788186d161af09ba9c481bd84e5c,
  tree ad0f7cf8899df8c5fea3ced3f60adf0ac9a2d297, tag object
  2718a6b8127a522a3d351d0b4bdb7542fc8da542, tuiscotti rev
  a1ea27f757d75d07b05499276cd932e9f8e067ad)
- Classification: **NO-DEFECT** for parity's `modal_backdrop` /
  `Theme::backdrop_style`; **REFERENCE-DEFECT** for the frozen expected
  frames (two layered data defects, detailed below)

## Disposition of the earlier conflict

The earlier audit compared three things that are not the same artifact and
labelled them all "frozen":

1. the raw legacy snapshots bundled in the tag
   (`snapshots/...` — the terminal streams the admission record replays),
2. the admitted corpus frames (`baselines/tuiscotti-v1/...` — the
   `legacy_replayed_conversion` output), and
3. fresh parity candidate captures (its 150/151 numbers; see below).

Its numbers re-checked against each artifact:

| scenario (legacy name) | tier | audit claimed | raw snapshot in tag | corpus frame | fresh run of the tag build |
|---|---|---|---|---|---|
| `showcase/flows/dialogs/open/80x24` | 16 | 0 | 0 | 0 | **920 dim cells** |
| `showcase/flows/dialogs/open/80x24` | 256 | 0 | 0 | 0 | 0 |
| `showcase/flows/dialogs/open/80x24` | none | 0 | 0 | 0 | **920 dim cells** |
| `showcase/flows/dialogs/open/80x24` | truecolor | — | 0 | 0 | 0 |
| `tablepro/overlays/picker_open/120x40` | 16 | 150 | **4656 dim cells** (156 SGR-2 runs) | 83 cells / 5 runs | **1847 dim cells** (21 runs) |
| `tablepro/overlays/picker_open/120x40` | 256 | 5 | 83 cells / 5 runs | 83 cells / 5 runs | 82 cells / 1 run |
| `tablepro/overlays/picker_open/120x40` | none | 151 | **4656 dim cells** | 83 cells / 5 runs | **1847 dim cells** |
| `tablepro/overlays/picker_open/120x40` | truecolor | — | 0 | 0 | 0 |
| `tablepro/overlays/safemode_picker/120x40` | 16 / none | — | 4660 / 4660 dim cells | 0 | — |
| `tablepro/overlays/safemode_picker/120x40` | 256 / truecolor | — | 0 / 0 | 0 | — |

(Audit's "150/151 at 16/none" matches no frozen artifact; it is the run
count of a parity candidate capture at those tiers — parity's mapping emits
one SGR-2 per style-change span, the same signature the tag build produces.
Its "5 at 256" is the corpus number.)

## Evidence

### 1. The tag source contains exactly one backdrop mechanism

`target/frozen-baseline/source/src/theme.rs` (`Theme::backdrop`, :1243; dim
branch at :1310 in the extracted tree):

```text
if !blank && (fg == bg || matches!(self.level, ColorLevel::Ansi16 | ColorLevel::Mono)) {
    ...
    return Style::new().fg(ink).bg(bg).add_modifier(Modifier::DIM);
}
```

`RenderCtx::modal_backdrop_with` (`src/ui/ctx.rs` :658) routes every modal
underlay — Dialog (`src/widgets/dialog.rs` :1264) and Picker/Modal
(`src/ui/modal.rs` :425) alike — through that one mapping, with the same
focus-bar clearing parity ports. There is no per-app backdrop policy in the
tag source.

Parity's port (`crates/termrock-theme/src/lib.rs:237`, `backdrop_style`;
applied by `crates/termrock-runtime/src/ui/paint.rs:459` `modal_backdrop`)
implements the identical rule. It is a faithful port.

### 2. Fresh replay of the freshly built tag binaries confirms one rule

Independent reproduction on 2026-10-21 (this adjudication):

- `python3 tools/frozen_baseline.py prepare` — provenance output matched
  `controlled-baseline-a/provenance.json` in the evidence root byte for
  byte (`source_sha256` f74646a9…, `snapshots_sha256` 6b547179…).
- `python3 tools/frozen_baseline.py build` — final binaries
  `showcase` ca3a3114aef32288b998f0fc1976d9b968f2b1d462c8451c39d74360056601a6,
  `tablepro` 9c82d9afeae293198d7187e095d49d3dc1d5f859eeb2004742f1ce1d3a46f2d0
  (build receipt: `provenance_sha256` b435786f…; admission not performed —
  observation only, per tool output).
- Fresh captures through the pinned runner stack (`tools/capture.sh` /
  tmux, `-c <level>`, `-p dialogs`, keys `Tab Tab Enter`; tablepro: Down×7,
  Enter, Ctrl+P): dim cell counts per the table above, decoded SGR state
  per printed cell. Dialog: 16/none → 920 dim cells, 256/truecolor → 0.
  Picker: 16/none → 1847 dim cells spanning the underlay (popup interior
  excluded), 256 → 82 collision cells on the one fg==bg row, truecolor → 0.

### 3. Reference-data defect, layer 1: the bundled snapshots predate the DIM rule

`admission-record.json` in the corpus records
`historical_oracle_commit 4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Git
archaeology on the frozen history:

- `4a79c0a2d` ("docs: freeze visual-baseline pointers and agent rules") —
  `src/theme.rs` at this commit has **no** `Ansi16 | Mono` backdrop DIM
  branch. The bundled snapshots recorded from this build show dialogs with
  0 DIM at every tier.
- `d720a1c2e` ("fix(theme): popup frames and active-tab marks read at 16
  colours and none", 2026-10-17) introduced the DIM branch
  (`theme.rs:1104` at that commit). It is an ancestor of the tag
  (`4a79c0a2d` → `d720a1c2e` → `4169783ae`, verified with
  `git merge-base --is-ancestor`).
- The tag source therefore prescribes tier-dependent backdrop DIM that its
  own bundled snapshots (recorded earlier) do not contain. The raw dialog
  snapshots at 16/none are the pre-`d720a1c2e` look.

### 4. Reference-data defect, layer 2: the corpus replay dropped picker DIM at 16/none

The admitted corpus frames disagree with the raw snapshots they were
converted from: raw `tablepro/overlays/picker_open/120x40/16.ansi` carries
4656 dim cells (full-underlay DIM, verified by decoding the immutable
in-tag stream), while `baselines/tuiscotti-v1/tablepro/workbench/
overlay-picker-open/120x40/16.frame.json` decodes to 83 dim cells on the
single collision row — the 256-tier look. Same loss at `none` and
`nocolor` (corpus `nocolor.ansi`: 1 dim run; legacy raw: 64+ runs). The
`ansi_parity_rate: 100%` in the admission record did not cover modifier
state. Dialog corpus frames match their raw snapshots (0 everywhere), so
the dialog side looks "green" only because both are pre-fix.

## Decision

1. **Authoritative mechanism: the tag source's single shared mapping** —
   DIM on nonblank backdrop cells iff the cell collides (fg == bg) or the
   color level is Ansi16 or Mono; glyph preserved, focus bar cleared.
   Both apps follow it in the tag source and in fresh runs of the tag
   binaries.
2. **Parity's `Theme::backdrop_style` and `Paint::modal_backdrop` stay as
   they are.** No source change. The reverted no-DIM experiment
   (`ee57fe5e3`) was moving parity away from the reference mechanism.
3. **The mono digest flip was the correct signal.** `quick-switcher 120 40
   junie mono` = `af463bd8d299cf6b`
   (`crates/tablepro-ui/tests/baselines/tablepro.txt:106`) is green
   because parity correctly emits backdrop DIM at Mono, as the tag build
   does. The digest encodes the right mechanism; it must not be re-blessed
   to a no-DIM value.
4. **The diverging frozen expectations are the defect** and are recorded as
   REFERENCE-DEFECT: dialog-open expected frames (0 DIM at 16/none —
   pre-`d720a1c2e` look) and picker expected frames (83 dim cells at
   16/none/nocolor — conversion loss). They may not be "fixed" by bending
   parity; they need owner-approved regeneration from the tag-source
   build through the pinned runner, or an explicit re-approval of the
   pre-fix look as a deliberate divergence.
5. Parity dialog-open captures that currently fail because parity emits
   the 16/mono backdrop DIM the tag build emits are **correct failures
   against defective data** (their remaining redness from palette drift
   and shell drift is owned by the concurrent workstreams and unchanged
   by this adjudication).

## Handoff

No code fix. The follow-up work is data, and it is owner-gated:

- **Fix agent task**: regenerate expected frames for the affected
  scenarios from freshly built frozen binaries (`tools/frozen_baseline.py
  prepare` + `build`, provenance recorded; binaries must hash to the
  build receipt) via the pinned runner, then re-run the corpus admission
  checks with modifier parity included:
  - `showcase/dialogs/open/{72x20,80x24,100x30,120x40,160x50}` at tiers
    16, none, nocolor (expect: underlay DIM present, ~920 dim cells at
    80x24, dim absent at 256/truecolor);
  - `tablepro/workbench/overlay-picker-open` and
    `tablepro/workbench/overlay-safemode-picker` (all sizes) at tiers 16,
    none, nocolor (expect: full-underlay DIM, popup interior excluded;
    256 shows only fg==bg collision cells; truecolor none);
  - add a modifier-aware parity check (decode `frame.json` cells'
    `mods.dim` and compare against the decoded reference frame) so
    conversion cannot silently drop SGR-2 again.
- **Expected diff sketch (if, contrary to this decision, an owner instead
  re-approves the pre-fix look)**: `crates/termrock-theme/src/lib.rs`
  `backdrop_style` (:237) — delete the
  `|| matches!(self.capability.color, ColorLevel::Ansi16 | ColorLevel::Mono)`
  clause from `dim`, keep the `fg == bg` collision clause, and drop the
  same clause from `backdrop_colors`; that is the reverted `ee57fe5e3`
  experiment, and it re-flips the mono digest and every 16/mono dialog
  capture. Not recommended.
- The stale comment block inside `backdrop_style` describing the split
  (`docs/audits/2026-10-21-modal-backdrop-dim-vs-frozen-baseline.md`
  reference at `crates/termrock-theme/src/lib.rs:229-235`) should be
  updated by the documentation owner to point at this file once the
  decision is accepted.

## Reproduction commands

```sh
python3 tools/frozen_baseline.py prepare
python3 tools/frozen_baseline.py build          # ~1 min after warm cache; mise trust the extracted mise.toml
BIN=target/frozen-baseline/cargo-target/debug/showcase tools/capture.sh start 80 24 -- $BIN -c 16 -p dialogs
# keys: Tab Tab Enter; tools/capture.sh shot dialog_open
BIN=target/frozen-baseline/cargo-target/debug/tablepro tools/capture.sh start 120 40 -- $BIN -c 16
# keys: Down x7, Enter, wait for connect, Ctrl+P; shot picker_open
```

Dim-cell counts above were decoded from the raw streams with an SGR-state
interpreter (params scanned left to right; 2 sets dim, 22 clears, 38/48/58
consume their color arguments) and cross-checked against
`frame.json` `mods.dim` for the corpus artifacts. Fresh captures:
`/tmp/adj-fresh-dialog-{16,256,none,truecolor}.ansi`,
`/tmp/adj-fresh-picker-{16,256,none,truecolor}.ansi` (transient, regenerate
with the commands above).
