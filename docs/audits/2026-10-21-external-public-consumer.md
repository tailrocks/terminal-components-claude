# External public consumer of the Termrock library

Base under test: `300a97bf8ef78b079cc79acc948894029dbefc3b` (tip of
`parity/termrock-visual-baseline`).

## What this closes

`docs/architecture/termrock-migration-matrix.md` still recorded
"External public consumer — Not yet demonstrated for proposed composition API
with unrelated domain data and no private app helpers" as red. This increment
demonstrates it with a real, separate crate.

`crates/termrock-consumer-example` is a standalone workspace member. It
depends on the public `termrock` facade only, carries its own domain data (a
recipe shelf, unrelated to Showcase, Holla, TablePro or Jackin) and composes a
working screen through public APIs alone.

## The composed screen

```text
termrock::AppShell                     (termrock-shell, via termrock::blocks)
 ├── header : termrock::overlays::MenuBar
 ├── body   : termrock::blocks::MasterDetail
 │    ├── master  : termrock::navigation::NavList
 │    └── detail  : termrock::controls::PropsList
 │                  termrock::controls::Button   (action row)
 └── footer : termrock::feedback::StatusBar
```

The consumer implements `termrock::author::CompositionLeaf` for three leaves
(header, body, footer) so the shell can hand each one the region it resolved.
Those leaves own no geometry: they pass the resolved region straight to the
shared widget.

## Exact public API surface used

| Purpose | Public item |
| --- | --- |
| App contract | `termrock::App`, `termrock::Cx`, `termrock::Ui`, `termrock::Response` |
| Runtime | `termrock::Runtime` (`prepare`, `draw_prepared`, `acknowledge_presented`, `handle`, `area_of`, `area_of_part`, `focus`) |
| Theme | `termrock::Theme::junie` |
| Shell | `termrock::AppShell`, `termrock::Chrome` |
| Leaf protocol | `termrock::author::{CompositionLeaf, Measure, Constraints, Size}` |
| Layout block | `termrock::blocks::{MasterDetail, MasterDetailPolicy, MasterDetailState}` |
| Composition | `termrock::composition::{Group, button, props_list, nav_list}` |
| List widget | `termrock::navigation::{NavList, NavListState, NavListAction}` |
| Detail widget | `termrock::controls::{PropsList, PropsRow, PropsState, Button}` |
| Menu bar | `termrock::overlays::{MenuBar, Menu, MenuItem, MenuState, MenuAction}` |
| Status bar | `termrock::feedback::{StatusBar, StatusItem, StatusAction}` |
| Identifiers | `termrock::{Id, ItemKey, ActionKey, Part, PartRef}` |
| Events | `termrock::{Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind}` |
| Session (binary) | `termrock::run` |

The consumer never imports `termrock::author`'s painting surface for its own
content, never matches on `Input`, `Intent`, `KeyCode` or `MouseKind`, and
never builds a `Rect`.

## Typed update → typed action flow

The consumer defines its own domain action enum:

```text
pub enum Action {
    Focused(ItemKey),
    Chose(ItemKey),
    DetailOpened,
    DetailClosed,
    MenuChosen(ActionKey),
    StatusChosen(ItemKey),
    Scaled(ItemKey),
    Cooked(ItemKey),
}
```

Shared widgets answer with their own typed actions, and the consumer maps
them once, in one place:

| Widget | Typed action | Consumer action |
| --- | --- | --- |
| `NavList` | `NavListAction::Moved` / `Chose` / `EnterContent` | `Action::Focused` / `Action::Chose` |
| `MasterDetail` | `MasterDetailAction::DrawerOpened` / `DrawerClosed` | `Action::DetailOpened` / `Action::DetailClosed` |
| `MenuBar` | `MenuAction::Chosen(ActionKey)` | `Action::MenuChosen` |
| `StatusBar` | `StatusAction::Chose(ItemKey)` | `Action::StatusChosen` |
| `Button` | `Response<Activated>::activated()` | `Action::Scaled` / `Action::Cooked` |

`RecipeBox::dispatch` is the only place where a domain change happens, and
the typed action history is exposed for assertions (`RecipeBox::actions`,
`RecipeBox::last_action`).

## Dependency-graph proof

`crates/termrock-consumer-example/Cargo.toml` declares exactly one normal
dependency and one dev-dependency:

```toml
[dependencies]
termrock.workspace = true

[dev-dependencies]
ratatui.workspace = true
```

`cargo tree -p termrock-consumer-example --depth 1`:

```text
termrock-consumer-example v0.1.0 (/private/tmp/external-consumer-20261021/crates/termrock-consumer-example)
└── termrock v0.1.0 (/private/tmp/external-consumer-20261021/crates/termrock)
[dev-dependencies]
└── ratatui v0.30.2
```

The full transitive crate set of the consumer is the `termrock` facade, the
nineteen `termrock-*` library crates it re-exports (`termrock-core`,
`-layout`, `-theme`, `-text`, `-render`, `-collections`, `-runtime`,
`-controls`, `-fields`, `-feedback`, `-navigation`, `-viewport`, `-overlays`,
`-grid`, `-editors`, `-forms`, `-terminal`, `-session`, `-shell`) and Ratatui
with its own third-party dependencies.

