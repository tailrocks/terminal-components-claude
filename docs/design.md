# Design and interaction contract

## Status

`design/interaction-polish` defines the intended visual language, colors, layout, interaction, and behavior. Planned improvements remain active until their individual acceptance evidence is recorded.

## Authority

Use this branch's UI/UX as the target. Preserve its established appearance and behavior while implementing the split-crate architecture described in `docs/architecture.md`. The `main` branch's incomplete refactor is not the visual or behavioral authority. Reusable API proposals from either branch may inform the final API, subject to the crate architecture.

## Current evidence

The specifications and plans under `docs/design/`, `docs/audits/`, `docs/improvements/`, `docs/parity/`, and `docs/visual-testing/` record intended behavior, findings, and historical verification. Each record's own scope and date controls what it proves. A planned behavior is not implemented merely because it appears here; a historical pass is not proof for the current revision.

## Remaining work

Keep all planned work from `design/interaction-polish`, including open audits, improvements, parity gaps, and refactoring tasks. Resolve contradictory or superseded prose by updating the relevant active contract and linking historical evidence as historical. Do not silently discard a concrete requirement or finding.

## Verification

Use `docs/visual-testing/README.md` and the current visual baseline procedures to verify rendered appearance and interaction. Use the relevant component or app tests for behavior. Record the tested revision, scope, and result. No documentation change authorizes a code or behavior change by itself.
