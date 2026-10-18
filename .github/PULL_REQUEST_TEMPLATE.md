## Summary

<!-- State what changes for users or maintainers in this WIP. -->

## What ships

- <!-- Capability, documentation contract, or maintenance outcome. -->

## Behavior changes

- <!-- Describe any user-visible behavior change; remove this section if none. -->

## Not included

- <!-- Note related work intentionally left open; remove this section if none. -->

## Verification

Run the checks that cover the changed files and acceptance criteria. A passing check proves only its revision and exercised scope. This WIP does not establish that `main` is ready or that all work in `design/interaction-polish` is implemented.

### Rust changes

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo nextest run --all-targets
```

### UI or visual changes

```sh
cargo xtask visual check-touched
```

Record the exact scenarios, terminal sizes, color levels, and approval scope reviewed. Follow [the visual testing guide](../docs/visual-testing/README.md); do not accept new expected output from candidate output alone.

### Documentation-only changes

- <!-- List rendered pages, generated-doc freshness, link checks, or tool-consumed docs reviewed. Remove if not applicable. -->

## Migration notes

- <!-- Include compatibility, task IDs/dependencies, and remaining work where relevant. -->
