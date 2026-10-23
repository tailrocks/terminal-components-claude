# Quickstart

`termrock` is the workspace facade package and the Rust crate name. Its API is still under development. Start with [architecture](../architecture.md), the [product reference](../product.md), and the [component authoring contract](../contracts/authoring.md). See [development](../development.md) for workspace workflows and guidance about interactive applications.

## An app updates, then draws

Keep application data and stable component IDs in your `App`. In `update`, pass `Cx` to the controls, handle their typed responses, and return the combined response. In `draw`, use the same IDs and current state to draw through `Ui`; drawing also registers the geometry used by later input.

The small [SaveProbe example](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock/tests/ex01_button.rs#L9-L42) separates button update and draw from the `App` implementation. Its [test](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock/tests/ex01_button.rs#L64-L107) covers a disabled Save button: update returns an ignored response with no activation, and draw paints its label. It is a minimal lifecycle example, not an enabled-button input journey.

The facade exports `App`, `Cx`, `Ui`, `Runtime`, and `run`. Its [`termrock::author` namespace](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock/src/lib.rs#L61-L80) exposes the registration vocabulary described by the authoring contract.

## Follow a complete consumer

The [`termrock-consumer-example` manifest](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock-consumer-example/Cargo.toml#L1-L26) marks the example `publish = false`; `termrock` is its only normal dependency, and Ratatui is a test-only dependency. This is an unpublished package inside the root workspace, not an independent published consumer. It does not establish compatibility for a separately released third-party crate.

Its [application](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock-consumer-example/src/app.rs#L291-L430) implements `App::update` and `App::draw` with shared controls and composition. Its [binary entry point](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock-consumer-example/src/main.rs#L10-L12) passes the app to `termrock::run`; this source link does not mean the binary was launched.

For deterministic tests or a custom event loop, inspect the [consumer's Runtime helpers](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock-consumer-example/tests/consumer.rs#L38-L93) and [journeys](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock-consumer-example/tests/consumer.rs#L96-L350). The public presentation path is `prepare`, `draw_prepared`, then `acknowledge_presented`; the tests also send input with `Runtime::handle`. They use Ratatui's `TestBackend` and cover rendered content and layout, typed keyboard and pointer actions, app actions, and narrow-screen detail focus and restoration. They do not exercise a PTY, clipboard, or operating-system input, and they do not prove full visual parity.

## The `testing` feature

The facade's [`ex01_button` target requires `testing`](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock/Cargo.toml#L8-L10), as does the target's [manifest entry](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock/Cargo.toml#L35-L43). That test uses the facade's `initialize`, `draw_buffer`, and `commit_presented` helpers. The consumer example instead tests the public `Runtime` presentation path and does not require the facade's `testing` feature.

From the repository root, these are the focused checks for the linked examples. A result must be recorded against the source actually checked; this page reports no fresh test or compile result.

```sh
/Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo check --locked -p termrock-consumer-example
/Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo nextest run --locked -p termrock-consumer-example --test consumer
/Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo nextest run --locked -p termrock --features testing --test ex01_button
```

## Historical source

This page keeps the useful facade, update/draw, testing-feature, and `Runtime` guidance from the [original quickstart at `b0edda8`](https://github.com/tailrocks/terminal-components-claude/blob/b0edda802880473f5541dc71d18d5cfe5ecd991c/docs/guides/quickstart.md). The original inline code and direct Cargo commands are historical references, not current instructions.
