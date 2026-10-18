# F12 · Optional terminal/session adapter

**Status:** canonical optional edge-integration contract; implementation is future work.

**Legacy families:** C04.

**Visual authority:** the unchanged `visual-baseline` commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Session integration must preserve the frozen applications' output and observable input behavior when it is used; it does not define a new product.

**Scope:** an optional terminal edge adapter that maps host terminal events to Termrock input, presents prepared frames, and restores terminal state. The core library and components remain headless and usable without this feature.

This adapter is future in-place implementation work on `termrock-implementation` in this repository. It is an optional library edge, not a separate terminal product or repository destination.

See the [runtime architecture](../foundations/runtime.md), [public API contract](../api/public-api.md), [interaction contract](../design/interaction-contract.md), [conformance contract](conformance.md), and [TerminalView boundary](../components/README.md).

## Source evidence

- [`src/runtime.rs`](../../src/runtime.rs) is the baseline's current `Application`/`TerminalSession` implementation. Its crossterm setup, restoration guard, panic hook, and Unix suspend path are evidence for lifecycle cases, not the final Termrock public API.
- [`src/core/event.rs`](../../src/core/event.rs) is the baseline input normalization evidence. The future adapter maps host events into the canonical input/event foundation before component update.
- PTY and terminal cleanup tests remain application/runtime evidence. They do not make Termrock a terminal product.

## Proposed public surface

These declarations are target API notation. The adapter is feature-gated and optional; they are not currently compiled exports.

```rust
#[cfg(feature = "crossterm")]
pub struct TerminalSession { /* owns host terminal modes and backend */ }

#[cfg(feature = "crossterm")]
impl TerminalSession {
    pub fn enter(options: SessionOptions) -> Result<Self, SessionError>;
    pub fn read(
        &mut self,
        deadline: Option<Moment>,
    ) -> Result<Option<Input>, SessionError>;
    pub fn present(&mut self, frame: &PaintedFrame) -> Result<(), SessionError>;
    pub fn restore(&mut self) -> Result<(), SessionError>;
}

pub struct SessionOptions {
    pub mouse: bool,
    pub bracketed_paste: bool,
    pub alternate_screen: bool,
}
```

The final API may use a backend trait or separate read/present handles if that improves headless testing, but it must retain the same ownership and cleanup guarantees. The production core must compile without the backend feature and without a live terminal.

## Boundary and ownership

1. `TerminalSession` owns only host-terminal state it explicitly changes: raw mode, alternate screen, mouse capture, bracketed paste, cursor visibility, line-wrap policy, and backend output/read handles. It restores them in reverse order and at most once.
2. The runtime owns focus, hover, pointer capture, layers, geometry, cursor arbitration, supplied time, and semantic event dispatch. Components own caller-provided model/state and return typed actions. The session adapter does not inspect or mutate component internals.
3. The caller owns domain work, process/PTY handles, terminal emulation/parser state, shell, daemon, agent session, clipboard policy, link policy, title policy, and persistence. Prepared cells may be passed to `TerminalView`; the session does not create them from terminal escape streams.
4. Capability and motion policy are explicit inputs at the edge. Core draw code does not read environment variables, terminal globals, or a real clock. The adapter normalizes host modifiers, key repeat/release, paste boundaries, pointer coordinates, resize, focus, and capability ceilings before runtime dispatch.
5. `SessionError` reports setup, read, present, restore, capability, and terminal-loss failures. A lost output device cannot receive restoration bytes; the adapter reports the error while retaining idempotent in-process cleanup where possible.

## Lifecycle and cleanup contract

Initialization and cleanup are transactional:

- If any setup step fails after a preceding mode was enabled, already-owned modes are restored before the error is returned.
- Normal quit, explicit `restore`, read failure, present/draw failure, and panic/unwind all run the same idempotent restoration path.
- Raw mode, alternate screen, mouse capture, bracketed paste, cursor visibility, and line wrap are restored exactly once per acquisition. Partial setup must not leave a mode enabled.
- A supported suspend/resume path first returns terminal control to the host, stops through the platform's normal job-control mechanism, reacquires modes after continuation, invalidates old geometry/frame cache, and forces a full redraw. Repeated transitions are idempotent.
- Signal handlers, where required, record a request with an async-signal-safe flag only. Terminal writes, cleanup, and re-entry happen on the ordinary event loop.
- Non-Unix targets may omit job-control support. `SIGSTOP`/`SIGKILL` cannot be intercepted, and a dead output device cannot be repaired by the adapter; these are documented platform limits, not reasons to put signal logic in widgets.

