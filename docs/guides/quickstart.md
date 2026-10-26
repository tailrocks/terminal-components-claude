# Quickstart

`termrock` is the workspace facade package and the Rust crate name. Its API is still under development. Start with [architecture](../architecture.md), the [product reference](../product.md), and the [component authoring contract](../contracts/authoring.md). See [development](../development.md) for workspace workflows and guidance about interactive applications.

T37 retires the legacy test targets cited on this page. Commit-pinned test links below are historical source evidence only. They do not identify current targets or prove current behavior. T01 and T35 must update the API and user guidance after T38 approves the functional test architecture. Product behavior remains `NOT VERIFIED`.

The API and command examples also refer to older source snapshots. Check the
cleaned source before using them. T35 owns the current quickstart rewrite.

## Choose terminal colors

[termrock::run](https://github.com/tailrocks/terminal-components-claude/blob/cd349edc6dbaa1d2e550e69a7fb5edae1a766bf1/crates/termrock-session/src/session.rs#L537-L539) accepts a starting theme and detects terminal capability during session startup ([the startup helper applies Theme::for_terminal() before creating Runtime](https://github.com/tailrocks/terminal-components-claude/blob/cd349edc6dbaa1d2e550e69a7fb5edae1a766bf1/crates/termrock-session/src/session.rs#L638)). The usual call needs no manual detection: `termrock::run(app, termrock::Theme::junie())`. The [consumer binary](../../crates/termrock-consumer-example/src/main.rs) uses this form. The [historical consumer test](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-consumer-example/tests/consumer.rs#L27-L56) recorded the two call forms and their starting theme levels. T37 retired that target.

To set an upper bound, pass [Theme::for_level(...)](https://github.com/tailrocks/terminal-components-claude/blob/cd349edc6dbaa1d2e550e69a7fb5edae1a766bf1/crates/termrock-theme/src/downgrade.rs#L404-L407): `termrock::run(app, termrock::Theme::junie().for_level(termrock::ColorLevel::Ansi16))`.

The session may narrow that theme further to the detected level; for_level() does not force that level or bypass detection. A caller that constructs [Runtime directly](https://github.com/tailrocks/terminal-components-claude/blob/cd349edc6dbaa1d2e550e69a7fb5edae1a766bf1/crates/termrock-runtime/src/runtime.rs#L714-L727) keeps the supplied theme. That path can opt into detection with [Theme::for_terminal()](https://github.com/tailrocks/terminal-components-claude/blob/cd349edc6dbaa1d2e550e69a7fb5edae1a766bf1/crates/termrock-theme/src/downgrade.rs#L410-L412); applying it before termrock::run() is redundant.

[ColorLevel::detect()](https://github.com/tailrocks/terminal-components-claude/blob/cd349edc6dbaa1d2e550e69a7fb5edae1a766bf1/crates/termrock-theme/src/tokens.rs#L435-L480) reads CLICOLOR_FORCE, NO_COLOR, TERM, COLORTERM, and whether stdout is a terminal. TERM=dumb selects Mono first. Otherwise, a nonempty CLICOLOR_FORCE value other than 0 skips the NO_COLOR and non-TTY Mono checks, but does not select TrueColor by itself. Without that force, nonempty NO_COLOR or non-TTY stdout selects Mono. Then COLORTERM=truecolor or 24bit selects TrueColor; TERM containing 256color, ghostty, or kitty selects Ansi256; remaining terminals select Ansi16. See the [theme and visual customization contract](../contracts/theming.md) for capability profiles and evidence limits.

## An app updates, then draws

Keep application data and stable component IDs in your `App`. In `update`, pass `Cx` to the controls, handle their typed responses, and return the combined response. In `draw`, use the same IDs and current state to draw through `Ui`; drawing also registers the geometry used by later input.

The [historical SaveProbe example](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock/tests/ex01_button.rs#L9-L42) separates button update and draw from the `App` implementation. Its [historical test](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock/tests/ex01_button.rs#L64-L107) covers a disabled Save button. T37 retired this target; the cited result is not current proof.

The facade exports `App`, `Cx`, `Ui`, `Runtime`, and `run`. Its [`termrock::author` namespace](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock/src/lib.rs#L61-L80) exposes the registration vocabulary described by the authoring contract.

## Follow a complete consumer

The [`termrock-consumer-example` manifest](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock-consumer-example/Cargo.toml#L1-L26) marks the example `publish = false`; `termrock` is its only normal dependency, and Ratatui is a test-only dependency. This is an unpublished package inside the root workspace, not an independent published consumer. It does not establish compatibility for a separately released third-party crate.

Its [application](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock-consumer-example/src/app.rs#L291-L430) implements `App::update` and `App::draw` with shared controls and composition. Its [binary entry point](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock-consumer-example/src/main.rs#L10-L12) passes the app to `termrock::run`; this source link does not mean the binary was launched.

The [historical consumer Runtime helpers](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock-consumer-example/tests/consumer.rs#L38-L93) and [historical journeys](https://github.com/tailrocks/terminal-components-claude/blob/07ba30f5cb33410d7d6fbe09a3691c877f082915/crates/termrock-consumer-example/tests/consumer.rs#L96-L350) record the `prepare`, `draw_prepared`, `acknowledge_presented`, and `Runtime::handle` paths. T37 retired those targets. Their old cases do not establish current behavior, PTY handling, clipboard handling, operating-system input, or full visual parity.

## Historical facade test targets

The facade's [`testing` feature and `ex01_button` target](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock/Cargo.toml#L8-L43) are recorded at the pre-cleanup source revision. T37 retired the legacy facade test targets. Do not run their old Nextest commands. T05 will classify any remaining compatibility feature; a replacement test needs an approved T38 plan.

## Historical source

This page keeps the useful facade, update/draw, testing-feature, and `Runtime` guidance from the [original quickstart at `b0edda8`](https://github.com/tailrocks/terminal-components-claude/blob/b0edda802880473f5541dc71d18d5cfe5ecd991c/docs/guides/quickstart.md). The original inline code and direct Cargo commands are historical references, not current instructions.
