# Facade example contracts

This page records caller guidance for seven `termrock` examples. Source examples can show API setup. They do not prove behavior unless they drive the relevant input and inspect an observable result.

## Input and composition examples

- [`code_editor_blur_consumer`](../../crates/termrock/examples/code_editor_blur_consumer.rs) creates two editors through the public facade. It focuses the SQL editor at bootstrap and sets `commit_on_blur(false)` on the scratch editor. Its `main` prepares and draws one frame with `TestBackend`. It does not send focus-out input or inspect a commit result. Do not use it as proof of editor blur behavior.
- [`composition_consumer`](../../crates/termrock/examples/composition_consumer.rs) composes borrowed Button and Tabs components through the public facade. It renders one prepared frame. The workspace rows are borrowed by the Tabs composition.
- [`file_catalogue_master_detail`](../../crates/termrock/examples/file_catalogue_master_detail.rs) stores a caller-provided `&'static [CatalogueDocument]`. `with_documents` initializes editor text from the first document, or from an empty string when the slice is empty. Its update method calls `MasterDetail::update` before it updates the buttons and editor. It returns early when the block response is consumed. The caller contract gives the block first refusal on Escape. Its input iterator is observational for every other input, so the focused editor can receive that input. This example sends no input, so it does not verify the routing.

The file catalogue render helpers create the root `Rect` at the `Runtime::prepare` boundary. `render_with_policy_once` uses its caller-supplied policy. The example's `main` supplies a resizable-master policy. The helper names do not promise that every supplied policy is resizable.

- [`master_detail_composition_consumer`](../../crates/termrock/examples/master_detail_composition_consumer.rs) uses the public facade to compose a NavList inside a master-detail block and an AppShell body slot. Its `main` is empty. It does not run or draw the composition.
- [`semantic_item_row_consumer`](../../crates/termrock/examples/semantic_item_row_consumer.rs) maps application-owned `Document` values to semantic items with `AsItem` and `SemanticItemRow`. Its separate closure-row function keeps the closure callback form. The `.key` and `.node` calls provide the domain type for `.row`, so the example needs no explicit callback argument types. Its `main` is empty; the example source checks API use at compile time only.

## Picker examples

- [`picker_card_layout_consumer`](../../crates/termrock/examples/picker_card_layout_consumer.rs) configures a picker over a fixed three-item project array. It sets card layout, search, a row limit of 12, and geometric-center alignment. Its update method stores a chosen item key. Its `main` constructs a `Runtime`; it does not prepare or draw a frame.
- [`picker_row_defaults`](../../crates/termrock/examples/picker_row_defaults.rs) renders the same rows with Bar and Fill cursor styles. It also creates a focused ICON resolution after setting a same-value family base. The code discards the resolved value, so this example does not check the result. Treat the claim that an authored same-value base remains authoritative while focused as an open behavior requirement. The [theme resolution order](theming.md#resolution-order) and [override](overrides.md) contracts describe resolver order and require component-specific precedence checks.
