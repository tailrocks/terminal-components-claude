# F12 · Optional terminal/session adapter

**Scope:** shared generic library/test infrastructure, not a product subsystem.  
**Legacy families:** C04  
**Visual source:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; interface spellings below are proposals.

## Source references

- [src/runtime.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/runtime.rs)

[Target architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) · [refactoring task catalog](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv)

## Proposed public surface

Rustdoc-style declaration notation. This is not a compiled implementation. Read with [PUBLIC-API.md](../PUBLIC-API.md) and the [shared type dictionary](../reference/TYPES.md).

```rust
// Available only behind the crossterm feature.
TerminalSession::enter(options: SessionOptions) -> Result<TerminalSession, SessionError>
TerminalSession::read(&mut self, deadline: Option<Moment>) -> Result<Option<Input>, SessionError>
TerminalSession::present(&mut self, frame: &PaintedFrame) -> Result<(), SessionError>
TerminalSession::restore(&mut self) -> Result<(), SessionError>
SessionOptions { mouse: bool, bracketed_paste: bool, alternate_screen: bool }
```

## Contract

1. Components and Runtime work headlessly without this feature. Adapter maps terminal events, presents cells and scopes raw mode/alternate screen/mouse/paste/cursor settings.
2. Normal exit, initialization failure, draw failure, panic/unwind and supported suspend/resume paths restore owned terminal state. Signal behavior belongs to the platform adapter, not widget code.
3. Capabilities and motion policy are explicit. Detect/accept a capability ceiling at the edge; core rendering never reads environment variables or real clocks.
4. OSC clipboard, links, title and pointer-shape emission are host policy. The adapter must not turn untrusted text into arbitrary escape sequences or automatically copy protected data.
5. PTY tests run a tiny component gallery or micro-scene executable, not a new Jackin application. Child PTY processing/emulation for a future product is not part of this session adapter.

## Required proof

- cleanup after every failure point and unwind
- mouse/paste mode enabled and restored exactly once
- resize/focus events correctly normalized
- shutdown while component capture/layer active
- headless core builds with no backend/default features

This foundation has no invented independent hover/pressed screenshot. Its visible effects are proved through the components and composed fixtures that use it. Nonvisual invariants have headless/state/compile-fail/protocol tests. Implementation and independent verification are not executed in this document pack.
