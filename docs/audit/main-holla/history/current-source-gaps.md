# Current-source corroboration

Read-only source inspection; no runtime reproduction claimed.

- xtask/src/main.rs SHA256 a306865782095880b7d9d471693b024b7b4ddfad896737372062cfb1f002bf31
- crates/tui/src/runtime.rs SHA256 63bd832578162f6ea03c6e61bb9f67f66f0ddaea27fc1851409e00d5422d90f5
- crates/tui-testing/src/conformance/driver.rs SHA256 efbb58ec51972506be79645abf8c5b3d9124845639a77baedff03eecb08ce92b

1. `xtask/src/main.rs:7118`: `matches!(n, 3..=17 | 21..=u32::MAX)` excludes migration/alternatives/performance+visual change sections18–20; open-ended tail correction does not close this original scope hole.
2. `runtime.rs:737–763`: captured pointer emits Drag/Release/DragEnd/Click/Press without checking disabled admissibility. Current contract needs active capture policy and tests; no claim of live reproduction.
3. `conformance/driver.rs:653–667`: local override assertion accepts same final digest if recorded Resolved differs; source proves queried-style effect, not visible cell customization. User requires independent visible sentinel proof.
