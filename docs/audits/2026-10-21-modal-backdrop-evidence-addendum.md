# Modal backdrop evidence addendum

- Date: 2026-10-21
- Scope: provenance correction for `2026-10-21-modal-backdrop-dim-vs-frozen-baseline.md`
- Independent review: `modal-dim-independent-review-ffa723642/review.json`
- Review SHA-256: `7257382a92be0845c9b5d5567e799e0cd64db348b736e87d3aa123c113bc18a1`
- Frozen source: `4169783ae883788186d161af09ba9c481bd84e5c`
- Incoming source reviewed: `7c94b00331413350f37365c7a3a34151dc952e38`

## Supersedes the earlier oracle conclusion

The earlier report accurately counted DIM attributes in stored ANSI files, but those files do not prove output from a fresh run of the frozen CLI. The 80x24 Showcase artifact with zero DIM attributes predates the frozen `Theme::backdrop` implementation. The TablePro digest cited there comes from the candidate conformance corpus, not a frozen CLI capture. The statement that the frozen baseline disagrees with itself, and the claim that the stored captures are approved frozen output, are therefore withdrawn.

Keep the original report as a dated record of its investigation. Do not use its stored-artifact counts as a reference decision, parity approval, or reason to change backdrop behavior. No fresh frozen CLI capture or repeated reference run was performed for this addendum.

The incoming source commit changed documentation and handoff files only. Its Theme and runtime backdrop implementations are byte-identical to cleared source `02742e48d58ce319ce98e64095ad86cb1709f5a`; this review found no new production DIM behavior defect. This source comparison does not establish frozen-output parity.

## Escape vocabulary finding

Frozen source contains a visible Dialog contract for the cancel label `Keep launching`: `src/bin/showcase/pages/launch_cockpit.rs:415` sets that label, and `src/widgets/dialog.rs:1987-1997` checks the resulting `Esc Keep launching` hint. The Escape vocabulary gate must preserve this exact baseline behavior. This exception does not approve the candidate's separate hard-coded Jackin presentation path or establish that its screen implementation is structurally valid.

The updated gate scans production `src/` files, excluding comments, documentation, test directories, and inline `#[cfg(test)]` modules. It permits only the exact `Keep launching` label in `jackin-preview-app/src/app/launch_cockpit.rs`, pins the frozen source commit and visible hint evidence, and asserts the two reviewed occurrences. A new non-verb label at that path or the same frozen label at another path remains a failure.

Validation on this source: the focused `esc_vocabulary` nextest target passed 5/5, including both negative fixtures. The full `termrock-theme` nextest run produced 153 passes and the known failure `capability_palettes::all_73_junie_slots_match_reference_capability_output` (0 skipped); this work does not refresh or approve those palette expectations. Strict package Clippy and `rustfmt --check` passed.

## Remaining evidence gap

Fresh, repeated CLI captures of the fixed frozen binaries and candidate binaries remain required before any application parity claim. Preserve stored artifacts and their hashes; do not rewrite them to resolve this provenance gap.
