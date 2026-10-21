# Backdrop expected-frame regeneration: dialogs 80x24 and the tablepro overlay pickers

- Date: 2026-10-21
- Workstream: theme/visual-state extraction (PR #38), branch
  `parity/termrock-visual-baseline`
- Follows: `docs/audits/2026-10-21-modal-backdrop-adjudication.md`
  (REFERENCE-DEFECT verdict for the frozen expected frames)
- Scope: expected-frame regeneration only. Source code is untouched.
- Oracle inputs: frozen binaries built from tag `visual-baseline`
  (4169783ae), verified against their `provenance.json`:
  - `showcase` sha256 `ca3a3114aef32288b998f0fc1976d9b968f2b1d462c8451c39d74360056601a6`
  - `tablepro` sha256 `9c82d9afeae293198d7187e095d49d3dc1d5f859eeb2004742f1ce1d3a46f2d0`
- Renderer: pinned tuiscotti `a1ea27f757d75d07b05499276cd932e9f8e067ad`
  via the visual-harness normalized-capture path
  (`VISUAL_BIN_DIR=<frozen dir> VISUAL_TOOL=capture VISUAL_NAME=...`),
  converted to the canonical artifact set by `capture_all`.

## Independent re-verification (before regenerating)

Fresh captures from the frozen binaries were re-measured with an
independent SGR-state decoder before anything was replaced:

| scenario | tier | adjudication claimed | re-measured | disposition |
|---|---|---|---|---|
| `showcase/flows/dialogs/open/80x24` | 16 / none / nocolor | 920 dim cells | 1819 dim cells padded (1505 after tmux trailing-trim, 794 nonblank) | tier signature identical (DIM present); exact count differs by metric — the adjudication counted a differently-trimmed span set. No material disagreement. |
| `showcase/flows/dialogs/open/80x24` | 256 / truecolor | 0 | 0 dim cells | confirmed |
| `tablepro/overlays/picker_open/120x40` | 16 / none / nocolor | 1847 dim cells (21 runs) | 4660 dim cells / 59 runs | the 1847 figure came from a Ctrl+P capture, which is `TpCmd::Switcher` (`app.rs:2370`) — a different overlay, not the ctrl-o picker. The ctrl-o fresh capture matches the raw legacy snapshot class (4656 cells / 156 runs, full-screen DIM including the popup interior). |
| `tablepro/overlays/picker_open/120x40` | 256 | 82–83 / 1 | 83 dim cells / 1 run | confirmed |
| `tablepro/overlays/picker_open/120x40` | truecolor | 0 | 0 dim cells | confirmed |
| `tablepro/overlays/safemode_picker/*` | 16 / none / nocolor | 4660-class | 4660 dim cells, full-screen DIM | confirmed |

Additional finding from the re-verification: the existing
`snapshots/` frames for the picker families are byte-identical to fresh
frozen-binary captures. The visual-harness snapshots were never defective —
only the corpus conversion (`legacy_replayed_conversion`) dropped the DIM.

## What was regenerated

33 corpus tiers + their snapshots tiers, all from the frozen-binary
captures:

- `showcase/dialogs/open/80x24/{16,none,nocolor}` (legacy name
  `showcase/flows/dialogs/open/80x24/...`)
- `tablepro/workbench/overlay-picker-open/{72x20,80x24,100x30,120x40,160x50}/{16,none,nocolor}`
  (legacy `tablepro/overlays/picker_open/...`)
- `tablepro/workbench/overlay-safemode-picker/{...same sizes...}/{16,none,nocolor}`
  (legacy `tablepro/overlays/safemode_picker/...`)

Per tier: the full 10-artifact corpus set (ansi, txt, frame.json, html,
png, png.fidelity.json, ascii, ascii.loss.json, observations.json,
manifest.json) was rebuilt, `corpus-index.json` manifest hashes and the
admission record's `corpus_index_sha256` updated. The matching
`snapshots/` tiers were re-approved to the committed `.ansi`-only form
(legacy `.txt`/`.png`/`.html` siblings removed). Every replaced artifact
was preserved under
`baselines/regen-sidecars/2026-10-21-backdrop-frame-regeneration/`
(see its README); before/after sha256 for every tier are in
`docs/audits/2026-10-21-backdrop-frame-regeneration-receipts.json`.

Truecolor and 256 corpus tiers were not defective and were not touched.
The truecolor snapshots of both picker families were verified
byte-identical to fresh frozen captures. The dialogs truecolor snapshot
differs from the frozen build's render because the tag build's dialogs
page content itself drifted from the legacy committed snapshots
(breadcrumb "Overlays › Dialogs" vs "Components / Dialogs", different
dialog body); it is out of scope here and left as-is.

## Detection hardening: `frozen-baseline corpus-check`

`tools/frozen_baseline.py` gained a fail-closed `corpus-check` subcommand:
per-cell SGR modifier state decoded from `<tier>.ansi` compared against the
canonical frame JSON `mods`. Because the defective conversion rewrote
`.ansi` and `frame.json` consistently, the intra-store pass alone cannot
see the class; an optional cross-store pass
(`--snapshots`, default `snapshots`) additionally compares each tier
against the conversion-independent snapshot dump selected via the
manifest's `legacy_name`. Ten unit tests pin the behaviour
(`tools/test_frozen_baseline.py::CorpusModifierParity`); the full runner
suite (85 tests) passes.

Ran store-wide, the cross-store pass reports the picker defect class is
**not unique to the picker families**: ~1170 of 1510 corpus scenarios show
modifier disagreement against their snapshots (dropped DIM or bold),
5 `showcase/diff/fade-wheel` scenarios even fail the intra-store pass
(ansi carries dim the frame JSON lacks). Fixing those tiers is outside
this workstream's ownership and was not attempted; the tool now makes
every one of them detectable, and future admissions should run
`frozen-baseline corpus-check` scoped to the admitted tiers.

## Downstream verification (zero source changes)

At the worktree base (9b0277bc6):

- `visual-harness` `store_integrity`, `store_ansi_round_trips`,
  `oracle::manifest_and_hash_are_stable`, `oracle::store_check_accepts_and_rejects`,
  `oracle_integrity`: green.
- `termrock-conformance` `visual_baseline` `store_integrity`: green.
- Harness tier cases (`--run-ignored only`), measured twice:
  - at the worktree base (9b0277bc6, old candidate): 966/1044 green, 78
    red in `showcase::pages::dialogs`, `showcase::flows::dialogs::open::80x24`
    and the `tablepro::overlays` picker families.
  - at the origin tip (ff0005a84) with these baselines cherry-picked:
    212/290 green, 78 red in the same families (control without the
    regeneration: 215/290, 75 red — the three extra reds are the
    previously legacy-class-skipped `flows/dialogs/open` 80x24 tiers,
    which now run and compare).
  All reds are the candidate missing or differently-mapping the dimmed
  backdrop (e.g. safemode picker `none`: expected `fg=#898989
  bg=#000000+dim`, actual `fg=#000000 bg=#000000`; dialogs open `16`:
  expected `#898989` on `#000000+dim`, actual `#b2b2b2` on `#666666+dim`).
  The expected frames are correct against the frozen oracle; the reds are
  the app/theme streams' still-open candidate parity gap for these
  screens and must be closed in source by their owners. The baselines
  need no further edits when that lands.
- Known pre-existing reds at this base, unrelated to this change
  (reproduced on the stashed tree): `modes::jackin_boot_sets_terminal_modes`,
  `modes::holla_boot_sets_terminal_modes`, `determinism::sources_are_deterministic`,
  `overlay_sites`, and conformance `button_busy_live::busy_checked_row_geometry_shared_marker`,
  `negative_controls_phase8ap::negative_phase8ap_focus_hover_disabled_states_differ`.

## Decision

Regeneration approved on the adjudication's REFERENCE-DEFECT verdict: the
corpus frames for the three families were provably missing the backdrop
DIM carried by the raw tag-era streams; the replacements are derived from
the same frozen binaries through the pinned renderer, sidecars preserve
the old artifacts, and the records chain (manifest → corpus index →
admission record) is intact.
