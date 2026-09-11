# F03: immutable Settings failure through the full application

Status: the failure is reproduced; its product-compatibility disposition awaits user direction. This is diagnostic evidence, not a refactored implementation or an approved post-failure baseline.

## Source and boundary

The disposable source tree is `/tmp/showcase-reaudit.AWN3SD`. The existing actual `App` test harness uses `ratatui::Terminal<TestBackend>` and calls `App::handle`, followed by actual rendering, after every key. Focus changes use repeated ordinary Tab inputs until the published focus owner matches; no private focus assignment, list mutation or direct removal call is used. Initial `App::goto(Settings)` is the existing supported page selection used by the oracle CLI. The test changes only the disposable `src/bin/showcase/app_tests.rs`.

The four production files were independently hashed and compared with immutable commit `02f5294bfdbf38004cc49130d0aff1d01f31434c`:

| File | Exact Git blob |
| --- | --- |
| `src/bin/showcase/app.rs` | `c735c1bee0c77220447c8864e33c598da4215fd2` |
| `src/bin/showcase/main.rs` | `b225ead7cb501c93a4ffcff37b6c40b5e7ec7883` |
| `src/bin/showcase/pages/settings.rs` | `8123f7f319a13c8e5a3ecbc2d19586c61ef684d2` |
| `src/widgets/list.rs` | `44cbbab4fdd3a04f67a2712f2be241264a966ee1` |

## Reproduction and observation

At each of `120x40`, `160x50`, `80x24` and `72x20`: start Settings; Tab to page controls; `3` selects Environment; Tab to `settings/env`; Down three times; Shift+Down; Tab to `settings/rmvars`; Enter; assert the rendered status contains `Removed 2 variables`; Tab back to `settings/env`; Shift+Up. Every preceding event completes its real application redraw.

The final input panics at immutable `src/widgets/list.rs:99:31` with `index out of bounds: the len is 3 but the index is 3`, at all four sizes. The test catches the panic only to verify and report the diagnostic; it does not produce a successful continuation frame. Command executed twice:

```sh
rtk proxy cargo test --locked --bin showcase reaudit_settings_delete_then_shift_full_app -- --nocapture
```

Result: one diagnostic test passed, all four expected failures observed. This result is not application correctness. It upgrades the earlier direct production-page reproduction in `history-semantic-reaudit.md` by proving normal App focus and redraw do not eliminate the stale anchor.

## Enabling condition and decision boundary

Settings removes selected items and updates cursor/check storage, while ListBox retains a private range anchor pointing beyond the new item count. Later range navigation clamps the cursor but indexes through that old anchor. Main's accepted keyed identity/reconciliation architecture should not be weakened to manufacture this panic. Conversely, a successful post-deletion continuation cannot truthfully be described as identical to the oracle's crashing trajectory.

The user has been asked whether to grant an explicit, narrowly bounded crash exception preserving pre-crash behavior and requiring safe continuation. Until answered, F03 remains an unresolved authority conflict, not an implicit permission to redesign behavior, a discarded historical requirement, or an approved candidate failure. Other independent planning and branch-diff review can continue.
