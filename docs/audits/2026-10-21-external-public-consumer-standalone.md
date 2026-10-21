# External public consumer, standalone (not a workspace member)

Base under test: `5478ecfc93e294ccd9ebd75058de7be09e2a389f` (tip of
`parity/termrock-visual-baseline`).

## What this closes

The prior demonstration of the external-consumer gate
(`origin/wip/external-public-consumer-20261021`, commits `154495e2b` and
`fd1e65243`, crate `crates/termrock-consumer-example`) was sound in its API
usage but was registered as a **workspace member** (`members` entry in the
root `Cargo.toml`, workspace-inherited manifests and lockfile). A workspace
member resolves its dependencies through the repository's workspace — that is
not the standing of a real external consumer.

This increment lands the same demonstration as
`external-consumer-demo/`, a crate that is **not a workspace member**:

- the directory is absent from the root `members` list (`cargo metadata
  --no-deps` at the workspace root does not list it);
- its `Cargo.toml` carries an empty `[workspace]` table, so it resolves and
  locks on its own (`external-consumer-demo/Cargo.lock`);
- it depends on the public `termrock` facade by path and on `ratatui 0.30`
  (dev-only, for the public `TestBackend`) and builds standalone:

  ```text
  cargo nextest run --manifest-path external-consumer-demo/Cargo.toml
  ```

The domain (a recipe shelf) and the composed screen are carried over from the
prior art after re-verification against the current tip; the widget-level
delta of `fd1e65243` (the `MasterDetailAction` import, the `NavListAction`
mapping that no longer early-returns, the narrow-terminal collapse test) is
included.

## The composed screen

```text
termrock::AppShell
 ├── header : termrock::overlays::MenuBar
 ├── body   : termrock::blocks::MasterDetail
 │    ├── master  : termrock::navigation::NavList
 │    └── detail  : termrock::controls::PropsList
 │                  termrock::controls::Button   (action row)
 └── footer : termrock::feedback::StatusBar
```

The consumer implements `termrock::author::CompositionLeaf` for the three
shell leaves; the leaves own no geometry — each passes its resolved region
straight to the shared widget. The consumer computes no rectangle, paints no
cell and matches on no raw key or mouse event.

## Exact public API surface used

| Purpose | Public item |
| --- | --- |
| App contract | `termrock::App`, `termrock::Cx`, `termrock::Ui`, `termrock::Response` |
| Runtime | `termrock::Runtime` (`prepare`, `draw_prepared`, `acknowledge_presented`, `handle`, `area_of`, `area_of_part`, `focus`) — none of these behind a feature flag |
| Theme | `termrock::Theme::junie` |
| Shell | `termrock::AppShell`, `termrock::Chrome` |
| Leaf protocol | `termrock::author::{CompositionLeaf, Measure, Constraints, Size}` |
| Layout block | `termrock::blocks::{MasterDetail, MasterDetailAction, MasterDetailPolicy, MasterDetailState}` |
| Composition | `termrock::composition::{Group, button, props_list, nav_list}` |
| List widget | `termrock::navigation::{NavList, NavListState, NavListAction}` |
| Detail widget | `termrock::controls::{PropsList, PropsRow, PropsState, Button}` |
| Menu bar | `termrock::overlays::{MenuBar, Menu, MenuItem, MenuState, MenuAction}` |
| Status bar | `termrock::feedback::{StatusBar, StatusItem, StatusAction}` |
| Identifiers | `termrock::{Id, ItemKey, ActionKey, Part, PartRef}` |
| Events | `termrock::{Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind, Rect, Position, Buffer}` |
| Session (binary) | `termrock::run` |

## Public-API-only proof

`external-consumer-demo/tests/public_api_only.rs` asserts the contract in the
crate itself:

- the active (non-comment) manifest lines name `termrock = { path =
  "../crates/termrock" }` and no `termrock-*` crate as the consumer's own
  dependency;
- every `use` / `extern crate` line in `src/` and `tests/` imports the
  `termrock` facade only — no `termrock_*` internal crate identifier appears.

(The facade's internal `termrock-*` library crates do appear in the
standalone lockfile as the facade's own transitive path dependencies; that is
the facade's implementation, not the consumer reaching into it. The consumer
imports none of them.)

## Typed update → typed action flow

Unchanged from the prior art: shared widgets answer with their own typed
actions (`NavListAction`, `MasterDetailAction`, `MenuAction`,
`StatusAction`, `Button`'s `activated()`), and `RecipeBox::dispatch` is the
single place that maps them onto the consumer's own `Action` enum and mutates
domain state. The typed action history is exposed for assertions
(`RecipeBox::actions`, `RecipeBox::last_action`).

## Test evidence

`external-consumer-demo/tests/consumer.rs` drives the consumer through the
public production render path (`Runtime::prepare` → `Runtime::draw_prepared`
→ `Runtime::acknowledge_presented`) on Ratatui's public `TestBackend`:

| Test | What it proves |
| --- | --- |
| `external_consumer_renders_its_own_domain_data_in_a_stable_layout` | The consumer's own recipes, the menu bar and the status bar are on screen; header owns row 0, footer the last row, the block the rows between |
| `external_consumer_keyboard_input_produces_typed_actions` | `Down` → `Action::Focused(ItemKey)`, `Enter` → `Action::Chose(ItemKey)`; the app focuses the list itself on bootstrap |
| `external_consumer_pointer_input_produces_typed_actions` | Clicking a keyed recipe row → `Action::Chose(ItemKey)` |
| `external_consumer_summary_button_produces_a_typed_domain_action` | Clicking the detail action button → `Action::Scaled(ItemKey)`, domain state changes |
| `external_consumer_status_bar_reports_a_typed_action` | Clicking the keyed status item → `Action::StatusChosen(ItemKey)` |
| `external_consumer_collapses_to_one_pane_on_a_narrow_terminal` | At 48 columns the block collapses the detail pane, the list stays inside the terminal |
| `external_consumer_renders_identically_on_two_runs` | Two independent runs render a byte-identical frame, cell by cell |
| `manifest_depends_on_the_public_facade_only` | Manifest names the facade and no internal crate |
| `sources_import_only_the_public_facade` | No `use termrock_*` anywhere in the crate |

Validation on this source base:

```text
cargo nextest run --manifest-path external-consumer-demo/Cargo.toml
9 passed, 0 failed, 0 skipped

cargo clippy --manifest-path external-consumer-demo/Cargo.toml --all-targets
passed

cargo fmt --manifest-path external-consumer-demo/Cargo.toml --check
passed

cargo metadata --no-deps (workspace root)
external-consumer-demo not listed — not a workspace member
```

## Relationship to the prior branches

- `origin/wip/external-public-consumer-20261021` (`154495e2b`, `fd1e65243`):
  consumer crate source adopted here, re-verified against the tip, moved out
  of the workspace.
- `origin/wip/public-consumer-contracts-20261021` (`fc861e15a`):
  `crates/termrock/tests/public_component_contracts.rs` is owned by another
  workstream and is not landed or modified here.

## Re-run on the recovered PR #38 integration (2026-10-21)

The standalone crate was checked again after materializing the staged sparse-checkout paths in the isolated integration worktree. `cargo nextest run --locked --all-targets --manifest-path external-consumer-demo/Cargo.toml` passed 9/9 with zero skips; strict all-target Clippy and standalone formatting passed. Exact logs and source index tree are recorded in [the gate receipt](evidence/2026-10-21-external-consumer/receipt.json). Root workspace metadata does not include this crate. This clears the separate-consumer demonstration only; all four real applications still need shared-component adoption and paired parity proof.
