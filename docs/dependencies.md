# Dependencies

Keep dependencies on their latest releases (crates, the Rust
toolchain, Tuiscotti, velnor-actions); upgrade at least monthly. A
version held below latest is recorded here with its blocker. Never
vendor or patch an upstream crate to get around a pin.

| Dependency | Pin | Notes |
| --- | --- | --- |
| Rust | 1.99.1 (`rust-toolchain.toml`, `mise.toml`) | Latest stable at last review. |
| `tuiscotti-tui-test` (`tui_test`) | `=0.2.1` | Live PTY driver for functional tests. |
| `tuiscotti` (`tuiscotti-latest`) | git rev `d8d54155947f6303a41dfd6079b53c0abc16f29a` | Offline `FrozenStore` visual gate; bump to newest `main` rev deliberately when it moves. |
| velnor-actions | 0.1.71 (`mise.toml`) | Generates CI from `.velnor/config.toml`; see [CI](ci.md). |
| mr-boxington (`mbx`) | 1.29.1 (`mise.toml`, `postinstall = "mbx setup --yes"`) | Cargo wrapper via mise (`[wrappers.cargo]`, `mr_boxington = true` on the rust tool); CI uses the velnor `mbx` compile driver. |

Reviewed 2026-11-01. Testing-stack rationale is in
[functional testing](functional-testing.md).
