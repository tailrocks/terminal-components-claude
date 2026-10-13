<!--
One paragraph per section, no hard-wrap. Describe the feature, not a file changelog.
Drop Behavior changes when it only repeats What ships. Drop Not included when nothing is deferred.
Verify-locally commands are the repository gates. Static checks apply when Rust sources change.
Tests apply when Rust sources or tests change. Drop Smoke. This repository has no app server.
Drop Documentation unless the diff changes docs the reviewer must render.
-->

## Summary

<What this pull request ships, who it is for, and what changes in their flow.>

## What ships

- <Capability or behavior that now exists>
- <Regression coverage, stated as an outcome>

## Behavior changes

- <Existing behavior that changes>

## Not included

- <Deferred behavior this pull request leaves unchanged>

## Verify locally

### Static checks

Include when the diff changes Rust. `cargo fmt --all -- --check` exits 0 when formatting matches. `cargo clippy --workspace --all-targets --all-features -- -D warnings` exits 0 when no warning remains.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Tests

Include when the diff changes Rust behavior or tests. The filtered command covers the touched package. The workspace command is the full gate. Both exit 0 with a nonzero pass count.

```sh
cargo test -p junie-tui --lib
cargo test --workspace --all-targets --all-features
```

## Migration notes

None.
