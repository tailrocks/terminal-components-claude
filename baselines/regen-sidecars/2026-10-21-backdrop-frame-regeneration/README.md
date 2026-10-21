# Sidecar: 2026-10-21 backdrop frame regeneration

Preserved (never deleted) copies of the pre-regeneration artifacts replaced by
the backdrop expected-frame regeneration on this branch. Nothing here is read
by any gate; the live artifacts live in `baselines/tuiscotti-v1/` (corpus) and
`snapshots/` (visual-harness store).

- `corpus/` — the defective corpus tiers (`<tier>.{ansi,txt,frame.json,html,png,png.fidelity.json,ascii,ascii.loss.json,observations.json,manifest.json}`)
  for `showcase/dialogs/open/80x24/{16,none,nocolor}`,
  `tablepro/workbench/overlay-picker-open/{size}/{16,none,nocolor}` and
  `tablepro/workbench/overlay-safemode-picker/{size}/{16,none,nocolor}`
  (legacy names `showcase/flows/dialogs/open/...`, `tablepro/overlays/picker_open/...`,
  `tablepro/overlays/safemode_picker/...`).
- `snapshots/` — the previous visual-harness snapshot artifacts for the same
  tiers (`.ansi` plus any legacy `.txt`/`.png`/`.html` siblings that existed).

Why they were replaced: the original corpus conversion dropped SGR modifier
state — the full-screen backdrop DIM (SGR-2 runs present in the raw capture
stream) is missing from both the stored `.ansi` and the frame JSON `mods`,
consistently, so the defect was invisible to glyph- and intra-store checks.
Detection and scope: `docs/audits/2026-10-21-modal-backdrop-adjudication.md`
(adjudication) and `docs/audits/2026-10-21-backdrop-frame-regeneration.md`
(regeneration record: before/after hashes, method, receipts).

The modifier-aware cross-store parity check that now detects this class lives
in `tools/frozen_baseline.py` (`frozen-baseline corpus-check`).