A scan of the resolved tree for application and test-only crates returns
nothing:

```text
$ cargo tree -p termrock-consumer-example --prefix none --no-dedupe \
    | sed 's/ (.*//' | awk 'NF{print $1}' | sort -u \
    | grep -E "showcase|holla|tablepro|jackin|e2e|visual-harness|test-support|conformance|xtask"
$ echo $?
1
```

So `showcase`, `showcase-data`, `showcase-demos`, `showcase-ui`, `holla`,
`holla-domain`, `holla-catalog`, `holla-plan`, `holla-sim`, `holla-ui`,
`tablepro`, `tablepro-domain`, `tablepro-sql`, `tablepro-demo`, `tablepro-ui`,
`jackin-preview`, `jackin-preview-domain`, `jackin-preview-sim`,
`jackin-preview-presentation`, `jackin-preview-host-ui`,
`jackin-preview-capsule-ui`, `jackin-preview-app`, `termrock-e2e`,
`visual-harness`, `termrock-test-support`, `termrock-conformance` and
`termrock-xtask` are all unreachable from the consumer.

The dev-dependency on `ratatui` supplies only the test backend that drives
the public production render path; Ratatui is already inside the Termrock
dependency graph, so the crate adds nothing to it.

## Test evidence

`crates/termrock-consumer-example/tests/consumer.rs` drives the consumer
through the public production render path (`Runtime::prepare` →
`Runtime::draw_prepared` → `Runtime::acknowledge_presented`) on Ratatui's
public `TestBackend`, and asserts on rendered cells.

| Test | What it proves |
| --- | --- |
| `external_consumer_renders_its_own_domain_data_in_a_stable_layout` | The consumer's own recipes, the menu bar and the status bar are on screen; the menu bar owns row 0, the status bar owns the last row, and the master-detail block owns everything between them |
| `external_consumer_keyboard_input_produces_typed_actions` | `Down` produces `Action::Focused(ItemKey)`, `Enter` produces `Action::Chose(ItemKey)`; the app focuses the recipe list itself when the shelf opens |
| `external_consumer_pointer_input_produces_typed_actions` | Clicking a keyed recipe row produces `Action::Chose(ItemKey)` |
| `external_consumer_summary_button_produces_a_typed_domain_action` | Clicking the detail pane's action button produces `Action::Scaled(ItemKey)` and changes the domain state |
| `external_consumer_status_bar_reports_a_typed_action` | Clicking the keyed status item produces `Action::StatusChosen(ItemKey)` |
| `external_consumer_renders_identically_on_two_runs` | Two independent runs render a byte-identical frame, cell by cell |

Validation on this source base:

```text
cargo nextest run -p termrock-consumer-example --all-targets
6 passed, 0 failed, 0 skipped

cargo clippy -p termrock-consumer-example --all-targets -- -D warnings
passed

cargo fmt --check -p termrock-consumer-example
passed

cargo tree -p termrock-consumer-example --depth 1
termrock-consumer-example -> termrock (+ dev: ratatui)
```

The umbrella crate still builds and passes with its default feature set after
the contracts file below is registered:

```text
cargo nextest run -p termrock
23 passed, 0 failed, 0 skipped

cargo nextest run -p termrock --all-features --test public_component_contracts
3 passed, 0 failed, 0 skipped
```

## Relationship to the unintegrated contracts branch

`crates/termrock/tests/public_component_contracts.rs` and
`docs/audits/2026-10-21-public-consumer-contracts.md` come from the
unintegrated branch `wip/public-consumer-contracts-20261021` (tip
`fc861e15a`). That file is owned by another workstream, so it is incorporated
verbatim and not rewritten.

One mechanical addition was unavoidable: the file uses `Runtime::draw_buffer`
and `Runtime::set_focus`, which live behind the `testing` feature, so
`crates/termrock/Cargo.toml` needs the matching four-line test registration
(`[[test]] name = "public_component_contracts"`, `required-features =
["testing"]`). Without it Cargo's automatic test discovery would compile the
file without the feature and break the umbrella crate's own test build. The
umbrella crate's source was not touched.

The two deliveries are complementary: the contracts file pins component-level
contracts inside the facade's own test target, while the consumer crate
proves the same components compose into a working application from outside
the repository's application crates.

## Notes and limits

- The consumer's geometry-free rule holds for the application code
  (`crates/termrock-consumer-example/src/**`): there is no `Rect::new`, no
  `Rect { .. }`, no coordinate arithmetic, no buffer write and no direct
  painting. The only rectangle arithmetic in the crate is the root terminal
  area in the test harness, which is the framework boundary.
- The master-detail summary region is not used: the action row lives at the
  bottom of the detail pane instead, so it stays reachable in the wide
  split layout. The summary region is only painted in the collapsed layout.
- `MasterDetail` reports its resolved region through `Ui::report_layout`
  rather than as a hit region, so `Runtime::area_of(WORKSPACE)` is `None` by
  design. The layout assertions use the pane and shell regions instead.
- No public API gap was hit that needed a workaround: every element the
  consumer needed was reachable through the public facade.
