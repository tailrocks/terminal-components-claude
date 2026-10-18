# Preserved application consumers

`showcase`, `tablepro`, `jackin-preview`, and `holla` are reference consumers of
the reusable TUI library in this repository. They are part of the Termrock
refactoring contract. Their current source, fixtures, routes, keyboard and
pointer behavior, rendered output, and approved snapshots remain unchanged.

## Authority

The immutable visual and observable-interaction oracle is the annotated
`visual-baseline` commit:

```text
4a79c0a2d40fca46fc406b77157ce3b3f12ec16b
```

The frozen source is under `src/bin/`; executable parity cases are under
`tests/visual_baseline/`; approved output is under `snapshots/`. The future
implementation may replace library internals, but every applicable frame and
interaction must remain 1:1 with that oracle. Exact comparison includes
dimensions, symbols and wide-cell continuation, supported colors and
modifiers, cursor state, focus owner, pointer capture owner, selected stable
key, draft and committed values, navigation key, and typed action count and
target.

The detailed gates are [visual parity](../verification/visual-parity.md),
[interaction parity](../verification/interaction-parity.md), and
[oracle provenance](../verification/oracle-and-provenance.md). Component and
shared-mechanism ownership lives in [component contracts](../components/README.md)
and [foundation contracts](../foundations/README.md).

## Shared inspection contract

The baseline capture matrix uses these terminal sizes:

```text
72×20   80×24   100×30   120×40   160×50
```

The qualified color paths are truecolor, ANSI 256, ANSI 16, explicit
monochrome (`none`), and the `NO_COLOR` environment path (`nocolor`) where a
capture declares it. The 72×20 frame is the minimum supported application
geometry; smaller frames show the existing too-small behavior. A component may
have additional local boundary cases in its own contract.

Use the existing binaries to inspect a fixture. Use the ignored visual suite
to inspect or compare approved output:

```sh
cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
cargo nextest run --run-ignored only -E 'binary(visual_baseline) & test(showcase_)'
cargo nextest run --run-ignored only -E 'binary(visual_baseline) & test(tablepro_)'
cargo nextest run --run-ignored only -E 'binary(visual_baseline) & test(jackin_)'
cargo nextest run --run-ignored only -E 'binary(visual_baseline) & test(holla_)'
```

The suite is evidence against the frozen store. It must never accept a
candidate frame as a new expected frame. Preserve all four artifact types
(`.ansi`, `.txt`, `.png`, `.html`) under `snapshots/`.

## Current names and future identity

At this documentation-only stage, `Cargo.toml` still names the package
`junie-tui` and the library `junie_tui`. Those are current implementation
identifiers, not the target architecture name. The eventual library identity
is Termrock. Renaming that library or its source modules is future
implementation work and must not alter any application-visible string or
fixture behavior. In particular, the literal marks `jackin❯` and `holla❯`,
the `TablePro` label, and the current showcase title remain baseline content.

The application migration boundary is narrow: move these consumers onto the
Termrock public components as the library is refactored, while preserving the
same product fixtures, scenarios, routes, output, and interactions. This
section does not authorize product development, service integrations, data
backends, or visual redesign.

## Consumers

| Consumer | Conformance role | Current entry point |
| --- | --- | --- |
| [Showcase](showcase.md) | Component laboratory and state reference | `src/bin/showcase/`, `showcase` |
| [TablePro](tablepro.md) | Grid, editor, overlay, selection, form, and workbench composition | `src/bin/tablepro/`, `tablepro` |
| [Jackin Preview](jackin-preview.md) | Host-management, accounts, usage, launch, overlay, and terminal-pane composition | `src/bin/jackin_preview/`, `jackin-preview` |
| [Holla](holla.md) | Finder, preview, action, plan, output, and context-adaptive composition | `src/bin/holla/`, `holla` |
