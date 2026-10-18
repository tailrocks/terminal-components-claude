# Component authoring

## Status

Current API index only. This is not a compiler-verified component tutorial: the previously cited `crates/tui/examples/12_author_component.rs` does not exist in the current split workspace, and `junie_tui::author` is not the current facade path. Independent downstream component acceptance remains unverified.

## Authority

Current architecture: [architecture](../architecture.md). Product behavior, visual appearance, and colors: `design/interaction-polish`. Public facade: [`crates/termrock/src/lib.rs`](../../crates/termrock/src/lib.rs), whose `author` module delegates to [`termrock-runtime/src/author.rs`](../../crates/termrock-runtime/src/author.rs).

## Current evidence

The current facade path is `termrock::author`. It reexports selected runtime-authoring types, including `Id`, `ItemKey`, `Part`, `PartRef`, input and intent types, `Response`, `StateFlags`, `Cx`, `FrameRead`, `Ui`, focus and scope types, hit and layer types, theme types, layout constraints, text types, collection types, and ratatui buffer/layout/style types. The owning runtime module defines `PartStyle` helpers. This is an API surface inventory, not proof that an external component can meet every runtime, styling, registration, or conformance requirement through public APIs.

The former worked example and its detailed authoring contract are preserved at immutable commit [`d1122f69`](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/guides/authoring.md). Treat its claims as historical proposals; it contains obsolete imports, paths, and assertions. No example there is claimed to compile against the current workspace.

## Remaining work

- Build a minimal independent crate using only published `termrock` APIs.
- Verify component construction, caller-owned interaction state, update/draw phase separation, stable item identity, focus and hit registration, theme parts and overrides, layout measurement, and deterministic input behavior.
- Verify a supported conformance path for downstream components, including styled-part accounting and tiny or empty geometry.
- Add a current example only after that consumer compiles and its behavior is verified against current APIs and the design reference.

Until that proof exists, use current crate exports and source directly. Do not rely on the former `junie_tui::author` import block or the nonexistent example path.

## Verification

Inspect current exports and build an independent consumer crate before describing a component API as compiler-verified. Compare rendered behavior with `design/interaction-polish`; a library build does not establish consumer usability or product parity.