The session must not hide cleanup behind a product application's `should_quit` branch. A panic in component update/draw or a present error still unwinds through the session guard. Tests cover each failure point, including setup failure after each individual mode.

## Input, presentation, and capability rules

`read` returns canonical normalized input. It preserves modifiers, repeat/release information, paste boundaries, pointer coordinates, resize/focus events, and any opaque host bytes explicitly needed for a caller policy. It does not dispatch actions itself or invent a second keymap.

`present` submits a `PaintedFrame` produced by the runtime. It does not run component update, perform reconciliation, change focus, or accept a candidate snapshot. The runtime may require presentation acknowledgement for activation feedback timing; the session reports that acknowledgement or failure through the explicit frame path.

The edge adapter negotiates or accepts a capability ceiling (truecolor, 256, 16, none, and explicit `NO_COLOR` policy lanes as applicable) and passes it to theme/rendering. A caller request cannot silently exceed the terminal's capability. Motion policy is explicit and can pause/reduce animation while still allowing externally supplied data updates.

OSC clipboard, links, title updates, pointer-shape emission, and other host escapes are host policy. The adapter must not turn untrusted text into arbitrary escape sequences or copy protected data automatically. Secret fields use the [secret-validation contract](secret-validation.md); a terminal session cannot bypass its no-copy default.

## Product and terminal boundary

Termrock is not:

- a terminal emulator;
- a PTY runtime or child-process supervisor;
- a shell or daemon;
- an agent-session manager;
- a terminal escape parser;
- a Jackin, Holla, TablePro, or Showcase product runtime.

Those concerns remain external. A future application may pair an external emulator/parser/PTY with `TerminalView`, which consumes prepared cells and returns typed copy/link/interaction requests. Session tests run a tiny generic component gallery or micro-scene executable, never a new product application and never a live provider/service workflow.

## Verification

The implementation phase must add these gates:

### Core and backend separation

- Headless Termrock core builds and runs component update/draw/measure with no backend/default session feature.
- The optional crossterm adapter compiles independently and has no dependency on product binaries, service clients, emulator code, or PTY business logic.
- Normalized resize, focus, mouse, paste, key repeat/release, capability, and motion inputs reach the runtime exactly once.

### Cleanup and failure paths

- Every setup failure, read failure, present/draw failure, normal exit, panic/unwind, and explicit restore leaves all owned modes in their prior host state.
- Mouse and bracketed-paste modes, alternate screen, raw mode, cursor visibility, and line wrap are enabled/restored exactly once, including partial initialization.
- Shutdown while a component owns focus, pointer capture, a drag, or an active layer releases runtime ownership before returning to the host.
- Suspend/resume restores the shell, re-enters deterministically, invalidates geometry, and redraws. Unsupported platforms and unrecoverable output loss report explicit limits.

### PTY and parity boundaries

- PTY tests exercise only a tiny generic gallery/micro-scene and terminal cleanup/input mapping.
- Prepared-cell `TerminalView` cases compare symbols, continuation, supported styles, cursor/selection, and typed requests through the conformance lanes; they do not test an in-library parser.
- Baseline application scenarios and snapshots remain unchanged. A backend test may prove the path used to display them, but it cannot bless new expected output or authorize an application redesign.
- Mutation tests that skip cleanup, double-activate a key, route paste outside the focused layer, exceed a capability ceiling, or parse/execute terminal escape text inside Termrock must fail.

## Migration note

The current `src/runtime.rs` is a broad application runtime with a `TerminalSession` type and Unix job-control code. During the in-place refactor, implementation may be decomposed into the minimal headless runtime plus this optional edge adapter. Existing application code remains the frozen reference until a separately scoped migration task. This document is the canonical owner for terminal/session lifecycle and backend boundaries; runtime architecture owns frame semantics, while [TerminalView](../components/README.md) owns prepared-cell presentation.
